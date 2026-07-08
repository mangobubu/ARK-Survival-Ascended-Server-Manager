use crate::{
    app_state::AppRuntime,
    web_auth_state::{WebAuthState, risk_confirmation_expires_in_seconds},
    web_auth_utils::auth_token_from_request,
    web_command_security,
    web_http::{HttpRequest, HttpResponse, json_response},
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::auth_key;
#[derive(Deserialize)]
struct RiskConfirmationRequest {
    command: String,
}

pub(crate) fn handle_risk_confirmation(
    runtime: AppRuntime,
    auth_state: &WebAuthState,
    request: HttpRequest,
) -> HttpResponse {
    let payload = match serde_json::from_slice::<RiskConfirmationRequest>(&request.body) {
        Ok(payload) => payload,
        Err(error) => {
            return json_response(
                400,
                json!({ "ok": false, "error": format!("Web 高风险确认请求 JSON 无效：{error}") })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    let command = payload.command.trim();
    if command.is_empty() {
        return json_response(
            400,
            json!({ "ok": false, "error": "Web 高风险确认命令不能为空" })
                .to_string()
                .into_bytes(),
        );
    }

    match web_command_security::web_command_policy(command) {
        Ok(web_command_security::WebCommandRisk::High) => {}
        Ok(_) => {
            return json_response(
                400,
                json!({ "ok": false, "error": format!("Web 命令 {command} 不需要高风险确认") })
                    .to_string()
                    .into_bytes(),
            );
        }
        Err(error) => {
            return json_response(
                400,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
    }

    let settings = match runtime.settings() {
        Ok(settings) => settings,
        Err(error) => {
            return json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
    };
    let Some(session_token) = auth_token_from_request(&request) else {
        return json_response(
            401,
            json!({ "ok": false, "error": "Web 高风险确认需要有效登录会话" })
                .to_string()
                .into_bytes(),
        );
    };
    let auth_key = auth_key(&settings);

    match auth_state.create_risk_confirmation(&session_token, &auth_key, command) {
        Ok(token) => json_response(
            200,
            json!({
                "ok": true,
                "data": {
                    "token": token,
                    "command": command,
                    "expiresInSeconds": risk_confirmation_expires_in_seconds()
                }
            })
            .to_string()
            .into_bytes(),
        ),
        Err(error) => json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    }
}

pub(crate) fn validate_high_risk_confirmation(
    runtime: &AppRuntime,
    auth_state: &WebAuthState,
    request: &HttpRequest,
    command: &str,
    args: &Value,
) -> Result<(), HttpResponse> {
    let confirmation_token = args
        .get("riskConfirmationToken")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            json_response(
                403,
                json!({ "ok": false, "error": "Web 高风险命令需要先完成服务端确认" })
                    .to_string()
                    .into_bytes(),
            )
        })?;
    let settings = runtime.settings().map_err(|error| {
        json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        )
    })?;
    let session_token = auth_token_from_request(request).ok_or_else(|| {
        json_response(
            401,
            json!({ "ok": false, "error": "Web 操作需要先登录" })
                .to_string()
                .into_bytes(),
        )
    })?;
    auth_state
        .consume_risk_confirmation(
            confirmation_token,
            &session_token,
            &auth_key(&settings),
            command,
        )
        .map_err(|error| {
            json_response(
                403,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            )
        })
}
