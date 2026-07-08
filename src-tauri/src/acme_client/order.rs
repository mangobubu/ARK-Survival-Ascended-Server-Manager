use std::{thread, time::Duration};

use crate::acme_crypto::base64_url;
use reqwest::StatusCode;
use serde_json::json;

use super::{ACME_POLL_ATTEMPTS, ACME_POLL_INTERVAL_SECONDS, AcmeClient, AcmeOrder};

impl AcmeClient {
    pub(crate) fn new_order(&mut self, domain: &str) -> Result<(String, AcmeOrder), String> {
        let response = self.post_json_with_kid(
            &self.directory.new_order.clone(),
            json!({
                "identifiers": [
                    { "type": "dns", "value": domain }
                ]
            }),
        )?;
        if !(response.status == StatusCode::CREATED || response.status == StatusCode::OK) {
            return Err(format!(
                "ACME 创建订单失败（HTTP {}）：{}",
                response.status, response.text
            ));
        }
        let order_url = response
            .location
            .ok_or_else(|| "ACME 创建订单响应缺少订单 Location".to_string())?;
        let order = serde_json::from_value::<AcmeOrder>(response.body)
            .map_err(|error| format!("解析 ACME 订单失败：{error}"))?;
        Ok((order_url, order))
    }

    pub(crate) fn finalize_order(
        &mut self,
        finalize_url: &str,
        csr_der: &[u8],
        order_url: &str,
    ) -> Result<AcmeOrder, String> {
        let response = self.post_json_with_kid(
            finalize_url,
            json!({
                "csr": base64_url(csr_der)
            }),
        )?;
        if !response.status.is_success() {
            return Err(format!(
                "ACME Finalize 订单失败（HTTP {}）：{}",
                response.status, response.text
            ));
        }
        let order = serde_json::from_value::<AcmeOrder>(response.body)
            .map_err(|error| format!("解析 ACME Finalize 响应失败：{error}"))?;
        if order.status == "valid" && order.certificate.is_some() {
            return Ok(order);
        }
        self.poll_order(order_url)
    }

    fn poll_order(&mut self, order_url: &str) -> Result<AcmeOrder, String> {
        for _ in 0..ACME_POLL_ATTEMPTS {
            let response = self.post_as_get_json(order_url)?;
            if !response.status.is_success() {
                return Err(format!(
                    "ACME 轮询订单失败（HTTP {}）：{}",
                    response.status, response.text
                ));
            }
            let order = serde_json::from_value::<AcmeOrder>(response.body)
                .map_err(|error| format!("解析 ACME 订单轮询响应失败：{error}"))?;
            match order.status.as_str() {
                "valid" if order.certificate.is_some() => return Ok(order),
                "invalid" => return Err(format!("ACME 订单签发失败：{order_url}")),
                _ => thread::sleep(Duration::from_secs(ACME_POLL_INTERVAL_SECONDS)),
            }
        }
        Err(format!("等待 ACME 订单签发超时：{order_url}"))
    }
}
