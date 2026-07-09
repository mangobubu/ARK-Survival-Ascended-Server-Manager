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
    pub ttl: u32,
    pub record_line: String,
    pub cleanup: TencentDnsRecordCleanup,
}

impl TencentDnsRecord {
    pub(crate) fn created(
        id: u64,
        domain: &str,
        sub_domain: &str,
        value: &str,
        ttl: u32,
        record_line: &str,
    ) -> Self {
        Self {
            id,
            domain: domain.to_string(),
            sub_domain: sub_domain.to_string(),
            value: value.to_string(),
            ttl,
            record_line: record_line.to_string(),
            cleanup: TencentDnsRecordCleanup::Delete,
        }
    }

    pub(crate) fn existing_unchanged(
        existing: TencentDnsListedRecord,
        domain: &str,
        sub_domain: &str,
        value: &str,
    ) -> Self {
        Self {
            id: existing.id,
            domain: domain.to_string(),
            sub_domain: sub_domain.to_string(),
            value: value.to_string(),
            ttl: existing.ttl,
            record_line: existing.record_line,
            cleanup: TencentDnsRecordCleanup::Keep,
        }
    }

    pub(crate) fn existing_modified(
        existing: TencentDnsListedRecord,
        domain: &str,
        sub_domain: &str,
        value: &str,
        ttl: u32,
    ) -> Self {
        let cleanup = TencentDnsRecordCleanup::Restore(TencentDnsRecordSnapshot {
            value: existing.value,
            ttl: existing.ttl,
            record_line: existing.record_line.clone(),
        });
        Self {
            id: existing.id,
            domain: domain.to_string(),
            sub_domain: sub_domain.to_string(),
            value: value.to_string(),
            ttl,
            record_line: existing.record_line,
            cleanup,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TencentDnsRecordCleanup {
    Delete,
    Restore(TencentDnsRecordSnapshot),
    Keep,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TencentDnsRecordSnapshot {
    pub value: String,
    pub ttl: u32,
    pub record_line: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TencentDnsListedRecord {
    pub id: u64,
    pub sub_domain: String,
    pub record_type: String,
    pub record_line: String,
    pub value: String,
    pub ttl: u32,
}
