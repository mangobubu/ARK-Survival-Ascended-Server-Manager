use reqwest::Client;
use reqwest::blocking::Client as BlockingClient;

use super::{API_VERSION, HOST, TencentDnsHttpRequest};

pub(super) async fn send_request(
    request: &TencentDnsHttpRequest,
) -> Result<serde_json::Value, String> {
    let response = Client::new()
        .post(&request.url)
        .header("Authorization", &request.authorization)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Host", HOST)
        .header("X-TC-Action", &request.action)
        .header("X-TC-Version", API_VERSION)
        .header("X-TC-Timestamp", request.timestamp.to_string())
        .body(request.body.clone())
        .send()
        .await
        .map_err(|error| format!("请求腾讯云 DNSPod API 失败：{error}"))?;
    parse_response(response.status(), response.text().await)
}

pub(super) fn send_request_blocking(
    request: &TencentDnsHttpRequest,
) -> Result<serde_json::Value, String> {
    let response = BlockingClient::new()
        .post(&request.url)
        .header("Authorization", &request.authorization)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Host", HOST)
        .header("X-TC-Action", &request.action)
        .header("X-TC-Version", API_VERSION)
        .header("X-TC-Timestamp", request.timestamp.to_string())
        .body(request.body.clone())
        .send()
        .map_err(|error| format!("请求腾讯云 DNSPod API 失败：{error}"))?;
    parse_response(response.status(), response.text())
}

pub(super) fn create_record_id_from_payload(payload: &serde_json::Value) -> Result<u64, String> {
    payload
        .get("Response")
        .and_then(|response| response.get("RecordId"))
        .and_then(|value| value.as_u64())
        .ok_or_else(|| format!("腾讯云 DNSPod 创建 TXT 记录响应缺少 RecordId：{payload}"))
}

fn parse_response(
    status: reqwest::StatusCode,
    text_result: Result<String, reqwest::Error>,
) -> Result<serde_json::Value, String> {
    let text = text_result.map_err(|error| format!("读取腾讯云 DNSPod 响应失败：{error}"))?;
    let payload = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|error| format!("解析腾讯云 DNSPod 响应失败：{error}；原始响应：{text}"))?;
    if status.is_success()
        && payload
            .get("Response")
            .and_then(|response| response.get("Error"))
            .is_none()
    {
        return Ok(payload);
    }
    Err(format!(
        "腾讯云 DNSPod API 返回失败（HTTP {status}）：{payload}"
    ))
}
