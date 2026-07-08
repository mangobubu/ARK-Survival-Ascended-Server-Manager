#![allow(dead_code)]

mod requests;
mod signing;
mod transport;
mod types;
mod zone;

pub use requests::{build_create_txt_record_request, build_delete_record_request};
pub use types::{TencentDnsCredential, TencentDnsHttpRequest, TencentDnsRecord};
pub use zone::acme_challenge_record_for_domain;

const SERVICE: &str = "dnspod";
const HOST: &str = "dnspod.tencentcloudapi.com";
const ENDPOINT: &str = "https://dnspod.tencentcloudapi.com";
const API_VERSION: &str = "2021-03-23";

pub async fn create_txt_record(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsRecord, String> {
    let request = build_create_txt_record_request(credential, domain, sub_domain, value, ttl)?;
    let payload = transport::send_request(&request).await?;
    let record_id = transport::create_record_id_from_payload(&payload)?;
    Ok(TencentDnsRecord {
        id: record_id,
        domain: domain.to_string(),
        sub_domain: sub_domain.to_string(),
        value: value.to_string(),
    })
}

pub fn create_txt_record_blocking(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsRecord, String> {
    let request = build_create_txt_record_request(credential, domain, sub_domain, value, ttl)?;
    let payload = transport::send_request_blocking(&request)?;
    let record_id = transport::create_record_id_from_payload(&payload)?;
    Ok(TencentDnsRecord {
        id: record_id,
        domain: domain.to_string(),
        sub_domain: sub_domain.to_string(),
        value: value.to_string(),
    })
}

pub async fn delete_record(
    credential: &TencentDnsCredential,
    domain: &str,
    record_id: u64,
) -> Result<(), String> {
    let request = build_delete_record_request(credential, domain, record_id)?;
    let _ = transport::send_request(&request).await?;
    Ok(())
}

pub fn delete_record_blocking(
    credential: &TencentDnsCredential,
    domain: &str,
    record_id: u64,
) -> Result<(), String> {
    let request = build_delete_record_request(credential, domain, record_id)?;
    let _ = transport::send_request_blocking(&request)?;
    Ok(())
}

#[cfg(test)]
mod tests;
