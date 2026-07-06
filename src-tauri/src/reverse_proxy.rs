use crate::{
    app_state::AppRuntime,
    models::{
        GlobalSettings, WEB_IP_WHITELIST_CHINA_MAINLAND, WebIpWhitelistEntry, WebSecurityBanRecord,
        WebSecurityUnbanResult,
    },
};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::Mutex,
    thread,
    time::Duration,
};
use tauri::{AppHandle, Manager};

const CONFIG_RELATIVE_PATH: &str = "conf/asa-web-openresty.conf";
const IP_WHITELIST_CIDR_RELATIVE_PATH: &str = "conf/asa-ip-whitelist-cidrs.txt";
const SECURITY_LUA_RELATIVE_PATH: &str = "lualib/asa_security.lua";
const OPENRESTY_EXECUTABLE_NAME: &str = "nginx.exe";
const PROXY_ROOT_DIR_NAME: &str = "web-reverse-proxy";
const STARTUP_PID_WAIT_STEP: Duration = Duration::from_millis(100);
const STARTUP_PID_WAIT_ATTEMPTS: usize = 20;
const RATE_LIMIT_PER_MINUTE: u32 = 180;
const API_RATE_LIMIT_PER_MINUTE: u32 = 120;
const LOGIN_RATE_LIMIT_PER_MINUTE: u32 = 10;
const PATH_RISK_BAN_THRESHOLD: u32 = 3;
const PATH_RISK_BAN_SECONDS: u32 = 60 * 60;
const BODY_RISK_BAN_SECONDS: u32 = 60 * 60;

#[derive(Default)]
pub struct ReverseProxyManager {
    active: Mutex<Option<ReverseProxyConfig>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReverseProxyConfig {
    openresty_executable_path: PathBuf,
    openresty_root_path: PathBuf,
    proxy_root_path: PathBuf,
    domain: String,
    public_port: u16,
    web_port: u16,
    login_failure_ban_threshold: u32,
    login_failure_ban_seconds: u32,
    ip_whitelist: Vec<WebIpWhitelistEntry>,
}

impl ReverseProxyManager {
    pub fn apply_settings(
        &self,
        runtime: &AppRuntime,
        settings: &GlobalSettings,
    ) -> Result<(), String> {
        if !settings.web_management_enabled || !settings.web_reverse_proxy_enabled {
            self.stop_current_best_effort();
            return Ok(());
        }

        validate_settings(settings)?;
        let desired = ReverseProxyConfig::from_settings(&runtime.data_dir(), settings)?;
        let already_active = self
            .active
            .lock()
            .map_err(|_| "Web 反向代理状态锁已损坏".to_string())?
            .as_ref()
            .is_some_and(|active| active == &desired);

        if already_active {
            return Ok(());
        }

        self.stop_current_best_effort();
        desired.stop_stale_instance_best_effort();
        desired.prepare_runtime_files()?;
        desired.test_config()?;
        desired.start()?;

        let mut active = self
            .active
            .lock()
            .map_err(|_| "Web 反向代理状态锁已损坏".to_string())?;
        *active = Some(desired);
        Ok(())
    }

    pub fn shutdown(&self) {
        self.stop_current_best_effort();
    }

    pub fn list_security_bans(&self) -> Result<Vec<WebSecurityBanRecord>, String> {
        let current = self
            .active
            .lock()
            .map_err(|_| "Web 反向代理状态锁已损坏".to_string())?
            .clone();
        let Some(config) = current else {
            return Ok(Vec::new());
        };
        config.list_security_bans()
    }

    pub fn unban_security_ip(&self, ip: &str) -> Result<WebSecurityUnbanResult, String> {
        let ip = normalize_admin_ip(ip)?;
        let current = self
            .active
            .lock()
            .map_err(|_| "Web 反向代理状态锁已损坏".to_string())?
            .clone()
            .ok_or_else(|| "OpenResty 反向代理当前未运行，无法手动解封 IP".to_string())?;
        current.unban_security_ip(&ip)
    }

    fn stop_current_best_effort(&self) {
        let current = self.active.lock().ok().and_then(|mut active| active.take());
        if let Some(config) = current {
            let _ = config.stop();
        }
    }
}

pub fn apply_settings_from_app(app: &AppHandle, settings: &GlobalSettings) -> Result<(), String> {
    let runtime = app
        .try_state::<AppRuntime>()
        .ok_or_else(|| "应用运行状态尚未初始化，无法应用 Web 反向代理".to_string())?;
    let manager = app
        .try_state::<ReverseProxyManager>()
        .ok_or_else(|| "Web 反向代理管理器尚未初始化".to_string())?;
    manager.apply_settings(&runtime, settings)
}

pub fn shutdown(app: &AppHandle) {
    if let Some(manager) = app.try_state::<ReverseProxyManager>() {
        manager.shutdown();
    }
}

pub fn list_security_bans_from_app(app: &AppHandle) -> Result<Vec<WebSecurityBanRecord>, String> {
    let manager = app
        .try_state::<ReverseProxyManager>()
        .ok_or_else(|| "Web 反向代理管理器尚未初始化".to_string())?;
    manager.list_security_bans()
}

pub fn unban_security_ip_from_app(
    app: &AppHandle,
    ip: &str,
) -> Result<WebSecurityUnbanResult, String> {
    let manager = app
        .try_state::<ReverseProxyManager>()
        .ok_or_else(|| "Web 反向代理管理器尚未初始化".to_string())?;
    manager.unban_security_ip(ip)
}

pub fn validate_settings(settings: &GlobalSettings) -> Result<(), String> {
    validate_security_settings(settings)?;

    if !settings.web_management_enabled || !settings.web_reverse_proxy_enabled {
        return Ok(());
    }

    let domain = normalize_domain(&settings.web_reverse_proxy_domain)?;
    if domain == "localhost" || domain.parse::<IpAddr>().is_ok() {
        return Err(
            "Web 反向代理访问域名必须是真实域名，不能填写 localhost 或 IP 地址".to_string(),
        );
    }

    if settings.web_reverse_proxy_port == 0 {
        return Err("Web 反向代理公开端口必须在 1-65535 之间".to_string());
    }
    if settings.web_reverse_proxy_port == settings.web_server_port {
        return Err("Web 反向代理公开端口不能与应用 Web 内部端口相同".to_string());
    }

    let _ = resolve_openresty_executable_path(&settings.web_reverse_proxy_openresty_path)?;
    Ok(())
}

pub fn should_enforce_proxy_host(settings: &GlobalSettings) -> bool {
    settings.web_management_enabled
        && settings.web_reverse_proxy_enabled
        && !settings.web_reverse_proxy_domain.trim().is_empty()
        && settings.web_reverse_proxy_port > 0
}

pub fn expected_proxy_host(settings: &GlobalSettings) -> Option<String> {
    if !should_enforce_proxy_host(settings) {
        return None;
    }
    let domain = normalize_domain(&settings.web_reverse_proxy_domain).ok()?;
    Some(format!("{domain}:{}", settings.web_reverse_proxy_port))
}

pub fn is_request_host_allowed(settings: &GlobalSettings, request_host: Option<&str>) -> bool {
    let Some(expected) = expected_proxy_host(settings) else {
        return true;
    };
    normalize_host_header(request_host.unwrap_or_default()).is_some_and(|host| host == expected)
}

impl ReverseProxyConfig {
    fn from_settings(data_dir: &Path, settings: &GlobalSettings) -> Result<Self, String> {
        let openresty_executable_path =
            resolve_openresty_executable_path(&settings.web_reverse_proxy_openresty_path)?;
        let openresty_root_path = openresty_executable_path
            .parent()
            .ok_or_else(|| "无法识别 OpenResty nginx.exe 所在目录".to_string())?
            .to_path_buf();

        Ok(Self {
            openresty_executable_path,
            openresty_root_path,
            proxy_root_path: data_dir.join(PROXY_ROOT_DIR_NAME),
            domain: normalize_domain(&settings.web_reverse_proxy_domain)?,
            public_port: settings.web_reverse_proxy_port,
            web_port: settings.web_server_port,
            login_failure_ban_threshold: settings.web_login_failure_ban_threshold,
            login_failure_ban_seconds: settings.web_login_failure_ban_seconds,
            ip_whitelist: normalize_ip_whitelist_entries(&settings.web_ip_whitelist),
        })
    }

    fn prepare_runtime_files(&self) -> Result<(), String> {
        fs::create_dir_all(self.proxy_root_path.join("conf")).map_err(|error| {
            format!(
                "无法创建 Web 反向代理配置目录 {}：{error}",
                self.proxy_root_path.join("conf").display()
            )
        })?;
        fs::create_dir_all(self.proxy_root_path.join("logs")).map_err(|error| {
            format!(
                "无法创建 Web 反向代理日志目录 {}：{error}",
                self.proxy_root_path.join("logs").display()
            )
        })?;
        fs::create_dir_all(self.proxy_root_path.join("temp")).map_err(|error| {
            format!(
                "无法创建 Web 反向代理临时目录 {}：{error}",
                self.proxy_root_path.join("temp").display()
            )
        })?;
        fs::create_dir_all(self.proxy_root_path.join("lualib")).map_err(|error| {
            format!(
                "无法创建 OpenResty Lua 脚本目录 {}：{error}",
                self.proxy_root_path.join("lualib").display()
            )
        })?;

        let config_path = self.config_path();
        fs::write(&config_path, self.render_config()).map_err(|error| {
            format!(
                "无法写入 Web 反向代理 OpenResty 配置 {}：{error}",
                config_path.display()
            )
        })?;

        let lua_path = self.security_lua_path();
        fs::write(&lua_path, self.render_security_lua()).map_err(|error| {
            format!(
                "无法写入 Web 反向代理 Lua 安全脚本 {}：{error}",
                lua_path.display()
            )
        })?;

        let cidr_path = self.ip_whitelist_cidr_path();
        let cidr_content = render_ip_whitelist_cidr_file(&self.ip_whitelist)?;
        fs::write(&cidr_path, cidr_content).map_err(|error| {
            format!(
                "无法写入 Web 访问 IP 白名单 CIDR 文件 {}：{error}",
                cidr_path.display()
            )
        })?;

        Ok(())
    }

    fn test_config(&self) -> Result<(), String> {
        let output = self.base_command().arg("-t").output().map_err(|error| {
            format!(
                "无法执行 OpenResty 配置校验 {}：{error}",
                self.openresty_executable_path.display()
            )
        })?;

        if output.status.success() {
            return Ok(());
        }

        Err(format!(
            "OpenResty 配置校验失败：{}",
            command_output_detail(&output)
        ))
    }

    fn start(&self) -> Result<(), String> {
        let mut command = self.base_command();
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        command.spawn().map_err(|error| {
            format!(
                "无法启动 OpenResty 反向代理 {}：{error}",
                self.openresty_executable_path.display()
            )
        })?;

        for _ in 0..STARTUP_PID_WAIT_ATTEMPTS {
            if self.pid_path().is_file() {
                return Ok(());
            }
            thread::sleep(STARTUP_PID_WAIT_STEP);
        }

        Err(format!(
            "OpenResty 已尝试启动，但未生成进程文件；请检查日志目录：{}",
            self.proxy_root_path.join("logs").display()
        ))
    }

    fn stop(&self) -> Result<(), String> {
        if !self.pid_path().exists() {
            return Ok(());
        }

        let _ = self
            .base_command()
            .arg("-s")
            .arg("quit")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    fn stop_stale_instance_best_effort(&self) {
        let _ = self.stop();
    }

    fn base_command(&self) -> Command {
        let mut command = Command::new(&self.openresty_executable_path);
        command
            .current_dir(&self.openresty_root_path)
            .arg("-p")
            .arg(&self.proxy_root_path)
            .arg("-c")
            .arg(CONFIG_RELATIVE_PATH);
        hide_console_window(&mut command);
        command
    }

    fn config_path(&self) -> PathBuf {
        self.proxy_root_path.join(CONFIG_RELATIVE_PATH)
    }

    fn security_lua_path(&self) -> PathBuf {
        self.proxy_root_path.join(SECURITY_LUA_RELATIVE_PATH)
    }

    fn ip_whitelist_cidr_path(&self) -> PathBuf {
        self.proxy_root_path.join(IP_WHITELIST_CIDR_RELATIVE_PATH)
    }

    fn pid_path(&self) -> PathBuf {
        self.proxy_root_path
            .join("logs")
            .join("asa-web-reverse-proxy.pid")
    }

    fn render_config(&self) -> String {
        format!(
            r#"# 由 ASA 服务端管理器自动生成，请勿手动编辑。
worker_processes  1;
pid logs/asa-web-reverse-proxy.pid;
error_log logs/asa-web-error.log warn;

events {{
    worker_connections  512;
}}

http {{
    access_log logs/asa-web-access.log;
    default_type application/octet-stream;
    sendfile on;
    keepalive_timeout 65;
    client_max_body_size 16m;
    client_body_buffer_size 128k;
    lua_package_path "lualib/?.lua;;";
    lua_shared_dict asa_ip_bans 10m;
    lua_shared_dict asa_rate_counters 20m;
    lua_shared_dict asa_login_failures 10m;
    lua_shared_dict asa_path_risks 10m;
    lua_shared_dict asa_cidr_cache 2m;

    init_by_lua_block {{
        require("asa_security").configure({{
            domain = "{domain}",
            public_port = {public_port},
            ip_whitelist_cidr_path = "{ip_whitelist_cidr_path}",
            login_failure_ban_threshold = {login_failure_ban_threshold},
            login_failure_ban_seconds = {login_failure_ban_seconds},
            rate_limit_per_minute = {rate_limit_per_minute},
            api_rate_limit_per_minute = {api_rate_limit_per_minute},
            login_rate_limit_per_minute = {login_rate_limit_per_minute},
            path_risk_ban_threshold = {path_risk_ban_threshold},
            path_risk_ban_seconds = {path_risk_ban_seconds},
            body_risk_ban_seconds = {body_risk_ban_seconds},
        }})
    }}

    map $http_upgrade $connection_upgrade {{
        default upgrade;
        '' close;
    }}

    server {{
        listen 0.0.0.0:{public_port} default_server;
        server_name _;
        return 403;
    }}

    server {{
        listen 0.0.0.0:{public_port};
        server_name {domain};

        access_by_lua_block {{
            require("asa_security").access()
        }}

        log_by_lua_block {{
            require("asa_security").log()
        }}

        location = /__asa_security/bans {{
            allow 127.0.0.1;
            deny all;
            access_by_lua_block {{}}
            content_by_lua_block {{
                require("asa_security").admin_bans()
            }}
        }}

        location = /__asa_security/unban {{
            allow 127.0.0.1;
            deny all;
            access_by_lua_block {{}}
            content_by_lua_block {{
                require("asa_security").admin_unban()
            }}
        }}

        location / {{
            proxy_pass http://127.0.0.1:{web_port};
            proxy_http_version 1.1;
            proxy_set_header Host $host:$server_port;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Host $host:$server_port;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection $connection_upgrade;
            proxy_buffering off;
            proxy_cache off;
            proxy_read_timeout 3600s;
            proxy_send_timeout 3600s;
        }}
    }}
}}
"#,
            public_port = self.public_port,
            domain = self.domain,
            web_port = self.web_port,
            ip_whitelist_cidr_path = IP_WHITELIST_CIDR_RELATIVE_PATH.replace('\\', "/"),
            login_failure_ban_threshold = self.login_failure_ban_threshold,
            login_failure_ban_seconds = self.login_failure_ban_seconds,
            rate_limit_per_minute = RATE_LIMIT_PER_MINUTE,
            api_rate_limit_per_minute = API_RATE_LIMIT_PER_MINUTE,
            login_rate_limit_per_minute = LOGIN_RATE_LIMIT_PER_MINUTE,
            path_risk_ban_threshold = PATH_RISK_BAN_THRESHOLD,
            path_risk_ban_seconds = PATH_RISK_BAN_SECONDS,
            body_risk_ban_seconds = BODY_RISK_BAN_SECONDS,
        )
    }

    fn render_security_lua(&self) -> String {
        SECURITY_LUA_SCRIPT.to_string()
    }

    fn list_security_bans(&self) -> Result<Vec<WebSecurityBanRecord>, String> {
        let payload = self.admin_json_request("GET", "/__asa_security/bans")?;
        let data = extract_admin_data(payload)?;
        serde_json::from_value::<Vec<WebSecurityBanRecord>>(data)
            .map_err(|error| format!("解析 OpenResty 封禁记录失败：{error}"))
    }

    fn unban_security_ip(&self, ip: &str) -> Result<WebSecurityUnbanResult, String> {
        let payload = self.admin_json_request("POST", &format!("/__asa_security/unban?ip={ip}"))?;
        let data = extract_admin_data(payload)?;
        serde_json::from_value::<WebSecurityUnbanResult>(data)
            .map_err(|error| format!("解析 OpenResty 解封结果失败：{error}"))
    }

    fn admin_json_request(&self, method: &str, path: &str) -> Result<Value, String> {
        let address = SocketAddr::from(([127, 0, 0, 1], self.public_port));
        let mut stream =
            TcpStream::connect_timeout(&address, Duration::from_secs(2)).map_err(|error| {
                format!(
                    "无法连接 OpenResty 本机管理端口 127.0.0.1:{}：{error}",
                    self.public_port
                )
            })?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| format!("设置 OpenResty 管理请求读取超时失败：{error}"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| format!("设置 OpenResty 管理请求写入超时失败：{error}"))?;

        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {domain}:{port}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
            domain = self.domain,
            port = self.public_port,
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| format!("发送 OpenResty 管理请求失败：{error}"))?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|error| format!("读取 OpenResty 管理响应失败：{error}"))?;
        parse_admin_http_response(&response)
    }
}

const SECURITY_LUA_SCRIPT: &str = r##"
local _M = {}

local config = {
    domain = "",
    public_port = 0,
    ip_whitelist_cidr_path = "conf/asa-ip-whitelist-cidrs.txt",
    login_failure_ban_threshold = 5,
    login_failure_ban_seconds = 1800,
    rate_limit_per_minute = 180,
    api_rate_limit_per_minute = 120,
    login_rate_limit_per_minute = 10,
    path_risk_ban_threshold = 3,
    path_risk_ban_seconds = 3600,
    body_risk_ban_seconds = 3600,
    redis_enabled = false,
    redis_host = "127.0.0.1",
    redis_port = 6379,
    redis_timeout_ms = 80,
    redis_whitelist_key = "asa:web:ip:whitelist",
    redis_blacklist_key = "asa:web:ip:blacklist",
}

local abnormal_ua_patterns = {
    "sqlmap",
    "nikto",
    "nmap",
    "masscan",
    "zgrab",
    "acunetix",
    "nessus",
    "openvas",
    "dirbuster",
    "gobuster",
    "wpscan",
    "curl",
    "wget",
    "python%-requests",
    "go%-http%-client",
    "java/",
    "powershell",
    "winhttp",
}

local dangerous_body_patterns = {
    "__proto__",
    "constructor%.prototype",
    "constructor%[prototype%]",
    "%$where",
    "<script",
    "javascript:",
    "%${jndi:",
    "%.%.%/",
    "%.%.\\",
    "%%2e%%2e",
    "union%s+select",
    "drop%s+table",
    "insert%s+into",
    "delete%s+from",
    "cmd%.exe",
    "powershell",
    "certutil",
    "bitsadmin",
}

local suspicious_path_patterns = {
    "%.env",
    "wp%-login%.php",
    "phpmyadmin",
    "server%-status",
    "%.git",
    "%.svn",
    "%.bak$",
    "%.sql$",
    "%.zip$",
    "%.tar",
    "%.7z$",
    "%%2e%%2e",
    "%.%.%/",
    "%.%.\\",
    "/admin",
    "/actuator",
    "/boaform",
    "/cgi%-bin",
}

function _M.configure(next_config)
    if type(next_config) ~= "table" then
        return
    end
    for key, value in pairs(next_config) do
        config[key] = value
    end
end

local function json_escape(value)
    value = tostring(value or "")
    value = value:gsub("\\", "\\\\")
    value = value:gsub('"', '\\"')
    value = value:gsub("\r", "\\r")
    value = value:gsub("\n", "\\n")
    return value
end

local function deny(status, message)
    ngx.status = status
    ngx.header["Content-Type"] = "application/json; charset=utf-8"
    ngx.say('{"ok":false,"error":"' .. json_escape(message) .. '"}')
    return ngx.exit(status)
end

local function json_response(status, body)
    ngx.status = status
    ngx.header["Content-Type"] = "application/json; charset=utf-8"
    ngx.say(body)
    return ngx.exit(status)
end

local function client_ip()
    return ngx.var.remote_addr or "0.0.0.0"
end

local function now_bucket(seconds)
    return math.floor(ngx.now() / seconds)
end

local function incr(dict, key, ttl)
    local value, err = dict:incr(key, 1, 0, ttl)
    if value then
        return value
    end
    dict:set(key, 1, ttl)
    return 1
end

local function ban_source(reason)
    reason = tostring(reason or "")
    if reason:find("登录") then
        return "login"
    end
    if reason:find("User%-Agent") or reason:find("User-Agent") then
        return "ua"
    end
    if reason:find("请求体") then
        return "body"
    end
    if reason:find("路径") then
        return "path"
    end
    if reason:find("频繁") then
        return "rate"
    end
    return "security"
end

local function encode_ban_record(reason, seconds)
    local safe_reason = tostring(reason or "blocked")
    return table.concat({
        safe_reason,
        tostring(math.floor(ngx.now() * 1000)),
        tostring(tonumber(seconds) or 0),
        ban_source(safe_reason),
    }, "\t")
end

local function shared_dict_ttl(dict, key)
    local ok, ttl = pcall(function()
        return dict:ttl(key)
    end)
    if ok and type(ttl) == "number" then
        return ttl
    end
    return 0
end

local function decode_ban_record(value, remaining_seconds)
    local text = tostring(value or "")
    local reason, banned_at_ms, duration_seconds, source = text:match("^([^\t]*)\t([^\t]*)\t([^\t]*)\t([^\t]*)$")
    if not reason then
        reason = text ~= "" and text or "blocked"
        banned_at_ms = "0"
        duration_seconds = "0"
        source = ban_source(reason)
    end
    return {
        reason = reason,
        banned_at_ms = tonumber(banned_at_ms) or 0,
        duration_seconds = tonumber(duration_seconds) or 0,
        source = source or ban_source(reason),
        remaining_seconds = math.max(0, math.floor(tonumber(remaining_seconds) or 0)),
    }
end

local function ban_ip(ip, seconds, reason)
    ngx.shared.asa_ip_bans:set("ban:" .. ip, encode_ban_record(reason, seconds), seconds)
end

local function ip_to_number(ip)
    local a, b, c, d = ip:match("^(%d+)%.(%d+)%.(%d+)%.(%d+)$")
    a, b, c, d = tonumber(a), tonumber(b), tonumber(c), tonumber(d)
    if not a or not b or not c or not d then
        return nil
    end
    if a > 255 or b > 255 or c > 255 or d > 255 then
        return nil
    end
    return ((a * 256 + b) * 256 + c) * 256 + d
end

local function mask(bits)
    if bits <= 0 then
        return 0
    end
    return (0xffffffff - (2 ^ (32 - bits) - 1))
end

local function cidr_match(ip_num, cidr)
    local network, bits = cidr:match("^%s*(%d+%.%d+%.%d+%.%d+)%s*/%s*(%d+)%s*$")
    bits = tonumber(bits)
    if not network or not bits or bits < 0 or bits > 32 then
        return false
    end
    local network_num = ip_to_number(network)
    if not network_num then
        return false
    end
    local m = mask(bits)
    return (ip_num - (ip_num % (2 ^ (32 - bits)))) == (network_num - (network_num % (2 ^ (32 - bits))))
end

local function is_private_or_loopback(ip)
    local ip_num = ip_to_number(ip)
    if not ip_num then
        return false
    end
    return cidr_match(ip_num, "10.0.0.0/8")
        or cidr_match(ip_num, "127.0.0.0/8")
        or cidr_match(ip_num, "172.16.0.0/12")
        or cidr_match(ip_num, "192.168.0.0/16")
        or cidr_match(ip_num, "100.64.0.0/10")
end

local function load_ip_whitelist_cidrs()
    local cache = ngx.shared.asa_cidr_cache
    local cached = cache:get("ip_whitelist_cidrs")
    if cached then
        local cidrs = {}
        for cidr in cached:gmatch("[^\n]+") do
            cidrs[#cidrs + 1] = cidr
        end
        return cidrs
    end

    local file = io.open(config.ip_whitelist_cidr_path, "r")
    local cidrs = {}
    if file then
        for line in file:lines() do
            local cidr = line:gsub("#.*$", ""):match("^%s*(.-)%s*$")
            if cidr ~= "" then
                cidrs[#cidrs + 1] = cidr
            end
        end
        file:close()
    end

    cache:set("ip_whitelist_cidrs", table.concat(cidrs, "\n"), 300)
    return cidrs
end

local function is_ip_whitelisted(ip)
    if is_private_or_loopback(ip) then
        return true
    end
    local ip_num = ip_to_number(ip)
    if not ip_num then
        return false
    end
    for _, cidr in ipairs(load_ip_whitelist_cidrs()) do
        if cidr_match(ip_num, cidr) then
            return true
        end
    end
    return false
end

local function redis_ip_policy(ip)
    if not config.redis_enabled then
        return nil
    end
    local ok, redis = pcall(require, "resty.redis")
    if not ok then
        return nil
    end
    local red = redis:new()
    red:set_timeout(config.redis_timeout_ms)
    local connected = red:connect(config.redis_host, config.redis_port)
    if not connected then
        return nil
    end
    local whitelist = red:sismember(config.redis_whitelist_key, ip)
    if whitelist == 1 then
        red:set_keepalive(10000, 16)
        return "allow"
    end
    local blacklist = red:sismember(config.redis_blacklist_key, ip)
    red:set_keepalive(10000, 16)
    if blacklist == 1 then
        return "deny"
    end
    return nil
end

local function has_abnormal_ua()
    local ua = (ngx.var.http_user_agent or ""):lower()
    if ua == "" then
        return true, "空 User-Agent"
    end
    for _, pattern in ipairs(abnormal_ua_patterns) do
        if ua:find(pattern) then
            return true, "异常 User-Agent"
        end
    end
    return false, nil
end

local function is_suspicious_path(uri)
    local lower_uri = (uri or ""):lower()
    for _, pattern in ipairs(suspicious_path_patterns) do
        if lower_uri:find(pattern) then
            return true
        end
    end
    return false
end

local function request_body_has_danger()
    local method = ngx.req.get_method()
    if method ~= "POST" and method ~= "PUT" and method ~= "PATCH" then
        return false
    end
    local content_type = (ngx.var.content_type or ""):lower()
    if content_type ~= "" and not content_type:find("json") and not content_type:find("form") and not content_type:find("text") then
        return false
    end
    ngx.req.read_body()
    local body = ngx.req.get_body_data()
    if not body then
        return false
    end
    local lower_body = body:lower()
    for _, pattern in ipairs(dangerous_body_patterns) do
        if lower_body:find(pattern) then
            return true
        end
    end
    return false
end

local function enforce_rate(ip, uri)
    local bucket = now_bucket(60)
    local counters = ngx.shared.asa_rate_counters
    local total = incr(counters, "total:" .. ip .. ":" .. bucket, 70)
    if total > config.rate_limit_per_minute then
        return false, "访问过于频繁"
    end
    if uri:find("^/api/") then
        local api = incr(counters, "api:" .. ip .. ":" .. bucket, 70)
        if api > config.api_rate_limit_per_minute then
            return false, "API 访问过于频繁"
        end
    end
    if uri == "/api/auth/login" then
        local login = incr(counters, "login:" .. ip .. ":" .. bucket, 70)
        if login > config.login_rate_limit_per_minute then
            ban_ip(ip, config.login_failure_ban_seconds, "登录接口访问过于频繁")
            return false, "登录接口访问过于频繁，IP 已临时封禁"
        end
    end
    return true, nil
end

local function enforce_path_risk(ip, uri)
    if not is_suspicious_path(uri) then
        return true, nil
    end
    local risks = incr(ngx.shared.asa_path_risks, "path:" .. ip, 600)
    if risks >= config.path_risk_ban_threshold then
        ban_ip(ip, config.path_risk_ban_seconds, "异常路径探测")
        return false, "异常路径探测过多，IP 已临时封禁"
    end
    return false, "异常路径已拦截"
end

local function ban_record_json(ip, record)
    return '{'
        .. '"ip":"' .. json_escape(ip) .. '",'
        .. '"reason":"' .. json_escape(record.reason) .. '",'
        .. '"source":"' .. json_escape(record.source) .. '",'
        .. '"bannedAtMs":' .. tostring(record.banned_at_ms) .. ','
        .. '"remainingSeconds":' .. tostring(record.remaining_seconds)
        .. '}'
end

function _M.admin_bans()
    local bans = ngx.shared.asa_ip_bans
    local ok, keys = pcall(function()
        return bans:get_keys(0)
    end)
    if not ok or type(keys) ~= "table" then
        keys = {}
    end
    table.sort(keys)

    local records = {}
    for _, key in ipairs(keys) do
        local ip = key:match("^ban:(.+)$")
        if ip then
            local value = bans:get(key)
            if value then
                local ttl = shared_dict_ttl(bans, key)
                local record = decode_ban_record(value, ttl)
                records[#records + 1] = ban_record_json(ip, record)
            end
        end
    end

    return json_response(200, '{"ok":true,"data":[' .. table.concat(records, ",") .. ']}')
end

function _M.admin_unban()
    local ip = ngx.var.arg_ip or ""
    if not ip_to_number(ip) then
        return json_response(400, '{"ok":false,"error":"手动解封 IP 必须是合法 IPv4 地址"}')
    end

    local key = "ban:" .. ip
    local bans = ngx.shared.asa_ip_bans
    local existed = bans:get(key) ~= nil
    bans:delete(key)
    ngx.shared.asa_login_failures:delete("login:" .. ip)
    ngx.shared.asa_path_risks:delete("path:" .. ip)

    return json_response(
        200,
        '{"ok":true,"data":{"ip":"' .. json_escape(ip) .. '","existed":' .. tostring(existed) .. '}}'
    )
end

function _M.access()
    local ip = client_ip()
    local uri = ngx.var.uri or "/"
    local policy = redis_ip_policy(ip)
    if policy == "allow" then
        return
    end
    if policy == "deny" then
        return deny(403, "IP 已命中 Redis 黑名单")
    end

    local ban_key = "ban:" .. ip
    local ban_value = ngx.shared.asa_ip_bans:get(ban_key)
    if ban_value then
        local ban_record = decode_ban_record(ban_value, shared_dict_ttl(ngx.shared.asa_ip_bans, ban_key))
        return deny(403, "IP 已被临时封禁：" .. ban_record.reason)
    end

    if not is_ip_whitelisted(ip) then
        return deny(451, "当前 IP 不在 Web 访问白名单")
    end

    local ua_blocked, ua_reason = has_abnormal_ua()
    if ua_blocked then
        ban_ip(ip, 600, ua_reason)
        return deny(403, ua_reason)
    end

    local ok, rate_reason = enforce_rate(ip, uri)
    if not ok then
        return deny(429, rate_reason)
    end

    local safe_path, path_reason = enforce_path_risk(ip, uri)
    if not safe_path then
        return deny(403, path_reason)
    end

    if request_body_has_danger() then
        ban_ip(ip, config.body_risk_ban_seconds, "请求体包含危险字段")
        return deny(403, "请求体包含危险字段，IP 已临时封禁")
    end
end

function _M.log()
    if (ngx.var.uri or "") ~= "/api/auth/login" then
        return
    end
    local ip = client_ip()
    local status = tonumber(ngx.status) or 0
    local failures = ngx.shared.asa_login_failures
    if status == 401 then
        local count = incr(failures, "login:" .. ip, 600)
        if count >= config.login_failure_ban_threshold then
            failures:delete("login:" .. ip)
            ban_ip(ip, config.login_failure_ban_seconds, "登录失败次数过多")
        end
    elseif status >= 200 and status < 300 then
        failures:delete("login:" .. ip)
    end
end

return _M
"##;

fn default_mainland_cidr_file() -> &'static str {
    include_str!("../resources/asa-mainland-cidrs.txt")
}

fn normalize_admin_ip(ip: &str) -> Result<String, String> {
    let trimmed = ip.trim();
    let ip = trimmed
        .parse::<Ipv4Addr>()
        .map_err(|_| "手动解封 IP 必须是合法 IPv4 地址".to_string())?;
    Ok(ip.to_string())
}

fn extract_admin_data(payload: Value) -> Result<Value, String> {
    if payload.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(payload.get("data").cloned().unwrap_or(Value::Null));
    }

    Err(payload
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("OpenResty 管理接口返回失败")
        .to_string())
}

fn parse_admin_http_response(response: &[u8]) -> Result<Value, String> {
    let header_end = find_http_header_end(response)
        .ok_or_else(|| "OpenResty 管理接口响应格式无效：缺少 HTTP 头".to_string())?;
    let header = String::from_utf8_lossy(&response[..header_end]);
    let status_line = header
        .lines()
        .next()
        .ok_or_else(|| "OpenResty 管理接口响应格式无效：缺少状态行".to_string())?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| format!("OpenResty 管理接口状态行无效：{status_line}"))?;

    let mut body = response[header_end + 4..].to_vec();
    if header
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        body = decode_chunked_body(&body)?;
    }

    if !(200..=299).contains(&status_code) {
        let detail: String = String::from_utf8_lossy(&body).chars().take(500).collect();
        return Err(format!(
            "OpenResty 管理接口返回 HTTP {status_code}：{detail}"
        ));
    }

    serde_json::from_slice::<Value>(&body)
        .map_err(|error| format!("解析 OpenResty 管理接口 JSON 失败：{error}"))
}

fn find_http_header_end(response: &[u8]) -> Option<usize> {
    response.windows(4).position(|window| window == b"\r\n\r\n")
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut cursor = 0;
    let mut decoded = Vec::new();

    loop {
        let Some(line_end) = find_crlf(body, cursor) else {
            return Err("OpenResty 管理接口分块响应格式无效：缺少块长度".to_string());
        };
        let size_text = String::from_utf8_lossy(&body[cursor..line_end]);
        let size_hex = size_text.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| format!("OpenResty 管理接口分块长度无效：{size_hex}"))?;
        cursor = line_end + 2;

        if size == 0 {
            break;
        }
        if body.len() < cursor + size + 2 {
            return Err("OpenResty 管理接口分块响应被截断".to_string());
        }
        decoded.extend_from_slice(&body[cursor..cursor + size]);
        cursor += size + 2;
    }

    Ok(decoded)
}

fn find_crlf(body: &[u8], start: usize) -> Option<usize> {
    body.get(start..)?
        .windows(2)
        .position(|window| window == b"\r\n")
        .map(|offset| start + offset)
}

fn validate_security_settings(settings: &GlobalSettings) -> Result<(), String> {
    if !(1..=100).contains(&settings.web_login_failure_ban_threshold) {
        return Err("Web 登录失败封禁阈值必须在 1-100 次之间".to_string());
    }
    if !(1..=86_400).contains(&settings.web_login_failure_ban_seconds) {
        return Err("Web 登录失败封禁时长必须在 1-86400 秒之间".to_string());
    }
    validate_ip_whitelist_entries(&settings.web_ip_whitelist)
}

fn validate_ip_whitelist_entries(entries: &[WebIpWhitelistEntry]) -> Result<(), String> {
    for entry in normalize_ip_whitelist_entries(entries) {
        if entry.group.chars().count() > 32 {
            return Err(format!(
                "Web IP 白名单条目 {} 的分组不能超过 32 个字符",
                entry.value
            ));
        }
        if entry.note.chars().count() > 120 {
            return Err(format!(
                "Web IP 白名单条目 {} 的备注不能超过 120 个字符",
                entry.value
            ));
        }
        if entry
            .value
            .eq_ignore_ascii_case(WEB_IP_WHITELIST_CHINA_MAINLAND)
        {
            continue;
        }
        let _ = normalize_ipv4_whitelist_entry(&entry.value)?;
    }
    Ok(())
}

fn normalize_ip_whitelist_entries(entries: &[WebIpWhitelistEntry]) -> Vec<WebIpWhitelistEntry> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for entry in entries {
        let value = entry.value.trim();
        if value.is_empty() {
            continue;
        }
        let normalized_value = if value.eq_ignore_ascii_case(WEB_IP_WHITELIST_CHINA_MAINLAND) {
            WEB_IP_WHITELIST_CHINA_MAINLAND.to_string()
        } else {
            value.to_string()
        };
        let normalized_entry = WebIpWhitelistEntry::new(
            normalized_value,
            entry.group.trim().to_string(),
            entry.note.trim().to_string(),
        );

        if seen.insert(normalized_entry.value.clone()) {
            normalized.push(normalized_entry);
        }
    }

    if normalized.is_empty() {
        vec![WebIpWhitelistEntry::new(
            WEB_IP_WHITELIST_CHINA_MAINLAND,
            "默认",
            "内置中国大陆 IPv4 CIDR",
        )]
    } else {
        normalized
    }
}

fn render_ip_whitelist_cidr_file(entries: &[WebIpWhitelistEntry]) -> Result<String, String> {
    let mut rendered = String::from(
        "# 由 ASA 服务端管理器自动生成，请勿手动编辑。\n\
         # 持久化来源：全局设置 webIpWhitelist；本文件只是运行时派生 CIDR 文件。\n\
         # CN_MAINLAND 表示内置中国大陆 IPv4 CIDR；单个 IPv4 会自动转换为 /32。\n",
    );
    let mut seen = HashSet::new();

    for entry in normalize_ip_whitelist_entries(entries) {
        let group = entry.group.trim();
        let note = entry.note.trim();
        if !group.is_empty() || !note.is_empty() {
            rendered.push_str(&format!(
                "# 条目：{}{}{}\n",
                entry.value,
                if group.is_empty() {
                    String::new()
                } else {
                    format!("；分组：{group}")
                },
                if note.is_empty() {
                    String::new()
                } else {
                    format!("；备注：{note}")
                }
            ));
        }

        if entry
            .value
            .eq_ignore_ascii_case(WEB_IP_WHITELIST_CHINA_MAINLAND)
        {
            rendered.push_str("# CN_MAINLAND BEGIN\n");
            for cidr in default_mainland_cidr_file()
                .lines()
                .filter_map(clean_cidr_line)
            {
                if seen.insert(cidr.to_string()) {
                    rendered.push_str(cidr);
                    rendered.push('\n');
                }
            }
            rendered.push_str("# CN_MAINLAND END\n");
            continue;
        }

        let cidr = normalize_ipv4_whitelist_entry(&entry.value)?;
        if seen.insert(cidr.clone()) {
            rendered.push_str(&cidr);
            rendered.push('\n');
        }
    }

    Ok(rendered)
}

fn clean_cidr_line(line: &str) -> Option<&str> {
    let cidr = line
        .split_once('#')
        .map(|(value, _)| value)
        .unwrap_or(line)
        .trim();
    (!cidr.is_empty()).then_some(cidr)
}

fn normalize_ipv4_whitelist_entry(entry: &str) -> Result<String, String> {
    if let Ok(ip) = entry.parse::<Ipv4Addr>() {
        return Ok(format!("{ip}/32"));
    }

    let Some((ip_text, prefix_text)) = entry.split_once('/') else {
        return Err(format!(
            "Web IP 白名单条目无效：{entry}，仅支持 CN_MAINLAND、IPv4 或 IPv4 CIDR"
        ));
    };
    if prefix_text.contains('/') {
        return Err(format!(
            "Web IP 白名单条目无效：{entry}，CIDR 前缀格式不正确"
        ));
    }

    let ip = ip_text
        .parse::<Ipv4Addr>()
        .map_err(|_| format!("Web IP 白名单条目无效：{entry}，IP 必须是 IPv4 地址"))?;
    let prefix = prefix_text
        .parse::<u8>()
        .map_err(|_| format!("Web IP 白名单条目无效：{entry}，CIDR 前缀必须是 0-32"))?;
    if prefix > 32 {
        return Err(format!(
            "Web IP 白名单条目无效：{entry}，CIDR 前缀必须是 0-32"
        ));
    }

    Ok(format!("{ip}/{prefix}"))
}

fn resolve_openresty_executable_path(path_text: &str) -> Result<PathBuf, String> {
    let trimmed = path_text.trim().trim_matches('"');
    if trimmed.is_empty() {
        return Err("启用 Web 反向代理时必须填写 OpenResty 安装目录或 nginx.exe 路径".to_string());
    }

    let path = PathBuf::from(trimmed);
    let executable = if path.is_dir() {
        path.join(OPENRESTY_EXECUTABLE_NAME)
    } else {
        path
    };

    let file_name = executable
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !file_name.eq_ignore_ascii_case(OPENRESTY_EXECUTABLE_NAME) {
        return Err("OpenResty 路径必须指向安装目录或 nginx.exe".to_string());
    }
    if !executable.is_file() {
        return Err(format!(
            "未找到 OpenResty nginx.exe：{}",
            executable.display()
        ));
    }
    Ok(executable)
}

fn normalize_domain(domain: &str) -> Result<String, String> {
    let normalized = domain.trim().trim_end_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("启用 Web 反向代理时必须填写访问域名".to_string());
    }
    if normalized.contains("://")
        || normalized.contains('/')
        || normalized.contains('\\')
        || normalized.contains(':')
        || normalized.contains('*')
    {
        return Err("访问域名只填写主机名，不包含协议、端口、路径或通配符".to_string());
    }
    if normalized.len() > 253 {
        return Err("访问域名不能超过 253 个字符".to_string());
    }
    if !normalized.split('.').all(is_valid_domain_label) {
        return Err("访问域名格式无效，只支持字母、数字、短横线和点号".to_string());
    }
    Ok(normalized)
}

fn is_valid_domain_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn normalize_host_header(host: &str) -> Option<String> {
    let value = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if value.is_empty() || value.contains('/') || value.contains('\\') {
        return None;
    }

    let (host, port) = value.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    let domain = normalize_domain(host).ok()?;
    Some(format!("{domain}:{port}"))
}

fn command_output_detail(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|value| !value.is_empty())
        .unwrap_or("无详细输出");
    detail.chars().take(1000).collect()
}

#[cfg(windows)]
fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console_window(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxy_settings() -> GlobalSettings {
        GlobalSettings {
            web_management_enabled: true,
            web_reverse_proxy_enabled: true,
            web_reverse_proxy_domain: "asa.example.com".to_string(),
            web_reverse_proxy_port: 18081,
            web_server_port: 18080,
            web_reverse_proxy_openresty_path: "C:\\openresty\\nginx.exe".to_string(),
            ..GlobalSettings::default()
        }
    }

    fn proxy_config() -> ReverseProxyConfig {
        ReverseProxyConfig {
            openresty_executable_path: PathBuf::from("C:\\openresty\\nginx.exe"),
            openresty_root_path: PathBuf::from("C:\\openresty"),
            proxy_root_path: PathBuf::from("D:\\ASA\\proxy"),
            domain: "asa.example.com".to_string(),
            public_port: 18081,
            web_port: 18080,
            login_failure_ban_threshold: 5,
            login_failure_ban_seconds: 1800,
            ip_whitelist: vec![WebIpWhitelistEntry::new(
                WEB_IP_WHITELIST_CHINA_MAINLAND,
                "默认",
                "内置中国大陆 IPv4 CIDR",
            )],
        }
    }

    #[test]
    fn 反代_host_必须匹配域名和端口() {
        let settings = proxy_settings();

        assert!(is_request_host_allowed(
            &settings,
            Some("ASA.EXAMPLE.COM:18081")
        ));
        assert!(!is_request_host_allowed(&settings, Some("127.0.0.1:18080")));
        assert!(!is_request_host_allowed(&settings, Some("asa.example.com")));
    }

    #[test]
    fn 未启用反代时不限制_host() {
        let mut settings = proxy_settings();
        settings.web_reverse_proxy_enabled = false;

        assert!(is_request_host_allowed(&settings, Some("127.0.0.1:18080")));
    }

    #[test]
    fn 未启用_web_管理时不限制_host() {
        let mut settings = proxy_settings();
        settings.web_management_enabled = false;

        assert!(is_request_host_allowed(&settings, Some("127.0.0.1:18080")));
    }

    #[test]
    fn 生成_openresty_配置包含_lua_安全网关与反代_host() {
        let mut config = proxy_config();
        config.login_failure_ban_threshold = 7;
        config.login_failure_ban_seconds = 900;

        let rendered = config.render_config();

        assert!(rendered.contains("listen 0.0.0.0:18081 default_server;"));
        assert!(rendered.contains("return 403;"));
        assert!(rendered.contains("server_name asa.example.com;"));
        assert!(rendered.contains("lua_shared_dict asa_ip_bans"));
        assert!(rendered.contains("access_by_lua_block"));
        assert!(rendered.contains("log_by_lua_block"));
        assert!(rendered.contains("/__asa_security/bans"));
        assert!(rendered.contains("/__asa_security/unban"));
        assert!(rendered.contains("login_failure_ban_threshold = 7"));
        assert!(rendered.contains("login_failure_ban_seconds = 900"));
        assert!(rendered.contains("ip_whitelist_cidr_path = \"conf/asa-ip-whitelist-cidrs.txt\""));
        assert!(rendered.contains("proxy_pass http://127.0.0.1:18080;"));
        assert!(rendered.contains("proxy_set_header Host $host:$server_port;"));
    }

    #[test]
    fn 生成_lua_安全脚本包含动态封禁和风险检测() {
        let config = proxy_config();

        let lua = config.render_security_lua();

        assert!(lua.contains("login_failure_ban_threshold"));
        assert!(lua.contains("admin_bans"));
        assert!(lua.contains("admin_unban"));
        assert!(lua.contains("abnormal_ua_patterns"));
        assert!(lua.contains("dangerous_body_patterns"));
        assert!(lua.contains("当前 IP 不在 Web 访问白名单"));
        assert!(lua.contains("redis_whitelist_key"));
        assert!(lua.contains("redis_blacklist_key"));
    }

    #[test]
    fn 默认全局设置使用中国大陆_ip_白名单() {
        assert_eq!(
            GlobalSettings::default().web_ip_whitelist,
            vec![WebIpWhitelistEntry::new(
                WEB_IP_WHITELIST_CHINA_MAINLAND,
                "默认",
                "内置中国大陆 IPv4 CIDR"
            )]
        );
    }

    #[test]
    fn 旧版字符串白名单会反序列化为结构化条目() {
        let mut value = serde_json::to_value(GlobalSettings::default()).unwrap();
        value["webIpWhitelist"] =
            serde_json::json!([WEB_IP_WHITELIST_CHINA_MAINLAND, "203.0.113.10"]);

        let settings = serde_json::from_value::<GlobalSettings>(value).unwrap();

        assert_eq!(
            settings.web_ip_whitelist[0],
            WebIpWhitelistEntry::new(WEB_IP_WHITELIST_CHINA_MAINLAND, "", "")
        );
        assert_eq!(
            settings.web_ip_whitelist[1],
            WebIpWhitelistEntry::new("203.0.113.10", "", "")
        );
    }

    #[test]
    fn 默认中国大陆_cidr_准入列表不是空模板() {
        let cidrs: Vec<&str> = default_mainland_cidr_file()
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#')
            })
            .collect();

        assert!(cidrs.len() > 1_000);
        assert!(cidrs.contains(&"1.0.1.0/24"));
        assert!(cidrs.iter().all(|cidr| cidr.contains('/')));
    }

    #[test]
    fn 渲染_ip_白名单会展开中国大陆_cidr() {
        let rendered = render_ip_whitelist_cidr_file(&[WebIpWhitelistEntry::new(
            WEB_IP_WHITELIST_CHINA_MAINLAND,
            "默认",
            "内置中国大陆 IPv4 CIDR",
        )])
        .expect("应该能渲染中国大陆白名单");
        let cidrs: Vec<&str> = rendered
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#')
            })
            .collect();

        assert!(cidrs.len() > 1_000);
        assert!(cidrs.contains(&"1.0.1.0/24"));
        assert!(rendered.contains("持久化来源：全局设置 webIpWhitelist"));
        assert!(rendered.contains("分组：默认"));
    }

    #[test]
    fn 渲染_ip_白名单支持单_ip_和_cidr() {
        let rendered = render_ip_whitelist_cidr_file(&[
            WebIpWhitelistEntry::new("203.0.113.10", "运维", "办公室出口"),
            WebIpWhitelistEntry::new("203.0.113.0/24", "临时", "测试网段"),
            WebIpWhitelistEntry::new("203.0.113.10", "重复", "应去重"),
        ])
        .expect("应该能渲染自定义白名单");

        assert!(rendered.contains("203.0.113.10/32"));
        assert!(rendered.contains("203.0.113.0/24"));
        assert!(rendered.contains("分组：运维"));
        assert!(rendered.contains("备注：办公室出口"));
        assert_eq!(rendered.matches("203.0.113.10/32").count(), 1);
    }

    #[test]
    fn ip_白名单拒绝非法条目() {
        assert!(
            validate_ip_whitelist_entries(&[WebIpWhitelistEntry::new("example.com", "", "")])
                .is_err()
        );
        assert!(
            validate_ip_whitelist_entries(&[WebIpWhitelistEntry::new("203.0.113.0/33", "", "")])
                .is_err()
        );
        assert!(
            render_ip_whitelist_cidr_file(&[WebIpWhitelistEntry::new("203.0.113.0/test", "", "")])
                .is_err()
        );
    }

    #[test]
    fn 管理接口响应支持普通_json_和分块_json() {
        let plain =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"ok\":true,\"data\":[]}";
        let parsed = parse_admin_http_response(plain).unwrap();
        assert_eq!(parsed["ok"], true);

        let part_a = "{\"ok\":true,\"data\":{\"ip\":\"";
        let part_b = "203.0.113.10\"}}";
        let chunked = format!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{}\r\n{:x}\r\n{}\r\n0\r\n\r\n",
            part_a.len(),
            part_a,
            part_b.len(),
            part_b
        );
        let parsed = parse_admin_http_response(chunked.as_bytes()).unwrap();
        assert_eq!(parsed["data"]["ip"], "203.0.113.10");
    }

    #[test]
    fn 手动解封_ip_只接受_ipv4() {
        assert_eq!(normalize_admin_ip("203.0.113.10").unwrap(), "203.0.113.10");
        assert!(normalize_admin_ip("example.com").is_err());
        assert!(normalize_admin_ip("2001:db8::1").is_err());
    }

    #[test]
    fn 域名禁止包含协议端口和通配符() {
        assert!(normalize_domain("https://asa.example.com").is_err());
        assert!(normalize_domain("asa.example.com:18081").is_err());
        assert!(normalize_domain("*.example.com").is_err());
    }
}
