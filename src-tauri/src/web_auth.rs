use sha2::{Digest, Sha256};

const WEB_PASSWORD_HASH_PREFIX: &str = "asa-sha256-v1";
const WEB_PASSWORD_HASH_ITERATIONS: u32 = 120_000;
const WEB_PASSWORD_SALT_BYTES: usize = 16;

pub fn hash_web_admin_password(password: &str) -> Result<String, String> {
    let mut salt = [0_u8; WEB_PASSWORD_SALT_BYTES];
    getrandom::fill(&mut salt).map_err(|error| format!("生成 Web 管理员密码盐失败：{error}"))?;
    Ok(format_password_hash(
        WEB_PASSWORD_HASH_ITERATIONS,
        &salt,
        &derive_password_hash(password, &salt, WEB_PASSWORD_HASH_ITERATIONS)?,
    ))
}

pub fn verify_web_admin_password(password: &str, stored: &str) -> Result<bool, String> {
    if stored.is_empty() {
        return Ok(false);
    }

    let Some(parsed) = parse_password_hash(stored)? else {
        return Ok(constant_time_eq(password.as_bytes(), stored.as_bytes()));
    };

    let calculated = derive_password_hash(password, &parsed.salt, parsed.iterations)?;
    Ok(constant_time_eq(&calculated, &parsed.hash))
}

pub fn is_web_admin_password_hash(value: &str) -> bool {
    matches!(parse_password_hash(value), Ok(Some(_)))
}

pub fn migrate_web_admin_password_hash(password_or_hash: &mut String) -> Result<bool, String> {
    if password_or_hash.is_empty() || is_web_admin_password_hash(password_or_hash) {
        return Ok(false);
    }

    let hashed = hash_web_admin_password(password_or_hash)?;
    *password_or_hash = hashed;
    Ok(true)
}

fn derive_password_hash(password: &str, salt: &[u8], iterations: u32) -> Result<[u8; 32], String> {
    if iterations == 0 {
        return Err("Web 管理员密码哈希迭代次数无效".to_string());
    }

    let mut digest = Sha256::new();
    digest.update(salt);
    digest.update(password.as_bytes());
    let mut output: [u8; 32] = digest.finalize().into();

    for _ in 1..iterations {
        let mut digest = Sha256::new();
        digest.update(output);
        digest.update(salt);
        digest.update(password.as_bytes());
        output = digest.finalize().into();
    }

    Ok(output)
}

fn format_password_hash(iterations: u32, salt: &[u8], hash: &[u8]) -> String {
    format!(
        "{WEB_PASSWORD_HASH_PREFIX}${iterations}${}${}",
        encode_hex(salt),
        encode_hex(hash)
    )
}

struct ParsedPasswordHash {
    iterations: u32,
    salt: Vec<u8>,
    hash: Vec<u8>,
}

fn parse_password_hash(value: &str) -> Result<Option<ParsedPasswordHash>, String> {
    let parts = value.split('$').collect::<Vec<_>>();
    if parts.len() != 4 || parts[0] != WEB_PASSWORD_HASH_PREFIX {
        return Ok(None);
    }

    let iterations = parts[1]
        .parse::<u32>()
        .map_err(|error| format!("Web 管理员密码哈希迭代次数无效：{error}"))?;
    let salt = decode_hex(parts[2])?;
    let hash = decode_hex(parts[3])?;
    if salt.is_empty() || hash.len() != 32 {
        return Err("Web 管理员密码哈希格式无效".to_string());
    }

    Ok(Some(ParsedPasswordHash {
        iterations,
        salt,
        hash,
    }))
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(hex_char(byte >> 4));
        output.push(hex_char(byte & 0x0f));
    }
    output
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("十六进制文本长度无效".to_string());
    }

    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_value(bytes[index])?;
        let low = hex_value(bytes[index + 1])?;
        output.push((high << 4) | low);
        index += 2;
    }
    Ok(output)
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => '0',
    }
}

fn hex_value(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err("十六进制文本包含非法字符".to_string()),
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut diff = left.len() ^ right.len();
    let max_len = left.len().max(right.len());
    for index in 0..max_len {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        diff |= usize::from(left_byte ^ right_byte);
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_password_with_salt_and_verifies_it() {
        let hash = hash_web_admin_password("secret").expect("生成密码哈希");

        assert_ne!(hash, "secret");
        assert!(is_web_admin_password_hash(&hash));
        assert!(verify_web_admin_password("secret", &hash).expect("校验正确密码"));
        assert!(!verify_web_admin_password("wrong", &hash).expect("校验错误密码"));
    }

    #[test]
    fn generated_hashes_use_random_salt() {
        let first = hash_web_admin_password("secret").expect("生成第一个哈希");
        let second = hash_web_admin_password("secret").expect("生成第二个哈希");

        assert_ne!(first, second);
    }

    #[test]
    fn migrates_legacy_plaintext_once() {
        let mut stored = "legacy-secret".to_string();

        assert!(migrate_web_admin_password_hash(&mut stored).expect("迁移明文密码"));
        assert_ne!(stored, "legacy-secret");
        assert!(verify_web_admin_password("legacy-secret", &stored).expect("校验迁移后密码"));

        let migrated = stored.clone();
        assert!(!migrate_web_admin_password_hash(&mut stored).expect("已迁移密码不重复迁移"));
        assert_eq!(stored, migrated);
    }
}
