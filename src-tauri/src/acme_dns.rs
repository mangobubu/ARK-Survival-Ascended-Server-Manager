use crate::{
    acme_certificate::AcmeLogSink,
    acme_client::AcmeClient,
    acme_crypto::dns01_txt_value,
    tencent_dns::{self, TencentDnsCredential, TencentDnsRecord},
};
use std::{thread, time::Duration};

const DNS_TXT_TTL_SECONDS: u32 = tencent_dns::DNSPOD_MIN_COMPATIBLE_TTL_SECONDS;

pub(crate) trait AcmeDnsProvider {
    fn create_txt_record(
        &self,
        domain: &str,
        sub_domain: &str,
        value: &str,
        ttl: u32,
    ) -> Result<TencentDnsRecord, String>;

    fn delete_record(&self, domain: &str, record_id: u64) -> Result<(), String>;
}

pub(crate) struct TencentAcmeDnsProvider {
    credential: TencentDnsCredential,
}

impl TencentAcmeDnsProvider {
    pub(crate) fn new(secret_id: String, secret_key: String) -> Self {
        Self {
            credential: TencentDnsCredential {
                secret_id,
                secret_key,
            },
        }
    }
}

impl AcmeDnsProvider for TencentAcmeDnsProvider {
    fn create_txt_record(
        &self,
        domain: &str,
        sub_domain: &str,
        value: &str,
        ttl: u32,
    ) -> Result<TencentDnsRecord, String> {
        tencent_dns::create_txt_record_blocking(&self.credential, domain, sub_domain, value, ttl)
    }

    fn delete_record(&self, domain: &str, record_id: u64) -> Result<(), String> {
        tencent_dns::delete_record_blocking(&self.credential, domain, record_id)
    }
}

pub(crate) fn perform_dns_authorization(
    client: &mut AcmeClient,
    dns_provider: &dyn AcmeDnsProvider,
    domain: &str,
    authorization_url: &str,
    dns_propagation_wait: Duration,
    log_sink: Option<&AcmeLogSink<'_>>,
) -> Result<(), String> {
    let authorization = client.get_authorization(authorization_url)?;
    if authorization.status == "valid" {
        acme_log(log_sink, "info", "ACME DNS-01 授权已有效，跳过重复验证");
        return Ok(());
    }
    if authorization.status == "invalid" {
        return Err(format!("ACME 授权已失败：{authorization_url}"));
    }

    let challenge = authorization
        .challenges
        .into_iter()
        .find(|challenge| {
            challenge.challenge_type == "dns-01"
                && challenge
                    .status
                    .as_deref()
                    .is_none_or(|status| status == "pending" || status == "processing")
        })
        .ok_or_else(|| "ACME 授权中未找到可用的 dns-01 Challenge".to_string())?;
    let txt_value = dns01_txt_value(&challenge.token, &client.account_thumbprint);
    let (dns_domain, sub_domain) = tencent_dns::acme_challenge_record_for_domain(domain)?;
    acme_log(
        log_sink,
        "info",
        &format!("正在创建腾讯云 DNS TXT 记录：{sub_domain}.{dns_domain}"),
    );
    let record = dns_provider.create_txt_record(
        &dns_domain,
        &sub_domain,
        &txt_value,
        DNS_TXT_TTL_SECONDS,
    )?;

    let result = complete_dns_authorization(
        client,
        authorization_url,
        &challenge.url,
        dns_propagation_wait,
        log_sink,
    );
    acme_log(log_sink, "info", "正在清理腾讯云 DNS TXT 验证记录");
    let cleanup_result = cleanup_dns_record(dns_provider, record);
    match (result, cleanup_result) {
        (Ok(()), _) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(format!(
            "{error}；同时清理腾讯云 DNS TXT 记录失败：{cleanup_error}"
        )),
    }
}

fn complete_dns_authorization(
    client: &mut AcmeClient,
    authorization_url: &str,
    challenge_url: &str,
    dns_propagation_wait: Duration,
    log_sink: Option<&AcmeLogSink<'_>>,
) -> Result<(), String> {
    if !dns_propagation_wait.is_zero() {
        acme_log(
            log_sink,
            "info",
            &format!(
                "等待 DNS TXT 记录传播 {} 秒",
                dns_propagation_wait.as_secs()
            ),
        );
        thread::sleep(dns_propagation_wait);
    }
    acme_log(log_sink, "info", "正在提交 ACME dns-01 Challenge");
    client.accept_challenge(challenge_url)?;
    acme_log(log_sink, "info", "正在轮询 ACME dns-01 授权状态");
    client.poll_authorization(authorization_url)
}

fn cleanup_dns_record(
    dns_provider: &dyn AcmeDnsProvider,
    record: TencentDnsRecord,
) -> Result<(), String> {
    dns_provider.delete_record(&record.domain, record.id)
}

fn acme_log(log_sink: Option<&AcmeLogSink<'_>>, level: &str, message: &str) {
    if let Some(log_sink) = log_sink {
        log_sink(level, message);
    }
}
