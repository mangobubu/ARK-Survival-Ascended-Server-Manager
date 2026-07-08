mod account;
mod authorization;
mod http;
mod order;
mod types;

pub(crate) use types::{AcmeAuthorization, AcmeOrder};

use crate::acme_crypto::{jwk_thumbprint, rsa_public_jwk};
use reqwest::blocking::Client;
use rsa::RsaPrivateKey;
use serde_json::Value;
use std::time::Duration;
use types::AcmeDirectory;

const ACME_HTTP_TIMEOUT_SECONDS: u64 = 60;
const ACME_POLL_INTERVAL_SECONDS: u64 = 5;
const ACME_POLL_ATTEMPTS: usize = 18;

#[derive(Debug)]
pub(crate) struct AcmeClient {
    http: Client,
    directory: AcmeDirectory,
    account_key: RsaPrivateKey,
    account_jwk: Value,
    pub(crate) account_thumbprint: String,
    nonce: String,
    kid: Option<String>,
}

impl AcmeClient {
    pub(crate) fn connect(directory_url: &str, account_key: RsaPrivateKey) -> Result<Self, String> {
        let http = Client::builder()
            .timeout(Duration::from_secs(ACME_HTTP_TIMEOUT_SECONDS))
            .user_agent("ASA-Server-Manager/0.1 ACME")
            .build()
            .map_err(|error| format!("创建 ACME HTTP 客户端失败：{error}"))?;
        let directory_text = http
            .get(directory_url)
            .send()
            .map_err(|error| format!("读取 ACME Directory 失败：{error}"))?
            .error_for_status()
            .map_err(|error| format!("ACME Directory 返回错误：{error}"))?
            .text()
            .map_err(|error| format!("读取 ACME Directory 响应失败：{error}"))?;
        let directory = serde_json::from_str::<AcmeDirectory>(&directory_text)
            .map_err(|error| format!("解析 ACME Directory 失败：{error}"))?;
        let nonce = http::fetch_nonce(&http, &directory.new_nonce)?;
        let account_jwk = rsa_public_jwk(&account_key);
        let account_thumbprint = jwk_thumbprint(&account_jwk)?;
        Ok(Self {
            http,
            directory,
            account_key,
            account_jwk,
            account_thumbprint,
            nonce,
            kid: None,
        })
    }
}
