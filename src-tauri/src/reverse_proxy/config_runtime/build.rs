use std::path::Path;

use crate::{
    acme_certificate, app_state::AppRuntime, command_events::emit_instance_log,
    models::GlobalSettings, reverse_proxy_host::normalize_domain,
    reverse_proxy_ip_whitelist::normalize_ip_whitelist_entries,
    reverse_proxy_runtime::resolve_openresty_executable_path,
};
use tauri::AppHandle;

use super::super::{CERTS_RELATIVE_DIR, PROXY_ROOT_DIR_NAME, ReverseProxyConfig};

impl ReverseProxyConfig {
    pub(in crate::reverse_proxy) fn from_settings(
        app: &AppHandle,
        runtime: &AppRuntime,
        data_dir: &Path,
        settings: &GlobalSettings,
    ) -> Result<Self, String> {
        let openresty_executable_path =
            resolve_openresty_executable_path(&settings.web_reverse_proxy_openresty_path)?;
        let openresty_root_path = openresty_executable_path
            .parent()
            .ok_or_else(|| "无法识别 OpenResty nginx.exe 所在目录".to_string())?
            .to_path_buf();

        Self {
            openresty_executable_path,
            openresty_root_path,
            proxy_root_path: data_dir.join(PROXY_ROOT_DIR_NAME),
            domain: normalize_domain(&settings.web_reverse_proxy_domain)?,
            public_port: settings.web_reverse_proxy_port,
            web_port: settings.web_server_port,
            https_enabled: settings.web_https_enabled,
            certificate_paths: None,
            login_failure_ban_threshold: settings.web_login_failure_ban_threshold,
            login_failure_ban_seconds: settings.web_login_failure_ban_seconds,
            ip_whitelist: normalize_ip_whitelist_entries(&settings.web_ip_whitelist),
        }
        .with_certificate_paths(app, runtime, settings)
    }

    fn with_certificate_paths(
        mut self,
        app: &AppHandle,
        runtime: &AppRuntime,
        settings: &GlobalSettings,
    ) -> Result<Self, String> {
        if self.https_enabled {
            let log_sink = |level: &str, message: &str| {
                let _ = emit_instance_log(app, runtime, "ACME证书", level, message);
            };
            self.certificate_paths = Some(acme_certificate::ensure_certificate_files(
                &self.proxy_root_path.join(CERTS_RELATIVE_DIR),
                &self.domain,
                settings,
                Some(&log_sink),
            )?);
        }
        Ok(self)
    }
}
