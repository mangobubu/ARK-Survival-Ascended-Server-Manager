use crate::web_http::{HttpResponse, json_response};
use serde_json::json;

pub(super) fn json_error(status: u16, error: impl Into<String>) -> HttpResponse {
    json_response(
        status,
        json!({ "ok": false, "error": error.into() })
            .to_string()
            .into_bytes(),
    )
}

pub(super) fn auth_not_configured_response() -> HttpResponse {
    json_response(
        403,
        json!({
            "ok": false,
            "error": "尚未配置 Web 管理员账号和密码，请先回到桌面端的全局设置中部署。"
        })
        .to_string()
        .into_bytes(),
    )
}
