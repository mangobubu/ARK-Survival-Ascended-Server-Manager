use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{ENDPOINT, HOST, SERVICE, TencentDnsCredential, TencentDnsHttpRequest};
pub(super) fn build_signed_request(
    credential: &TencentDnsCredential,
    action: &str,
    body: String,
) -> Result<TencentDnsHttpRequest, String> {
    if credential.secret_id.trim().is_empty() || credential.secret_key.trim().is_empty() {
        return Err("腾讯云 Secret ID 和 Secret Key 不能为空".to_string());
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("读取系统时间失败：{error}"))?
        .as_secs() as i64;
    let date = utc_date_from_unix(timestamp);
    let authorization = tc3_authorization(credential, action, &body, timestamp, &date);
    Ok(TencentDnsHttpRequest {
        url: ENDPOINT.to_string(),
        body,
        authorization,
        timestamp,
        date,
        action: action.to_string(),
    })
}

fn tc3_authorization(
    credential: &TencentDnsCredential,
    _action: &str,
    body: &str,
    timestamp: i64,
    date: &str,
) -> String {
    let canonical_request = format!(
        "POST\n/\n\ncontent-type:application/json; charset=utf-8\nhost:{HOST}\n\ncontent-type;host\n{}",
        sha256_hex(body.as_bytes())
    );
    let credential_scope = format!("{date}/{SERVICE}/tc3_request");
    let string_to_sign = format!(
        "TC3-HMAC-SHA256\n{timestamp}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );
    let secret_date = hmac_sha256(
        format!("TC3{}", credential.secret_key).as_bytes(),
        date.as_bytes(),
    );
    let secret_service = hmac_sha256(&secret_date, SERVICE.as_bytes());
    let secret_signing = hmac_sha256(&secret_service, b"tc3_request");
    let signature = hex(&hmac_sha256(&secret_signing, string_to_sign.as_bytes()));
    format!(
        "TC3-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders=content-type;host, Signature={signature}",
        credential.secret_id
    )
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;
    let mut normalized_key = [0_u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        normalized_key[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        normalized_key[..key.len()].copy_from_slice(key);
    }
    let mut outer = [0x5c_u8; BLOCK_SIZE];
    let mut inner = [0x36_u8; BLOCK_SIZE];
    for index in 0..BLOCK_SIZE {
        outer[index] ^= normalized_key[index];
        inner[index] ^= normalized_key[index];
    }

    let mut inner_digest = Sha256::new();
    inner_digest.update(inner);
    inner_digest.update(message);
    let inner_hash = inner_digest.finalize();

    let mut outer_digest = Sha256::new();
    outer_digest.update(outer);
    outer_digest.update(inner_hash);
    outer_digest.finalize().into()
}

fn sha256_hex(value: &[u8]) -> String {
    hex(&Sha256::digest(value))
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(hex_char(byte >> 4));
        output.push(hex_char(byte & 0x0f));
    }
    output
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => '0',
    }
}

pub(super) fn utc_date_from_unix(timestamp: i64) -> String {
    let days = timestamp.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}
