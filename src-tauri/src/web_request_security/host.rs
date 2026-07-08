use crate::{
    app_state::AppRuntime,
    models::GlobalSettings,
    reverse_proxy,
    web_http::{HttpRequest, HttpResponse, json_response},
};
use serde_json::json;

pub(crate) fn require_allowed_host(
    runtime: &AppRuntime,
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

    require_allowed_host_with_settings(&settings, request)
}

pub(super) fn require_allowed_host_with_settings(
    settings: &GlobalSettings,
    request: &HttpRequest,
) -> Result<(), HttpResponse> {
    if reverse_proxy::is_request_host_allowed(
        settings,
        request.headers.get("host").map(String::as_str),
    ) {
        Ok(())
    } else {
        Err(json_response(
            403,
            json!({
                "ok": false,
                "error": "当前 Web 管理端仅允许通过已配置的域名和端口访问"
            })
            .to_string()
            .into_bytes(),
        ))
    }
}
