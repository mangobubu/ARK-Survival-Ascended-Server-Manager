use crate::{
    acme_client::AcmeClient,
    acme_crypto::build_csr_der,
    acme_dns::{AcmeDnsProvider, TencentAcmeDnsProvider, perform_dns_authorization},
    acme_key_material::{
        load_or_generate_rsa_private_key, validate_certificate_pem, write_private_key_pem,
    },
    acme_persistence::{
        CERTIFICATE_RENEW_BEFORE_SECONDS, PENDING_MANIFEST_FILE_NAME, read_certificate_status,
        unix_timestamp, write_atomic, write_certificate_status, write_pending_acme_manifest,
    },
    models::GlobalSettings,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use x509_cert::Certificate;

pub const ACME_KEY_ALGORITHM: &str = "RSA2048";
pub const LETS_ENCRYPT_DIRECTORY_URL: &str = "https://acme-v02.api.letsencrypt.org/directory";

const ACCOUNT_KEY_FILE_NAME: &str = "acme-account.RSA2048.key.pem";
const DNS_PROPAGATION_WAIT_SECONDS: u64 = 30;

pub(crate) type AcmeLogSink<'a> = dyn Fn(&str, &str) + 'a;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebCertificatePaths {
    pub fullchain_pem: PathBuf,
    pub private_key_pem: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAcmeCertificateStatus {
    pub domain: String,
    pub issued_at_unix: Option<u64>,
    pub renew_after_unix: u64,
    pub expires_at_unix: u64,
    pub fullchain_pem: PathBuf,
    pub private_key_pem: PathBuf,
}

pub fn certificate_paths(cert_dir: &Path, domain: &str) -> WebCertificatePaths {
    let safe_domain = domain
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.' => ch,
            _ => '_',
        })
        .collect::<String>();
    WebCertificatePaths {
        fullchain_pem: cert_dir.join(format!("{safe_domain}.fullchain.pem")),
        private_key_pem: cert_dir.join(format!("{safe_domain}.{ACME_KEY_ALGORITHM}.key.pem")),
    }
}

pub fn read_web_certificate_status(
    cert_dir: &Path,
    domain: &str,
) -> Result<Option<WebAcmeCertificateStatus>, String> {
    let paths = certificate_paths(cert_dir, domain);
    if !paths.fullchain_pem.is_file() || !paths.private_key_pem.is_file() {
        return Ok(None);
    }

    let persisted_status = read_certificate_status(cert_dir)?
        .filter(|status| status.domain == domain && status.key_algorithm == ACME_KEY_ALGORITHM);
    let parsed_validity = read_certificate_validity_unix(&paths.fullchain_pem).ok();

    let Some(expires_at_unix) = parsed_validity
        .map(|(_, expires_at_unix)| expires_at_unix)
        .or_else(|| {
            persisted_status
                .as_ref()
                .map(|status| status.expires_at_unix)
        })
    else {
        return Err(format!(
            "已找到证书文件 {}，但无法读取证书到期时间",
            paths.fullchain_pem.display()
        ));
    };

    let issued_at_unix = parsed_validity
        .map(|(issued_at_unix, _)| issued_at_unix)
        .or_else(|| {
            persisted_status
                .as_ref()
                .map(|status| status.issued_at_unix)
        });
    let renew_after_unix = expires_at_unix.saturating_sub(CERTIFICATE_RENEW_BEFORE_SECONDS);

    Ok(Some(WebAcmeCertificateStatus {
        domain: domain.to_string(),
        issued_at_unix,
        renew_after_unix,
        expires_at_unix,
        fullchain_pem: paths.fullchain_pem,
        private_key_pem: paths.private_key_pem,
    }))
}

pub fn ensure_certificate_files(
    cert_dir: &Path,
    domain: &str,
    settings: &GlobalSettings,
    log_sink: Option<&AcmeLogSink<'_>>,
) -> Result<WebCertificatePaths, String> {
    fs::create_dir_all(cert_dir).map_err(|error| {
        format!(
            "无法创建 Web HTTPS 证书目录 {}：{error}",
            cert_dir.display()
        )
    })?;
    let paths = certificate_paths(cert_dir, domain);
    let cert_files_ready = paths.fullchain_pem.is_file() && paths.private_key_pem.is_file();

    if cert_files_ready
        && (!settings.web_acme_auto_issue_enabled || !certificate_renewal_due(cert_dir, domain)?)
    {
        acme_log(
            log_sink,
            "info",
            &format!("已检测到可用 HTTPS 证书，当前无需续期：{domain}"),
        );
        return Ok(paths);
    }

    if !settings.web_acme_auto_issue_enabled {
        return Err(format!(
            "启用 HTTPS 前需要先准备证书文件：{} 和 {}，或开启 Let's Encrypt 自动申请/续期",
            paths.fullchain_pem.display(),
            paths.private_key_pem.display()
        ));
    }

    validate_acme_settings(settings)?;
    acme_log(
        log_sink,
        "info",
        &format!("准备通过 Let's Encrypt ACME DNS-01 自动申请/续期证书：{domain}"),
    );
    write_pending_acme_manifest(cert_dir, domain, settings, &paths)?;
    issue_certificate_blocking(cert_dir, domain, settings, &paths, log_sink)?;
    Ok(paths)
}

fn read_certificate_validity_unix(path: &Path) -> Result<(u64, u64), String> {
    let content =
        fs::read(path).map_err(|error| format!("读取证书文件 {} 失败：{error}", path.display()))?;
    let certificates = Certificate::load_pem_chain(&content)
        .map_err(|error| format!("解析证书文件 {} 失败：{error}", path.display()))?;
    let certificate = certificates
        .first()
        .ok_or_else(|| format!("证书文件 {} 未包含任何 PEM 证书", path.display()))?;
    let validity = certificate.tbs_certificate.validity;
    Ok((
        validity.not_before.to_unix_duration().as_secs(),
        validity.not_after.to_unix_duration().as_secs(),
    ))
}

fn certificate_renewal_due(cert_dir: &Path, domain: &str) -> Result<bool, String> {
    let Some(status) = read_certificate_status(cert_dir)? else {
        return Ok(true);
    };
    if status.domain != domain || status.key_algorithm != ACME_KEY_ALGORITHM {
        return Ok(true);
    }
    let now = unix_timestamp()?;
    Ok(now >= status.renew_after_unix
        || status.expires_at_unix.saturating_sub(now) <= CERTIFICATE_RENEW_BEFORE_SECONDS)
}

fn issue_certificate_blocking(
    cert_dir: &Path,
    domain: &str,
    settings: &GlobalSettings,
    paths: &WebCertificatePaths,
    log_sink: Option<&AcmeLogSink<'_>>,
) -> Result<(), String> {
    let dns_provider = TencentAcmeDnsProvider::new(
        settings.web_acme_tencent_secret_id.trim().to_string(),
        settings.web_acme_tencent_secret_key.trim().to_string(),
    );
    issue_certificate_with_dns_provider(
        cert_dir,
        domain,
        settings,
        paths,
        &dns_provider,
        Duration::from_secs(DNS_PROPAGATION_WAIT_SECONDS),
        log_sink,
    )
}

fn issue_certificate_with_dns_provider(
    cert_dir: &Path,
    domain: &str,
    settings: &GlobalSettings,
    paths: &WebCertificatePaths,
    dns_provider: &dyn AcmeDnsProvider,
    dns_propagation_wait: Duration,
    log_sink: Option<&AcmeLogSink<'_>>,
) -> Result<(), String> {
    let account_key_path = cert_dir.join(ACCOUNT_KEY_FILE_NAME);
    acme_log(log_sink, "info", "正在准备 ACME 账户私钥和证书私钥");
    let account_key = load_or_generate_rsa_private_key(&account_key_path)?;
    let certificate_key = load_or_generate_rsa_private_key(&paths.private_key_pem)?;

    acme_log(log_sink, "info", "正在连接 Let's Encrypt ACME Directory");
    let mut client = AcmeClient::connect(settings.web_acme_directory_url.trim(), account_key)?;
    acme_log(log_sink, "info", "正在确认 Let's Encrypt ACME 账户");
    client.ensure_account(settings.web_acme_account_email.trim())?;
    acme_log(log_sink, "info", "正在创建 ACME 证书订单");
    let (order_url, order) = client.new_order(domain)?;

    for authorization_url in &order.authorizations {
        perform_dns_authorization(
            &mut client,
            dns_provider,
            domain,
            authorization_url,
            dns_propagation_wait,
            log_sink,
        )?;
    }

    acme_log(
        log_sink,
        "info",
        "DNS-01 授权已完成，正在生成 CSR 并提交签发",
    );
    let csr_der = build_csr_der(domain, &certificate_key)?;
    let finalized_order = client.finalize_order(&order.finalize, &csr_der, &order_url)?;
    let certificate_url = finalized_order
        .certificate
        .ok_or_else(|| "ACME 订单已完成但响应缺少证书下载地址".to_string())?;
    acme_log(log_sink, "info", "ACME 订单已签发，正在下载证书链");
    let certificate_pem = client.post_as_get_text(&certificate_url)?;
    validate_certificate_pem(&certificate_pem)?;

    acme_log(log_sink, "info", "正在写入 HTTPS 证书文件和续期状态");
    write_private_key_pem(&paths.private_key_pem, &certificate_key)?;
    write_atomic(&paths.fullchain_pem, certificate_pem.as_bytes())?;
    write_certificate_status(cert_dir, domain, settings, paths)?;
    let pending_path = cert_dir.join(PENDING_MANIFEST_FILE_NAME);
    if pending_path.is_file() {
        let _ = fs::remove_file(pending_path);
    }
    acme_log(log_sink, "success", "Let's Encrypt 证书申请/续期已完成");
    Ok(())
}

fn acme_log(log_sink: Option<&AcmeLogSink<'_>>, level: &str, message: &str) {
    if let Some(log_sink) = log_sink {
        log_sink(level, message);
    }
}

fn validate_acme_settings(settings: &GlobalSettings) -> Result<(), String> {
    if settings.web_acme_directory_url.trim() != LETS_ENCRYPT_DIRECTORY_URL {
        return Err("当前仅支持 Let's Encrypt 正式环境 ACME v2 目录地址".to_string());
    }
    if settings.web_acme_account_email.trim().is_empty()
        || !settings.web_acme_account_email.contains('@')
    {
        return Err("ACME 账户邮箱无效".to_string());
    }
    if settings.web_acme_tencent_secret_id.trim().is_empty() {
        return Err("腾讯云 Secret ID 不能为空".to_string());
    }
    if settings.web_acme_tencent_secret_key.trim().is_empty() {
        return Err("腾讯云 Secret Key 不能为空".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
