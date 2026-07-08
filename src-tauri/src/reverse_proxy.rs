mod config_runtime;

use crate::{
    acme_certificate::{self, WebAcmeCertificateStatus, WebCertificatePaths},
    app_state::AppRuntime,
    models::{GlobalSettings, WebIpWhitelistEntry, WebSecurityBanRecord, WebSecurityUnbanResult},
    reverse_proxy_admin::normalize_admin_ip,
    reverse_proxy_host::{self, normalize_domain},
    reverse_proxy_ip_whitelist::validate_security_settings,
    reverse_proxy_runtime::resolve_openresty_executable_path,
};
use std::{net::IpAddr, path::PathBuf, sync::Mutex, time::Duration};
use tauri::{AppHandle, Manager};

const CONFIG_RELATIVE_PATH: &str = "conf/asa-web-openresty.conf";
const IP_WHITELIST_CIDR_RELATIVE_PATH: &str = "conf/asa-ip-whitelist-cidrs.txt";
const SECURITY_LUA_RELATIVE_PATH: &str = "lualib/asa_security.lua";
const CERTS_RELATIVE_DIR: &str = "certs";
const PROXY_ROOT_DIR_NAME: &str = "web-reverse-proxy";
const STARTUP_PID_WAIT_STEP: Duration = Duration::from_millis(100);
const STARTUP_PID_WAIT_ATTEMPTS: usize = 20;
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
    https_enabled: bool,
    certificate_paths: Option<WebCertificatePaths>,
    login_failure_ban_threshold: u32,
    login_failure_ban_seconds: u32,
    ip_whitelist: Vec<WebIpWhitelistEntry>,
}

impl ReverseProxyManager {
    pub fn apply_settings(
        &self,
        app: &AppHandle,
        runtime: &AppRuntime,
        settings: &GlobalSettings,
    ) -> Result<(), String> {
        if !settings.web_management_enabled || !settings.web_reverse_proxy_enabled {
            self.stop_current_best_effort();
            return Ok(());
        }

        validate_settings(settings)?;
        let desired =
            ReverseProxyConfig::from_settings(app, runtime, &runtime.data_dir(), settings)?;
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
    manager.apply_settings(app, &runtime, settings)
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

pub fn read_acme_certificate_status(
    data_dir: PathBuf,
    settings: &GlobalSettings,
) -> Result<Option<WebAcmeCertificateStatus>, String> {
    if settings.web_reverse_proxy_domain.trim().is_empty() {
        return Ok(None);
    }
    let domain = normalize_domain(&settings.web_reverse_proxy_domain)?;
    let cert_dir = data_dir.join(PROXY_ROOT_DIR_NAME).join(CERTS_RELATIVE_DIR);
    acme_certificate::read_web_certificate_status(&cert_dir, &domain)
}

pub fn is_request_host_allowed(settings: &GlobalSettings, request_host: Option<&str>) -> bool {
    reverse_proxy_host::is_request_host_allowed(settings, request_host)
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
    if settings.web_https_enabled && settings.web_acme_auto_issue_enabled {
        if settings.web_acme_directory_url.trim() != acme_certificate::LETS_ENCRYPT_DIRECTORY_URL {
            return Err("当前仅支持 Let's Encrypt 正式环境 ACME v2 目录地址".to_string());
        }
        if settings.web_acme_account_email.trim().is_empty()
            || !settings.web_acme_account_email.contains('@')
        {
            return Err("启用 ACME 自动申请时必须填写有效邮箱".to_string());
        }
        if settings.web_acme_tencent_secret_id.trim().is_empty()
            || settings.web_acme_tencent_secret_key.trim().is_empty()
        {
            return Err("启用 ACME 自动申请时必须填写腾讯云 Secret ID 和 Secret Key".to_string());
        }
    }

    let _ = resolve_openresty_executable_path(&settings.web_reverse_proxy_openresty_path)?;
    Ok(())
}

#[cfg(test)]
mod tests;
