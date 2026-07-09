use serde_json::json;

use super::{
    DNSPOD_MAX_TTL_SECONDS, DNSPOD_MIN_COMPATIBLE_TTL_SECONDS, TencentDnsCredential,
    TencentDnsHttpRequest, signing,
};

pub(super) const DEFAULT_RECORD_LINE: &str = "默认";

pub fn build_create_txt_record_request(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    value: &str,
    ttl: u32,
) -> Result<TencentDnsHttpRequest, String> {
    signing::build_signed_request(
        credential,
        "CreateRecord",
        json!({
            "Domain": domain,
            "SubDomain": sub_domain,
            "RecordType": "TXT",
            "RecordLine": DEFAULT_RECORD_LINE,
            "Value": value,
            "TTL": ttl.clamp(DNSPOD_MIN_COMPATIBLE_TTL_SECONDS, DNSPOD_MAX_TTL_SECONDS),
        })
        .to_string(),
    )
}

pub fn build_describe_txt_records_request(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
) -> Result<TencentDnsHttpRequest, String> {
    signing::build_signed_request(
        credential,
        "DescribeRecordList",
        json!({
            "Domain": domain,
            "Subdomain": sub_domain,
            "RecordType": "TXT",
            "RecordLine": DEFAULT_RECORD_LINE,
            "Limit": 100,
            "ErrorOnEmpty": "no",
        })
        .to_string(),
    )
}

pub fn build_modify_txt_record_request(
    credential: &TencentDnsCredential,
    domain: &str,
    sub_domain: &str,
    record_id: u64,
    value: &str,
    ttl: u32,
    record_line: &str,
) -> Result<TencentDnsHttpRequest, String> {
    signing::build_signed_request(
        credential,
        "ModifyTXTRecord",
        json!({
            "Domain": domain,
            "SubDomain": sub_domain,
            "RecordLine": record_line,
            "Value": value,
            "RecordId": record_id,
            "TTL": ttl.clamp(1, DNSPOD_MAX_TTL_SECONDS),
        })
        .to_string(),
    )
}

pub fn build_delete_record_request(
    credential: &TencentDnsCredential,
    domain: &str,
    record_id: u64,
) -> Result<TencentDnsHttpRequest, String> {
    signing::build_signed_request(
        credential,
        "DeleteRecord",
        json!({
            "Domain": domain,
            "RecordId": record_id,
        })
        .to_string(),
    )
}
