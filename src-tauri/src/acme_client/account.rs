use reqwest::StatusCode;
use serde_json::json;

use super::AcmeClient;

impl AcmeClient {
    pub(crate) fn ensure_account(&mut self, email: &str) -> Result<(), String> {
        let response = self.post_json_with_jwk(
            &self.directory.new_account.clone(),
            json!({
                "termsOfServiceAgreed": true,
                "contact": [format!("mailto:{email}")]
            }),
        )?;
        if !(response.status == StatusCode::CREATED || response.status == StatusCode::OK) {
            return Err(format!(
                "ACME 创建账户失败（HTTP {}）：{}",
                response.status, response.text
            ));
        }
        let kid = response
            .location
            .ok_or_else(|| "ACME 创建账户响应缺少账户 Location".to_string())?;
        self.kid = Some(kid);
        Ok(())
    }
}
