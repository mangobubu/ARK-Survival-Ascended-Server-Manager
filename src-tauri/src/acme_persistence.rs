use crate::{
    acme_certificate::{ACME_KEY_ALGORITHM, WebCertificatePaths},
    models::GlobalSettings,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) const STATUS_FILE_NAME: &str = "acme-certificate-status.json";
pub(crate) const PENDING_MANIFEST_FILE_NAME: &str = "acme-pending.json";

const CERTIFICATE_VALIDITY_ASSUMPTION_SECONDS: u64 = 90 * 24 * 60 * 60;
pub(crate) const CERTIFICATE_RENEW_BEFORE_SECONDS: u64 = 30 * 24 * 60 * 60;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AcmeCertificateStatus {
    pub(crate) domain: String,
    pub(crate) directory_url: String,
    pub(crate) key_algorithm: String,
    pub(crate) dns_provider: String,
    pub(crate) issued_at_unix: u64,
    pub(crate) renew_after_unix: u64,
    pub(crate) expires_at_unix: u64,
    pub(crate) fullchain_pem: PathBuf,
    pub(crate) private_key_pem: PathBuf,
}

pub(crate) fn read_certificate_status(
    cert_dir: &Path,
) -> Result<Option<AcmeCertificateStatus>, String> {
    let path = cert_dir.join(STATUS_FILE_NAME);
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("读取 ACME 证书状态文件 {} 失败：{error}", path.display()))?;
    serde_json::from_str::<AcmeCertificateStatus>(&content)
        .map(Some)
        .map_err(|error| format!("解析 ACME 证书状态文件 {} 失败：{error}", path.display()))
}

pub(crate) fn write_certificate_status(
    cert_dir: &Path,
    domain: &str,
    settings: &GlobalSettings,
    paths: &WebCertificatePaths,
) -> Result<(), String> {
    let issued_at_unix = unix_timestamp()?;
    let expires_at_unix = issued_at_unix + CERTIFICATE_VALIDITY_ASSUMPTION_SECONDS;
    let renew_after_unix = expires_at_unix.saturating_sub(CERTIFICATE_RENEW_BEFORE_SECONDS);
    let status = AcmeCertificateStatus {
        domain: domain.to_string(),
        directory_url: settings.web_acme_directory_url.trim().to_string(),
        key_algorithm: ACME_KEY_ALGORITHM.to_string(),
        dns_provider: "tencent-cloud-dnspod".to_string(),
        issued_at_unix,
        renew_after_unix,
        expires_at_unix,
        fullchain_pem: paths.fullchain_pem.clone(),
        private_key_pem: paths.private_key_pem.clone(),
    };
    let content = serde_json::to_vec_pretty(&status)
        .map_err(|error| format!("序列化 ACME 证书状态失败：{error}"))?;
    write_atomic(&cert_dir.join(STATUS_FILE_NAME), &content)
}

pub(crate) fn write_pending_acme_manifest(
    cert_dir: &Path,
    domain: &str,
    settings: &GlobalSettings,
    paths: &WebCertificatePaths,
) -> Result<(), String> {
    let manifest = serde_json::json!({
        "domain": domain,
        "directoryUrl": settings.web_acme_directory_url,
        "accountEmail": settings.web_acme_account_email,
        "dnsProvider": "tencent-cloud-dnspod",
        "dnsCredential": {
            "secretId": settings.web_acme_tencent_secret_id,
            "secretKeyConfigured": !settings.web_acme_tencent_secret_key.is_empty()
        },
        "keyAlgorithm": ACME_KEY_ALGORITHM,
        "dnsChallengeRecord": format!("_acme-challenge.{domain}"),
        "fullchainPem": paths.fullchain_pem,
        "privateKeyPem": paths.private_key_pem
    });
    let content = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("无法序列化 ACME 待签发清单：{error}"))?;
    write_atomic(&cert_dir.join(PENDING_MANIFEST_FILE_NAME), &content)
}

pub(crate) fn write_atomic(path: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建目录 {}：{error}", parent.display()))?;
    }
    let temp_path = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("file")
    ));
    fs::write(&temp_path, content)
        .map_err(|error| format!("无法写入临时文件 {}：{error}", temp_path.display()))?;
    fs::rename(&temp_path, path)
        .map_err(|error| format!("无法替换文件 {}：{error}", path.display()))
}

pub(crate) fn unix_timestamp() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("系统时间早于 UNIX_EPOCH：{error}"))?
        .as_secs())
}
