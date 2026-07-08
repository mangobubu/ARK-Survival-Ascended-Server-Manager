use crate::{
    web_auth_state::WebAuthState,
    web_auth_utils::{auth_token_from_request, expired_auth_cookie},
    web_http::{HttpRequest, HttpResponse, json_response},
};
use serde_json::json;

use super::responses::json_error;

pub(crate) fn handle_logout(auth_state: &WebAuthState, request: &HttpRequest) -> HttpResponse {
    if let Some(token) = auth_token_from_request(request)
        && let Err(error) = auth_state.remove_session(&token)
    {
        return json_error(500, error);
    }
    let mut response = json_response(200, json!({ "ok": true }).to_string().into_bytes());
    response.header("Set-Cookie", expired_auth_cookie());
    response
}
