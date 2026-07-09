use crate::{
    reverse_proxy_config::{ReverseProxyRenderInput, render_openresty_config},
    reverse_proxy_security_gateway,
};

use super::super::ReverseProxyConfig;

impl ReverseProxyConfig {
    pub(in crate::reverse_proxy) fn render_config(&self) -> String {
        let lualib_path = self.lualib_path();
        let ip_whitelist_cidr_path = self.ip_whitelist_cidr_path();
        render_openresty_config(&ReverseProxyRenderInput {
            domain: &self.domain,
            public_port: self.public_port,
            web_port: self.web_port,
            https_enabled: self.https_enabled,
            certificate_paths: self.certificate_paths.as_ref(),
            lualib_path: &lualib_path,
            ip_whitelist_cidr_path: &ip_whitelist_cidr_path,
            login_failure_ban_threshold: self.login_failure_ban_threshold,
            login_failure_ban_seconds: self.login_failure_ban_seconds,
        })
    }

    pub(super) fn render_security_lua(&self) -> String {
        reverse_proxy_security_gateway::render_security_lua()
    }
}
