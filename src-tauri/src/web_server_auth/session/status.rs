use crate::{
    app_state::AppRuntime,
    web_auth_state::WebAuthState,
    web_auth_utils::client_identity_from_request,
    web_http::{HttpRequest, HttpResponse, json_response},
};
use serde_json::json;

use super::responses::json_error;
use crate::web_server_auth::auth_configured;

pub(crate) fn handle_auth_status(
    runtime: AppRuntime,
    auth_state: &WebAuthState,
    request: &HttpRequest,
) -> HttpResponse {
    match runtime.settings() {
        Ok(settings) => {
            let client_identity = client_identity_from_request(request);
            let captcha_required = if auth_configured(&settings) {
                match auth_state.is_captcha_required(&client_identity) {
                    Ok(required) => required,
                    Err(error) => return json_error(500, error),
                }
            } else {
                false
            };
            json_response(
                200,
                json!({
                    "ok": true,
                    "data": {
                        "configured": auth_configured(&settings),
                        "captchaRequired": captcha_required
                    }
                })
                .to_string()
                .into_bytes(),
            )
        }
        Err(error) => json_error(500, error),
    }
}
