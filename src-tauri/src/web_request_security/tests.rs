use super::{host::require_allowed_host_with_settings, same_origin::validate_same_origin_api_post};
use crate::{models::GlobalSettings, web_http::HttpRequest};
use std::{collections::HashMap, net::SocketAddr};

fn proxy_settings() -> GlobalSettings {
    GlobalSettings {
        web_management_enabled: true,
        web_reverse_proxy_enabled: true,
        web_reverse_proxy_domain: "asa.example.com".to_string(),
        web_reverse_proxy_port: 18081,
        web_server_port: 18080,
        ..GlobalSettings::default()
    }
}

fn post_api_request(host: &str) -> HttpRequest {
    let mut headers = HashMap::new();
    headers.insert("host".to_string(), host.to_string());
    HttpRequest {
        method: "POST".to_string(),
        path: "/api/invoke".to_string(),
        headers,
        body: Vec::new(),
        client_addr: SocketAddr::from(([127, 0, 0, 1], 18080)),
    }
}

#[test]
fn 反代域名_host_通过_web_请求安全层允许() {
    let settings = proxy_settings();
    let request = post_api_request("ASA.EXAMPLE.COM:18081");

    assert!(require_allowed_host_with_settings(&settings, &request).is_ok());
}

#[test]
fn 错误_host_会被_web_请求安全层拒绝() {
    let settings = proxy_settings();
    let request = post_api_request("127.0.0.1:18080");

    assert!(require_allowed_host_with_settings(&settings, &request).is_err());
}

#[test]
fn 未启用反代或_web_管理时_web_请求安全层不限制_host() {
    let mut settings = proxy_settings();
    let request = post_api_request("127.0.0.1:18080");

    settings.web_reverse_proxy_enabled = false;
    assert!(require_allowed_host_with_settings(&settings, &request).is_ok());

    settings.web_reverse_proxy_enabled = true;
    settings.web_management_enabled = false;
    assert!(require_allowed_host_with_settings(&settings, &request).is_ok());
}

#[test]
fn 同源_origin_允许_web_api_post() {
    let mut request = post_api_request("asa.example.com:18081");
    request.headers.insert(
        "origin".to_string(),
        "https://asa.example.com:18081".to_string(),
    );

    assert!(validate_same_origin_api_post(&request).is_ok());
}

#[test]
fn 跨站_origin_拒绝_web_api_post() {
    let mut request = post_api_request("asa.example.com:18081");
    request.headers.insert(
        "origin".to_string(),
        "https://evil.example.net:18081".to_string(),
    );

    assert!(validate_same_origin_api_post(&request).is_err());
}

#[test]
fn 缺少_origin_时只接受_same_origin_fetch_metadata() {
    let mut request = post_api_request("127.0.0.1:18080");
    request
        .headers
        .insert("sec-fetch-site".to_string(), "same-origin".to_string());
    assert!(validate_same_origin_api_post(&request).is_ok());

    request
        .headers
        .insert("sec-fetch-site".to_string(), "cross-site".to_string());
    assert!(validate_same_origin_api_post(&request).is_err());
}

#[test]
fn 缺少同源凭证的_web_api_post_会被拒绝() {
    let request = post_api_request("127.0.0.1:18080");
    assert!(validate_same_origin_api_post(&request).is_err());
}

#[test]
fn 同源校验会拒绝畸形_authority() {
    for host in [
        "",
        "asa.example.com/path",
        "user@asa.example.com",
        "asa example.com",
    ] {
        let request = post_api_request(host);
        assert!(
            validate_same_origin_api_post(&request).is_err(),
            "{host:?} 应被判定为无效 Host"
        );
    }
}
