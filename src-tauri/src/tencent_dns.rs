#![allow(dead_code)]

mod requests;
mod signing;
mod transport;
mod types;
mod zone;

pub use requests::{
    build_create_txt_record_request, build_delete_record_request,
    build_describe_txt_records_request, build_modify_txt_record_request,
};
pub use types::{
    TencentDnsCredential, TencentDnsHttpRequest, TencentDnsRecord, TencentDnsRecordCleanup,
};
pub use zone::acme_challenge_record_for_domain;

const SERVICE: &str = "dnspod";
const HOST: &str = "dnspod.tencentcloudapi.com";
const ENDPOINT: &str = "https://dnspod.tencentcloudapi.com";
const API_VERSION: &str = "2021-03-23";
pub const DNSPOD_MIN_COMPATIBLE_TTL_SECONDS: u32 = 600;
pub const DNSPOD_MAX_TTL_SECONDS: u32 = 604800;

pub async fn create_txt_record(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsRecord, String> {
    let request = build_create_txt_record_request(credential, domain, sub_domain, value, ttl)?;
    match transport::send_request(&request).await {
        Ok(payload) => record_from_create_payload(&payload, domain, sub_domain, value, ttl),
        Err(error) if transport::is_record_exist_error(&error) => {
            upsert_existing_txt_record(credential, domain, sub_domain, value, ttl)
                .await
                .map_err(|upsert_error| {
                    format!("{error}；尝试复用或修改已有腾讯云 DNS TXT 记录失败：{upsert_error}")
                })
        }
        Err(error) => Err(error),
    }
}

pub fn create_txt_record_blocking(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsRecord, String> {
    let request = build_create_txt_record_request(credential, domain, sub_domain, value, ttl)?;
    match transport::send_request_blocking(&request) {
        Ok(payload) => record_from_create_payload(&payload, domain, sub_domain, value, ttl),
        Err(error) if transport::is_record_exist_error(&error) => {
            upsert_existing_txt_record_blocking(credential, domain, sub_domain, value, ttl).map_err(
                |upsert_error| {
                    format!("{error}；尝试复用或修改已有腾讯云 DNS TXT 记录失败：{upsert_error}")
                },
            )
        }
        Err(error) => Err(error),
    }
}

pub async fn describe_txt_records(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
) -> Result<Vec<types::TencentDnsListedRecord>, String> {
    let request = build_describe_txt_records_request(credential, domain, sub_domain)?;
    let payload = transport::send_request(&request).await?;
    transport::record_list_from_payload(&payload)
}

pub fn describe_txt_records_blocking(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
) -> Result<Vec<types::TencentDnsListedRecord>, String> {
    let request = build_describe_txt_records_request(credential, domain, sub_domain)?;
    let payload = transport::send_request_blocking(&request)?;
    transport::record_list_from_payload(&payload)
}

pub async fn modify_txt_record(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    record_id: u64,
    value: &str,
    ttl: u32,
    record_line: &str,
) -> Result<(), String> {
    let request = build_modify_txt_record_request(
        credential,
        domain,
        sub_domain,
        record_id,
        value,
        ttl,
        record_line,
    )?;
    let _ = transport::send_request(&request).await?;
    Ok(())
}

pub fn modify_txt_record_blocking(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    record_id: u64,
    value: &str,
    ttl: u32,
    record_line: &str,
) -> Result<(), String> {
    let request = build_modify_txt_record_request(
        credential,
        domain,
        sub_domain,
        record_id,
        value,
        ttl,
        record_line,
    )?;
    let _ = transport::send_request_blocking(&request)?;
    Ok(())
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

pub fn cleanup_record_blocking(
    credential: &TencentDnsCredential,
    record: &TencentDnsRecord,
) -> Result<(), String> {
    match &record.cleanup {
        TencentDnsRecordCleanup::Delete => {
            delete_record_blocking(credential, &record.domain, record.id)
        }
        TencentDnsRecordCleanup::Restore(snapshot) => modify_txt_record_blocking(
            credential,
            &record.domain,
            &record.sub_domain,
            record.id,
            &snapshot.value,
            snapshot.ttl,
            &snapshot.record_line,
        ),
        TencentDnsRecordCleanup::Keep => Ok(()),
    }
}

fn record_from_create_payload(
    payload: &serde_json::Value,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsRecord, String> {
    let record_id = transport::create_record_id_from_payload(payload)?;
    Ok(TencentDnsRecord::created(
        record_id,
        domain,
        sub_domain,
        value,
        ttl.clamp(DNSPOD_MIN_COMPATIBLE_TTL_SECONDS, DNSPOD_MAX_TTL_SECONDS),
        requests::DEFAULT_RECORD_LINE,
    ))
}

async fn upsert_existing_txt_record(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsRecord, String> {
    let existing = find_existing_txt_record(
        describe_txt_records(credential, domain, sub_domain).await?,
        sub_domain,
    )?;
    if existing.value == value {
        return Ok(TencentDnsRecord::existing_unchanged(
            existing, domain, sub_domain, value,
        ));
    }
    let target_ttl = ttl.clamp(DNSPOD_MIN_COMPATIBLE_TTL_SECONDS, DNSPOD_MAX_TTL_SECONDS);
    modify_txt_record(
        credential,
        domain,
        sub_domain,
        existing.id,
        value,
        target_ttl,
        &existing.record_line,
    )
    .await?;
    Ok(TencentDnsRecord::existing_modified(
        existing, domain, sub_domain, value, target_ttl,
    ))
}

fn upsert_existing_txt_record_blocking(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsRecord, String> {
    let existing = find_existing_txt_record(
        describe_txt_records_blocking(credential, domain, sub_domain)?,
        sub_domain,
    )?;
    if existing.value == value {
        return Ok(TencentDnsRecord::existing_unchanged(
            existing, domain, sub_domain, value,
        ));
    }
    let target_ttl = ttl.clamp(DNSPOD_MIN_COMPATIBLE_TTL_SECONDS, DNSPOD_MAX_TTL_SECONDS);
    modify_txt_record_blocking(
        credential,
        domain,
        sub_domain,
        existing.id,
        value,
        target_ttl,
        &existing.record_line,
    )?;
    Ok(TencentDnsRecord::existing_modified(
        existing, domain, sub_domain, value, target_ttl,
    ))
}

fn find_existing_txt_record(
    records: Vec<types::TencentDnsListedRecord>,
    sub_domain: &str,
) -> Result<types::TencentDnsListedRecord, String> {
    records
        .into_iter()
        .find(|record| {
            record.sub_domain.eq_ignore_ascii_case(sub_domain)
                && record.record_type.eq_ignore_ascii_case("TXT")
        })
        .ok_or_else(|| format!("腾讯云 DNSPod 返回记录已存在，但未查询到 {sub_domain} 的 TXT 记录"))
}

#[cfg(test)]
mod tests;
