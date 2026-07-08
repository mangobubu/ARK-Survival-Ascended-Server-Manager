use super::*;

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
