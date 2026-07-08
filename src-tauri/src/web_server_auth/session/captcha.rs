use crate::{
    app_state::AppRuntime,
    web_auth_state::WebAuthState,
    web_auth_utils::client_identity_from_request,
    web_http::{HttpRequest, HttpResponse, json_response},
};
use serde_json::json;

use super::responses::{auth_not_configured_response, json_error};
use crate::web_server_auth::auth_configured;

pub(crate) fn handle_captcha(
    runtime: AppRuntime,
    auth_state: &WebAuthState,
    request: &HttpRequest,
) -> HttpResponse {
    let settings = match runtime.settings() {
        Ok(settings) => settings,
        Err(error) => return json_error(500, error),
    };

    if !auth_configured(&settings) {
        return auth_not_configured_response();
    }

    let client_identity = client_identity_from_request(request);
    let required = match auth_state.is_captcha_required(&client_identity) {
        Ok(required) => required,
        Err(error) => return json_error(500, error),
    };
    if !required {
        return json_response(
            200,
            json!({
                "ok": true,
                "data": {
                    "required": false,
                    "token": "",
                    "imageSvg": "",
                    "expiresInSeconds": 0
                }
            })
            .to_string()
            .into_bytes(),
        );
    }

    match auth_state.create_captcha_challenge(&client_identity, &settings) {
        Ok(challenge) => json_response(
            200,
            json!({
                "ok": true,
                "data": {
                    "required": true,
                    "token": challenge.token,
                    "imageSvg": challenge.image_svg,
                    "expiresInSeconds": challenge.expires_in_seconds
                }
            })
            .to_string()
            .into_bytes(),
        ),
        Err(error) => json_error(500, error),
    }
}
