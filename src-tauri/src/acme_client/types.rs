use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AcmeDirectory {
    pub(super) new_nonce: String,
    pub(super) new_account: String,
    pub(super) new_order: String,
}

#[derive(Debug)]
pub(super) struct AcmeHttpResponse {
    pub(super) status: StatusCode,
    pub(super) location: Option<String>,
    pub(super) body: Value,
    pub(super) text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AcmeOrder {
    pub(crate) status: String,
    pub(crate) authorizations: Vec<String>,
    pub(crate) finalize: String,
    #[serde(default)]
    pub(crate) certificate: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AcmeAuthorization {
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) challenges: Vec<AcmeChallenge>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AcmeChallenge {
    #[serde(rename = "type")]
    pub(crate) challenge_type: String,
    pub(crate) url: String,
    pub(crate) token: String,
    #[serde(default)]
    pub(crate) status: Option<String>,
}
