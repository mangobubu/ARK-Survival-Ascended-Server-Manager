use std::{thread, time::Duration};

use serde_json::json;

use super::{ACME_POLL_ATTEMPTS, ACME_POLL_INTERVAL_SECONDS, AcmeAuthorization, AcmeClient};

impl AcmeClient {
    pub(crate) fn get_authorization(
        &mut self,
        authorization_url: &str,
    ) -> Result<AcmeAuthorization, String> {
        let response = self.post_as_get_json(authorization_url)?;
        if !response.status.is_success() {
            return Err(format!(
                "ACME 读取授权失败（HTTP {}）：{}",
                response.status, response.text
            ));
        }
        serde_json::from_value::<AcmeAuthorization>(response.body)
            .map_err(|error| format!("解析 ACME 授权失败：{error}"))
    }

    pub(crate) fn accept_challenge(&mut self, challenge_url: &str) -> Result<(), String> {
        let response = self.post_json_with_kid(challenge_url, json!({}))?;
        if response.status.is_success() {
            Ok(())
        } else {
            Err(format!(
                "ACME 提交 dns-01 Challenge 失败（HTTP {}）：{}",
                response.status, response.text
            ))
        }
    }

    pub(crate) fn poll_authorization(&mut self, authorization_url: &str) -> Result<(), String> {
        for _ in 0..ACME_POLL_ATTEMPTS {
            let authorization = self.get_authorization(authorization_url)?;
            match authorization.status.as_str() {
                "valid" => return Ok(()),
                "invalid" => return Err(format!("ACME dns-01 授权失败：{authorization_url}")),
                _ => thread::sleep(Duration::from_secs(ACME_POLL_INTERVAL_SECONDS)),
            }
        }
        Err(format!("等待 ACME dns-01 授权超时：{authorization_url}"))
    }
}
