use crate::acme_certificate::WebCertificatePaths;
use std::path::Path;

const RATE_LIMIT_PER_MINUTE: u32 = 180;
const API_RATE_LIMIT_PER_MINUTE: u32 = 120;
const LOGIN_RATE_LIMIT_PER_MINUTE: u32 = 10;
const PATH_RISK_BAN_THRESHOLD: u32 = 3;
const PATH_RISK_BAN_SECONDS: u32 = 60 * 60;
const BODY_RISK_BAN_SECONDS: u32 = 60 * 60;

pub(crate) struct ReverseProxyRenderInput<'a> {
    pub(crate) domain: &'a str,
    pub(crate) public_port: u16,
    pub(crate) web_port: u16,
    pub(crate) https_enabled: bool,
    pub(crate) certificate_paths: Option<&'a WebCertificatePaths>,
    pub(crate) lualib_path: &'a Path,
    pub(crate) ip_whitelist_cidr_path: &'a Path,
    pub(crate) login_failure_ban_threshold: u32,
    pub(crate) login_failure_ban_seconds: u32,
}

pub(crate) fn render_openresty_config(input: &ReverseProxyRenderInput<'_>) -> String {
    let default_listen_directive = default_listen_directive(input);
    let listen_directive = listen_directive(input);
    let ssl_config = ssl_config(input);
    let security_headers = security_headers(input);
    let lua_package_path = lua_package_path(input.lualib_path);
    let ip_whitelist_cidr_path = lua_string_literal_path(input.ip_whitelist_cidr_path);
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
    lua_package_path {lua_package_path};
    lua_shared_dict asa_ip_bans 10m;
    lua_shared_dict asa_rate_counters 20m;
    lua_shared_dict asa_login_failures 10m;
    lua_shared_dict asa_path_risks 10m;
    lua_shared_dict asa_cidr_cache 2m;

    init_by_lua_block {{
        require("asa_security").configure({{
            domain = "{domain}",
            public_port = {public_port},
            ip_whitelist_cidr_path = {ip_whitelist_cidr_path},
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
        {default_listen_directive}
        server_name _;
{ssl_config}
        return 403;
    }}

    server {{
        {listen_directive}
        server_name {domain};
{ssl_config}
{security_headers}

        access_by_lua_block {{
            require("asa_security").access()
        }}

        log_by_lua_block {{
            require("asa_security").log()
        }}

        location = /__asa_security/bans {{
            allow 127.0.0.1;
            deny all;
            access_by_lua_block {{
                ngx.ctx.asa_security_admin_endpoint = true
            }}
            content_by_lua_block {{
                require("asa_security").admin_bans()
            }}
        }}

        location = /__asa_security/unban {{
            allow 127.0.0.1;
            deny all;
            access_by_lua_block {{
                ngx.ctx.asa_security_admin_endpoint = true
            }}
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
        default_listen_directive = default_listen_directive,
        public_port = input.public_port,
        domain = input.domain,
        web_port = input.web_port,
        listen_directive = listen_directive,
        ssl_config = ssl_config,
        security_headers = security_headers,
        lua_package_path = lua_package_path,
        ip_whitelist_cidr_path = ip_whitelist_cidr_path,
        login_failure_ban_threshold = input.login_failure_ban_threshold,
        login_failure_ban_seconds = input.login_failure_ban_seconds,
        rate_limit_per_minute = RATE_LIMIT_PER_MINUTE,
        api_rate_limit_per_minute = API_RATE_LIMIT_PER_MINUTE,
        login_rate_limit_per_minute = LOGIN_RATE_LIMIT_PER_MINUTE,
        path_risk_ban_threshold = PATH_RISK_BAN_THRESHOLD,
        path_risk_ban_seconds = PATH_RISK_BAN_SECONDS,
        body_risk_ban_seconds = BODY_RISK_BAN_SECONDS,
    )
}

fn lua_package_path(lualib_path: &Path) -> String {
    let mut normalized_path = lualib_path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('"', "\\\"");
    if !normalized_path.ends_with('/') {
        normalized_path.push('/');
    }
    format!("\"{normalized_path}?.lua;;\"")
}

fn lua_string_literal_path(path: &Path) -> String {
    lua_string_literal(&path.to_string_lossy().replace('\\', "/"))
}

fn lua_string_literal(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn default_listen_directive(input: &ReverseProxyRenderInput<'_>) -> String {
    if input.https_enabled {
        format!("listen 0.0.0.0:{} ssl default_server;", input.public_port)
    } else {
        format!("listen 0.0.0.0:{} default_server;", input.public_port)
    }
}

fn listen_directive(input: &ReverseProxyRenderInput<'_>) -> String {
    if input.https_enabled {
        format!("listen 0.0.0.0:{} ssl;", input.public_port)
    } else {
        format!("listen 0.0.0.0:{};", input.public_port)
    }
}

fn ssl_config(input: &ReverseProxyRenderInput<'_>) -> String {
    if !input.https_enabled {
        return String::new();
    }
    let Some(paths) = input.certificate_paths else {
        return String::new();
    };
    let cert_path = nginx_quoted_absolute_path(&paths.fullchain_pem);
    let key_path = nginx_quoted_absolute_path(&paths.private_key_pem);
    format!(
        r#"
        ssl_certificate {cert_path};
        ssl_certificate_key {key_path};
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_prefer_server_ciphers off;
        ssl_session_cache shared:ASAWebSSL:10m;
        ssl_session_timeout 1d;"#
    )
}

fn security_headers(input: &ReverseProxyRenderInput<'_>) -> String {
    let hsts = if input.https_enabled {
        r#"
        add_header Strict-Transport-Security "max-age=15552000; includeSubDomains" always;"#
    } else {
        ""
    };
    format!(
        r#"{hsts}
        add_header X-Frame-Options "DENY" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header Referrer-Policy "no-referrer" always;
        add_header Permissions-Policy "camera=(), microphone=(), geolocation=(), payment=()" always;"#
    )
}

fn nginx_quoted_absolute_path(path: &Path) -> String {
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('"', "\\\"");
    format!("\"{normalized_path}\"")
}
