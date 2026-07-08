use std::num::NonZeroU32;

use ring::pbkdf2;

use super::{
    constants::{
        WEB_PASSWORD_PBKDF2_HASH_PREFIX, WEB_PASSWORD_PBKDF2_ITERATIONS, WEB_PASSWORD_SALT_BYTES,
    },
    hex::encode_hex,
};

pub fn hash_web_admin_password(password: &str) -> Result<String, String> {
    let mut salt = [0_u8; WEB_PASSWORD_SALT_BYTES];
    getrandom::fill(&mut salt).map_err(|error| format!("生成 Web 管理员密码盐失败：{error}"))?;
    let mut hash = [0_u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(WEB_PASSWORD_PBKDF2_ITERATIONS)
            .ok_or_else(|| "Web 管理员密码哈希迭代次数无效".to_string())?,
        &salt,
        password.as_bytes(),
        &mut hash,
    );
    Ok(format_password_hash(
        WEB_PASSWORD_PBKDF2_HASH_PREFIX,
        WEB_PASSWORD_PBKDF2_ITERATIONS,
        &salt,
        &hash,
    ))
}

pub(super) fn format_password_hash(
    prefix: &str,
    iterations: u32,
    salt: &[u8],
    hash: &[u8],
) -> String {
    format!(
        "{prefix}${iterations}${}${}",
        encode_hex(salt),
        encode_hex(hash)
    )
}
