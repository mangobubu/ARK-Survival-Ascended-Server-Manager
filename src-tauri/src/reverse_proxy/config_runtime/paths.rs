use std::path::PathBuf;

use super::super::{
    CONFIG_RELATIVE_PATH, IP_WHITELIST_CIDR_RELATIVE_PATH, ReverseProxyConfig,
    SECURITY_LUA_RELATIVE_PATH,
};

impl ReverseProxyConfig {
    pub(super) fn config_path(&self) -> PathBuf {
        self.proxy_root_path.join(CONFIG_RELATIVE_PATH)
    }

    pub(super) fn security_lua_path(&self) -> PathBuf {
        self.proxy_root_path.join(SECURITY_LUA_RELATIVE_PATH)
    }

    pub(super) fn ip_whitelist_cidr_path(&self) -> PathBuf {
        self.proxy_root_path.join(IP_WHITELIST_CIDR_RELATIVE_PATH)
    }

    pub(super) fn pid_path(&self) -> PathBuf {
        self.proxy_root_path
            .join("logs")
            .join("asa-web-reverse-proxy.pid")
    }
}
