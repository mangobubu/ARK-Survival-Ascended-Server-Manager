use super::*;
use crate::{
    web_auth_utils::{auth_cookie, auth_token_from_request},
    web_http::HttpRequest,
};
use std::net::SocketAddr;

fn request(path: &str, authorization: Option<&str>) -> HttpRequest {
    let mut headers = HashMap::new();
    if let Some(authorization) = authorization {
        headers.insert("authorization".to_string(), authorization.to_string());
    }
    HttpRequest {
        method: "GET".to_string(),
        path: path.to_string(),
        headers,
        body: Vec::new(),
        client_addr: SocketAddr::from(([127, 0, 0, 1], 18080)),
    }
}

fn cookie_request(path: &str, cookie: &str) -> HttpRequest {
    let mut request = request(path, None);
    request
        .headers
        .insert("cookie".to_string(), cookie.to_string());
    request
}

#[test]
fn query_token_never_becomes_auth_token() {
    let request = request("/api/invoke?token=query-token", None);
    assert_eq!(auth_token_from_request(&request), None);
}

#[test]
fn authorization_header_never_becomes_auth_token() {
    let request = request("/api/events?token=query-token", Some("Bearer header-token"));
    assert_eq!(auth_token_from_request(&request), None);
}

#[test]
fn http_only_cookie_token_is_accepted() {
    let request = cookie_request(
        "/api/events?token=query-token",
        "other=value; asa-web-auth-token=cookie-token",
    );
    assert_eq!(
        auth_token_from_request(&request).as_deref(),
        Some("cookie-token")
    );
}

#[test]
fn web_auth_cookie_包含总有效期且保持_http_only() {
    let cookie = auth_cookie("session-token", true);

    assert!(cookie.contains("Max-Age=28800"));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Secure"));
}

#[test]
fn web_会话空闲超时后自动失效并清理高风险确认令牌() {
    let auth_state = WebAuthState::default();
    let session_token = auth_state
        .create_session("auth-key".to_string())
        .expect("创建 Web 会话");
    let confirmation_token = auth_state
        .create_risk_confirmation(&session_token, "auth-key", "delete_instance")
        .expect("创建高风险确认令牌");
    std::thread::sleep(WEB_SESSION_IDLE_TIMEOUT + Duration::from_millis(25));

    assert!(
        !auth_state
            .has_session(&session_token, "auth-key")
            .expect("校验过期会话")
    );
    assert!(
        auth_state
            .consume_risk_confirmation(
                &confirmation_token,
                &session_token,
                "auth-key",
                "delete_instance",
            )
            .is_err()
    );
}

#[test]
fn web_管理员密码变更后旧会话和确认令牌失效() {
    let auth_state = WebAuthState::default();
    let session_token = auth_state
        .create_session("old-auth-key".to_string())
        .expect("创建旧 Web 会话");
    let confirmation_token = auth_state
        .create_risk_confirmation(&session_token, "old-auth-key", "delete_instance")
        .expect("创建旧高风险确认令牌");

    assert!(
        !auth_state
            .has_session(&session_token, "new-auth-key")
            .expect("使用新认证键校验旧会话")
    );
    assert!(
        auth_state
            .consume_risk_confirmation(
                &confirmation_token,
                &session_token,
                "old-auth-key",
                "delete_instance",
            )
            .is_err()
    );
}

#[test]
fn 高风险确认令牌绑定会话和命令且只能使用一次() {
    let auth_state = WebAuthState::default();
    let token = auth_state
        .create_risk_confirmation("session-a", "auth-key", "delete_instance")
        .expect("创建高风险确认令牌");

    assert!(
        auth_state
            .consume_risk_confirmation(&token, "session-b", "auth-key", "delete_instance")
            .is_err()
    );

    let token = auth_state
        .create_risk_confirmation("session-a", "auth-key", "delete_instance")
        .expect("重新创建高风险确认令牌");
    assert!(
        auth_state
            .consume_risk_confirmation(&token, "session-a", "auth-key", "save_settings")
            .is_err()
    );

    let token = auth_state
        .create_risk_confirmation("session-a", "auth-key", "delete_instance")
        .expect("再次创建高风险确认令牌");
    assert!(
        auth_state
            .consume_risk_confirmation(&token, "session-a", "auth-key", "delete_instance")
            .is_ok()
    );
    assert!(
        auth_state
            .consume_risk_confirmation(&token, "session-a", "auth-key", "delete_instance")
            .is_err()
    );
}

#[test]
fn login_failures_lock_account_temporarily_and_success_resets_counter() {
    let auth_state = WebAuthState::default();
    for _ in 0..LOGIN_MAX_FAILED_ATTEMPTS {
        auth_state
            .record_login_failure("Admin")
            .expect("记录登录失败");
    }

    assert!(auth_state.check_login_allowed("admin").is_err());

    auth_state
        .record_login_success("ADMIN")
        .expect("登录成功后清理失败计数");
    assert!(auth_state.check_login_allowed("admin").is_ok());
}

#[test]
fn first_failed_login_requires_captcha_for_client_identity() {
    let auth_state = WebAuthState::default();
    let client_identity = "203.0.113.10";

    assert!(
        !auth_state
            .is_captcha_required(client_identity)
            .expect("读取初始验证码状态")
    );
    auth_state
        .require_captcha(client_identity)
        .expect("标记需要验证码");

    assert!(
        auth_state
            .is_captcha_required(client_identity)
            .expect("读取验证码状态")
    );
}

#[test]
fn captcha_challenge_can_only_be_used_once() {
    let auth_state = WebAuthState::default();
    let settings = GlobalSettings::default();
    let client_identity = "127.0.0.1";
    let challenge = auth_state
        .create_captcha_challenge(client_identity, &settings)
        .expect("创建验证码");
    let answer = {
        let challenges = auth_state
            .captcha_challenges
            .lock()
            .expect("读取验证码题库");
        challenges
            .get(&challenge.token)
            .expect("验证码存在")
            .answer
            .clone()
    };

    assert!(
        auth_state
            .verify_captcha(client_identity, Some(&challenge.token), Some(&answer))
            .is_ok()
    );
    assert!(
        auth_state
            .verify_captcha(client_identity, Some(&challenge.token), Some(&answer))
            .is_err()
    );
}
