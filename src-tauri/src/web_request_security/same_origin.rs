use crate::web_http::{HttpRequest, HttpResponse, json_response};
use serde_json::json;

use super::authority::{authority_from_absolute_url, normalized_authority};

pub(crate) fn require_same_origin_api_post(request: &HttpRequest) -> Result<(), HttpResponse> {
    validate_same_origin_api_post(request).map_err(|error| {
        json_response(
            403,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        )
    })
}

pub(super) fn validate_same_origin_api_post(request: &HttpRequest) -> Result<(), String> {
    let host = normalized_authority(
        request
            .headers
            .get("host")
            .ok_or_else(|| "Web API POST 请求缺少 Host 头，已拒绝".to_string())?,
    )
    .ok_or_else(|| "Web API POST 请求 Host 头无效，已拒绝".to_string())?;

    if let Some(origin) = request.headers.get("origin") {
        let origin_authority = authority_from_absolute_url(origin)
            .ok_or_else(|| "Web API POST 请求 Origin 头无效，已拒绝".to_string())?;
        if origin_authority == host {
            return Ok(());
        }
        return Err("Web API POST 请求 Origin 与当前访问域名不一致，已拒绝".to_string());
    }

    if let Some(referer) = request.headers.get("referer") {
        let referer_authority = authority_from_absolute_url(referer)
            .ok_or_else(|| "Web API POST 请求 Referer 头无效，已拒绝".to_string())?;
        if referer_authority == host {
            return Ok(());
        }
        return Err("Web API POST 请求 Referer 与当前访问域名不一致，已拒绝".to_string());
    }

    if let Some(fetch_site) = request.headers.get("sec-fetch-site") {
        let fetch_site = fetch_site.trim().to_ascii_lowercase();
        if fetch_site == "same-origin" || fetch_site == "none" {
            return Ok(());
        }
        return Err("Web API POST 请求不是同源浏览器请求，已拒绝".to_string());
    }

    Err("Web API POST 请求缺少 Origin/Referer 同源凭证，已拒绝".to_string())
}
