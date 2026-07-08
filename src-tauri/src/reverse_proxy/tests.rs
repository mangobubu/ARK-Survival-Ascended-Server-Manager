use super::*;
use crate::models::{WEB_IP_WHITELIST_CHINA_MAINLAND, WebIpWhitelistEntry};
use std::path::PathBuf;

fn proxy_settings() -> GlobalSettings {
    GlobalSettings {
        web_management_enabled: true,
        web_reverse_proxy_enabled: true,
        web_reverse_proxy_domain: "asa.example.com".to_string(),
        web_reverse_proxy_port: 18081,
        web_server_port: 18080,
        web_reverse_proxy_openresty_path: r"C:\openresty\nginx.exe".to_string(),
        ..GlobalSettings::default()
    }
}

fn proxy_config() -> ReverseProxyConfig {
    ReverseProxyConfig {
        openresty_executable_path: PathBuf::from(r"C:\openresty\nginx.exe"),
        openresty_root_path: PathBuf::from(r"C:\openresty"),
        proxy_root_path: PathBuf::from(r"D:\ASA\proxy"),
        domain: "asa.example.com".to_string(),
        public_port: 18081,
        web_port: 18080,
        https_enabled: false,
        certificate_paths: None,
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
    assert!(rendered.contains("ngx.ctx.asa_security_admin_endpoint = true"));
    assert!(!rendered.contains("by_lua_block {}"));
    assert!(rendered.contains("login_failure_ban_threshold = 7"));
    assert!(rendered.contains("login_failure_ban_seconds = 900"));
    assert!(rendered.contains("ip_whitelist_cidr_path = \"conf/asa-ip-whitelist-cidrs.txt\""));
    assert!(rendered.contains("proxy_pass http://127.0.0.1:18080;"));
    assert!(rendered.contains("proxy_set_header Host $host:$server_port;"));
}

#[test]
fn 域名禁止包含协议端口和通配符() {
    assert!(normalize_domain("https://asa.example.com").is_err());
    assert!(normalize_domain("asa.example.com:18081").is_err());
    assert!(normalize_domain("*.example.com").is_err());
}
