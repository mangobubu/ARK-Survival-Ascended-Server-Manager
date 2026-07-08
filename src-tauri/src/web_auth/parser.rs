use super::{
    constants::{WEB_PASSWORD_HASH_PREFIX, WEB_PASSWORD_PBKDF2_HASH_PREFIX},
    hex::decode_hex,
};

pub(super) enum PasswordHashAlgorithm {
    LegacySha256,
    Pbkdf2Sha256,
}

pub(super) struct ParsedPasswordHash {
    pub(super) algorithm: PasswordHashAlgorithm,
    pub(super) iterations: u32,
    pub(super) salt: Vec<u8>,
    pub(super) hash: Vec<u8>,
}

pub(super) fn parse_password_hash(value: &str) -> Result<Option<ParsedPasswordHash>, String> {
    let parts = value.split('$').collect::<Vec<_>>();
    if parts.len() != 4 {
        return Ok(None);
    }
    let algorithm = match parts[0] {
        WEB_PASSWORD_HASH_PREFIX => PasswordHashAlgorithm::LegacySha256,
        WEB_PASSWORD_PBKDF2_HASH_PREFIX => PasswordHashAlgorithm::Pbkdf2Sha256,
        _ => return Ok(None),
    };

    let iterations = parts[1]
        .parse::<u32>()
        .map_err(|error| format!("Web 管理员密码哈希迭代次数无效：{error}"))?;
    let salt = decode_hex(parts[2])?;
    let hash = decode_hex(parts[3])?;
    if salt.is_empty() || hash.len() != 32 {
        return Err("Web 管理员密码哈希格式无效".to_string());
    }

    Ok(Some(ParsedPasswordHash {
        algorithm,
        iterations,
        salt,
        hash,
    }))
}
