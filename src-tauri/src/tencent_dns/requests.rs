use serde_json::json;

use super::{TencentDnsCredential, TencentDnsHttpRequest, signing};

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
            "RecordLine": "默认",
            "Value": value,
            "TTL": ttl.clamp(60, 604800),
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
