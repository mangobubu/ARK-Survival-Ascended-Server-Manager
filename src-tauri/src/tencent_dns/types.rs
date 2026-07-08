#[derive(Clone, Debug)]
pub struct TencentDnsCredential {
    pub secret_id: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TencentDnsHttpRequest {
    pub url: String,
    pub body: String,
    pub authorization: String,
    pub timestamp: i64,
    pub date: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TencentDnsRecord {
    pub id: u64,
    pub domain: String,
    pub sub_domain: String,
    pub value: String,
}
