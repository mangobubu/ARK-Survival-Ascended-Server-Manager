use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LoginRequest {
    pub(super) username: String,
    pub(super) password: String,
    #[serde(default)]
    pub(super) captcha_token: Option<String>,
    #[serde(default)]
    pub(super) captcha_answer: Option<String>,
}
