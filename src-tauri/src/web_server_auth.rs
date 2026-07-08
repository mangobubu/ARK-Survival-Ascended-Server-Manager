use crate::{
    app_state::AppRuntime,
    web_auth_state::WebAuthState,
    web_auth_utils::auth_token_from_request,
    web_http::{HttpRequest, HttpResponse, json_response},
};
use serde_json::json;

mod risk;
mod session;

pub(crate) use risk::{handle_risk_confirmation, validate_high_risk_confirmation};
pub(crate) use session::{handle_auth_status, handle_captcha, handle_login, handle_logout};
pub(crate) fn require_auth(
    runtime: &AppRuntime,
    auth_state: &WebAuthState,
    request: &HttpRequest,
) -> Result<(), HttpResponse> {
    let settings = runtime.settings().map_err(|error| {
        json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        )
    })?;

    if !auth_configured(&settings) {
        return Err(json_response(
            403,
            json!({
                "ok": false,
                "error": "Web 管理员账号和密码尚未配置，请先在桌面端全局设置中部署。"
            })
            .to_string()
            .into_bytes(),
        ));
    }

    let Some(token) = auth_token_from_request(request) else {
        return Err(json_response(
            401,
            json!({ "ok": false, "error": "Web 操作需要先登录" })
                .to_string()
                .into_bytes(),
        ));
    };

    let valid = auth_state
        .has_session(&token, &auth_key(&settings))
        .map_err(|error| {
            json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            )
        })?;

    if valid {
        Ok(())
    } else {
        Err(json_response(
            401,
            json!({ "ok": false, "error": "Web 登录已失效，请重新登录" })
                .to_string()
                .into_bytes(),
        ))
    }
}

fn auth_configured(settings: &crate::models::GlobalSettings) -> bool {
    !settings.web_admin_username.trim().is_empty() && !settings.web_admin_password.is_empty()
}

fn auth_key(settings: &crate::models::GlobalSettings) -> String {
    format!(
        "{}\u{0}{}",
        settings.web_admin_username.trim(),
        settings.web_admin_password
    )
}
