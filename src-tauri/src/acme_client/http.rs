use crate::acme_crypto::{base64_url, sign_rs256};
use reqwest::{
    blocking::{Client, Response},
    header::{HeaderMap, LOCATION},
};
use serde_json::{Map, Value, json};

use super::{AcmeClient, types::AcmeHttpResponse};
impl AcmeClient {
    pub(super) fn post_as_get_json(&mut self, url: &str) -> Result<AcmeHttpResponse, String> {
        self.post_jws(url, None)
    }

    pub(crate) fn post_as_get_text(&mut self, url: &str) -> Result<String, String> {
        let response = self.post_jws(url, None)?;
        if response.status.is_success() {
            Ok(response.text)
        } else {
            Err(format!(
                "ACME 下载证书失败（HTTP {}）：{}",
                response.status, response.text
            ))
        }
    }

    pub(super) fn post_json_with_jwk(
        &mut self,
        url: &str,
        payload: Value,
    ) -> Result<AcmeHttpResponse, String> {
        self.post_jws_with_protected_key(url, Some(payload), ProtectedKey::Jwk)
    }

    pub(super) fn post_json_with_kid(
        &mut self,
        url: &str,
        payload: Value,
    ) -> Result<AcmeHttpResponse, String> {
        self.post_jws(url, Some(payload))
    }

    fn post_jws(&mut self, url: &str, payload: Option<Value>) -> Result<AcmeHttpResponse, String> {
        self.post_jws_with_protected_key(url, payload, ProtectedKey::Kid)
    }

    fn post_jws_with_protected_key(
        &mut self,
        url: &str,
        payload: Option<Value>,
        key_mode: ProtectedKey,
    ) -> Result<AcmeHttpResponse, String> {
        let body = self.signed_jws(url, payload, key_mode)?;
        let response = self
            .http
            .post(url)
            .header("Content-Type", "application/jose+json")
            .body(body)
            .send()
            .map_err(|error| format!("请求 ACME 接口失败 {url}：{error}"))?;
        self.capture_response(response)
    }

    fn signed_jws(
        &mut self,
        url: &str,
        payload: Option<Value>,
        key_mode: ProtectedKey,
    ) -> Result<String, String> {
        let mut protected = Map::new();
        protected.insert("alg".to_string(), json!("RS256"));
        protected.insert("nonce".to_string(), json!(self.nonce.clone()));
        protected.insert("url".to_string(), json!(url));
        match key_mode {
            ProtectedKey::Jwk => {
                protected.insert("jwk".to_string(), self.account_jwk.clone());
            }
            ProtectedKey::Kid => {
                let kid = self
                    .kid
                    .clone()
                    .ok_or_else(|| "ACME 账户尚未创建，无法使用 kid 签名".to_string())?;
                protected.insert("kid".to_string(), json!(kid));
            }
        }

        let protected64 = base64_url(
            serde_json::to_string(&Value::Object(protected))
                .map_err(|error| format!("序列化 ACME JWS protected 失败：{error}"))?
                .as_bytes(),
        );
        let payload64 = match payload {
            Some(payload) => base64_url(
                serde_json::to_string(&payload)
                    .map_err(|error| format!("序列化 ACME JWS payload 失败：{error}"))?
                    .as_bytes(),
            ),
            None => String::new(),
        };
        let signing_input = format!("{protected64}.{payload64}");
        let signature = sign_rs256(&self.account_key, signing_input.as_bytes())?;
        Ok(json!({
            "protected": protected64,
            "payload": payload64,
            "signature": base64_url(&signature),
        })
        .to_string())
    }

    fn capture_response(&mut self, response: Response) -> Result<AcmeHttpResponse, String> {
        let status = response.status();
        let headers = response.headers().clone();
        self.nonce = replay_nonce(&headers)
            .or_else(|| fetch_nonce(&self.http, &self.directory.new_nonce).ok())
            .ok_or_else(|| "ACME 响应缺少 Replay-Nonce，且刷新 Nonce 失败".to_string())?;
        let location = header_string(&headers, LOCATION.as_str());
        let text = response
            .text()
            .map_err(|error| format!("读取 ACME 响应失败：{error}"))?;
        let body = if text.trim().is_empty() || !looks_like_json(&text) {
            Value::Null
        } else {
            serde_json::from_str::<Value>(&text)
                .map_err(|error| format!("解析 ACME JSON 响应失败：{error}；原始响应：{text}"))?
        };
        Ok(AcmeHttpResponse {
            status,
            location,
            body,
            text,
        })
    }
}

#[derive(Clone, Copy)]
enum ProtectedKey {
    Jwk,
    Kid,
}

pub(super) fn fetch_nonce(http: &Client, nonce_url: &str) -> Result<String, String> {
    let response = http
        .head(nonce_url)
        .send()
        .map_err(|error| format!("获取 ACME Nonce 失败：{error}"))?;
    replay_nonce(response.headers()).ok_or_else(|| "ACME Nonce 响应缺少 Replay-Nonce".to_string())
}

fn replay_nonce(headers: &HeaderMap) -> Option<String> {
    header_string(headers, "replay-nonce")
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn looks_like_json(value: &str) -> bool {
    value
        .trim_start()
        .chars()
        .next()
        .is_some_and(|ch| ch == '{' || ch == '[')
}
