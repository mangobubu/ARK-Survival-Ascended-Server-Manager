use crate::{
    app_state::AppRuntime,
    web_auth,
    web_auth_state::WebAuthState,
    web_auth_utils::{auth_cookie, client_identity_from_request},
    web_http::{HttpRequest, HttpResponse, json_response},
};
use serde_json::json;

use super::{
    responses::{auth_not_configured_response, json_error},
    types::LoginRequest,
};
use crate::web_server_auth::{auth_configured, auth_key};

pub(crate) async fn handle_login(
    runtime: AppRuntime,
    auth_state: WebAuthState,
    request: HttpRequest,
) -> HttpResponse {
    let payload = match serde_json::from_slice::<LoginRequest>(&request.body) {
        Ok(payload) => payload,
        Err(error) => return json_error(400, format!("登录请求 JSON 无效：{error}")),
    };

    let settings = match runtime.settings() {
        Ok(settings) => settings,
        Err(error) => return json_error(500, error),
    };

    if !auth_configured(&settings) {
        return auth_not_configured_response();
    }

    if let Err(error) = auth_state.check_login_allowed(&payload.username) {
        return json_error(429, error);
    }

    let client_identity = client_identity_from_request(&request);
    match auth_state.is_captcha_required(&client_identity) {
        Ok(true) => {
            if let Err(error) = auth_state.verify_captcha(
                &client_identity,
                payload.captcha_token.as_deref(),
                payload.captcha_answer.as_deref(),
            ) {
                return json_response(
                    400,
                    json!({
                        "ok": false,
                        "error": error,
                        "captchaRequired": true
                    })
                    .to_string()
                    .into_bytes(),
                );
            }
        }
        Ok(false) => {}
        Err(error) => return json_error(500, error),
    }

    let username_matches = payload.username.trim() == settings.web_admin_username.trim();
    let password_matches = username_matches
        && web_auth::verify_web_admin_password(&payload.password, &settings.web_admin_password)
            .unwrap_or(false);
    if !username_matches || !password_matches {
        return handle_failed_login(&auth_state, &payload.username, &client_identity);
    }

    if let Err(error) = auth_state.record_login_success(&payload.username) {
        return json_error(500, error);
    }

    match auth_state.create_session(auth_key(&settings)) {
        Ok(token) => {
            let secure_cookie = settings.web_https_enabled && settings.web_reverse_proxy_enabled;
            let mut response = json_response(
                200,
                json!({ "ok": true, "data": {} }).to_string().into_bytes(),
            );
            response.header("Set-Cookie", auth_cookie(&token, secure_cookie));
            response
        }
        Err(error) => json_error(500, error),
    }
}

fn handle_failed_login(
    auth_state: &WebAuthState,
    username: &str,
    client_identity: &str,
) -> HttpResponse {
    if let Err(error) = auth_state.record_login_failure(username) {
        return json_error(500, error);
    }
    if let Err(error) = auth_state.require_captcha(client_identity) {
        return json_error(500, error);
    }
    json_response(
        401,
        json!({
            "ok": false,
            "error": "管理员账号或密码不正确，后续一小时内登录需要输入验证码",
            "captchaRequired": true
        })
        .to_string()
        .into_bytes(),
    )
}
