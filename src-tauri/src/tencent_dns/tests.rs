use super::*;
use serde_json::{Value, json};

#[test]
fn unix_时间能转换为_utc_日期() {
    assert_eq!(signing::utc_date_from_unix(0), "1970-01-01");
    assert_eq!(signing::utc_date_from_unix(1_704_067_200), "2024-01-01");
}

#[test]
fn 构造腾讯云_txt_记录请求会包含_dnspod_签名() {
    let request = build_create_txt_record_request(
        &TencentDnsCredential {
            secret_id: "AKIDEXAMPLE".to_string(),
            secret_key: "SECRETEXAMPLE".to_string(),
        },
        "example.com",
        "_acme-challenge",
        "dns-token",
        60,
    )
    .expect("构造请求");

    assert_eq!(request.url, ENDPOINT);
    assert!(request.authorization.contains("TC3-HMAC-SHA256"));
    assert!(request.authorization.contains("Credential=AKIDEXAMPLE/"));
    assert!(request.body.contains("\"RecordType\":\"TXT\""));
}

#[test]
fn 构造腾讯云_txt_记录请求会使用兼容免费版的_ttl() {
    let request = build_create_txt_record_request(
        &TencentDnsCredential {
            secret_id: "AKIDEXAMPLE".to_string(),
            secret_key: "SECRETEXAMPLE".to_string(),
        },
        "example.com",
        "_acme-challenge",
        "dns-token",
        60,
    )
    .expect("构造请求");
    let body: Value = serde_json::from_str(&request.body).expect("解析请求体");

    assert_eq!(body["TTL"].as_u64(), Some(600));
}

#[test]
fn 构造腾讯云_txt_记录查询请求会按子域名和类型过滤() {
    let request = build_describe_txt_records_request(
        &TencentDnsCredential {
            secret_id: "AKIDEXAMPLE".to_string(),
            secret_key: "SECRETEXAMPLE".to_string(),
        },
        "example.com",
        "_acme-challenge.ark",
    )
    .expect("构造请求");
    let body: Value = serde_json::from_str(&request.body).expect("解析请求体");

    assert_eq!(request.action, "DescribeRecordList");
    assert_eq!(body["Domain"].as_str(), Some("example.com"));
    assert_eq!(body["Subdomain"].as_str(), Some("_acme-challenge.ark"));
    assert_eq!(body["RecordType"].as_str(), Some("TXT"));
    assert_eq!(body["RecordLine"].as_str(), Some("默认"));
    assert_eq!(body["ErrorOnEmpty"].as_str(), Some("no"));
}

#[test]
fn 构造腾讯云_txt_记录修改请求会带上_record_id_和新值() {
    let request = build_modify_txt_record_request(
        &TencentDnsCredential {
            secret_id: "AKIDEXAMPLE".to_string(),
            secret_key: "SECRETEXAMPLE".to_string(),
        },
        "example.com",
        "_acme-challenge.ark",
        123,
        "new-token",
        60,
        "默认",
    )
    .expect("构造请求");
    let body: Value = serde_json::from_str(&request.body).expect("解析请求体");

    assert_eq!(request.action, "ModifyTXTRecord");
    assert_eq!(body["RecordId"].as_u64(), Some(123));
    assert_eq!(body["SubDomain"].as_str(), Some("_acme-challenge.ark"));
    assert_eq!(body["Value"].as_str(), Some("new-token"));
    assert_eq!(body["TTL"].as_u64(), Some(60));
}

#[test]
fn 能识别_dnspod_记录已存在错误并解析已有_txt_记录() {
    let error = r#"腾讯云 DNSPod API 返回失败（HTTP 200 OK）：{"Response":{"Error":{"Code":"InvalidParameter.DomainRecordExist","Message":"记录已经存在，无需再次添加。"}}}"#;
    assert!(transport::is_record_exist_error(error));

    let payload = json!({
        "Response": {
            "RecordList": [{
                "RecordId": 123,
                "Name": "_acme-challenge.ark",
                "Type": "TXT",
                "Line": "默认",
                "Value": "old-token",
                "TTL": 600
            }]
        }
    });
    let records = transport::record_list_from_payload(&payload).expect("解析记录列表");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, 123);
    assert_eq!(records[0].sub_domain, "_acme-challenge.ark");
    assert_eq!(records[0].record_type, "TXT");
    assert_eq!(records[0].value, "old-token");
}

#[test]
fn 能拆到_acme_challenge_记录到_dnspod_主域名() {
    assert_eq!(
        acme_challenge_record_for_domain("ark.example.com").unwrap(),
        ("example.com".to_string(), "_acme-challenge.ark".to_string())
    );
    assert_eq!(
        acme_challenge_record_for_domain("ark.example.com.cn").unwrap(),
        (
            "example.com.cn".to_string(),
            "_acme-challenge.ark".to_string()
        )
    );
}
