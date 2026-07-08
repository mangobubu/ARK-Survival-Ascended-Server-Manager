use std::num::NonZeroU32;

use ring::pbkdf2;
use sha2::{Digest, Sha256};

use super::{
    hash::hash_web_admin_password,
    parser::{PasswordHashAlgorithm, parse_password_hash},
};

pub fn verify_web_admin_password(password: &str, stored: &str) -> Result<bool, String> {
    if stored.is_empty() {
        return Ok(false);
    }

    let Some(parsed) = parse_password_hash(stored)? else {
        return Ok(constant_time_eq(password.as_bytes(), stored.as_bytes()));
    };

    let calculated = match parsed.algorithm {
        PasswordHashAlgorithm::LegacySha256 => {
            derive_password_hash(password, &parsed.salt, parsed.iterations)?
        }
        PasswordHashAlgorithm::Pbkdf2Sha256 => {
            let iterations = NonZeroU32::new(parsed.iterations)
                .ok_or_else(|| "Web 管理员密码哈希迭代次数无效".to_string())?;
            let mut output = [0_u8; 32];
            pbkdf2::derive(
                pbkdf2::PBKDF2_HMAC_SHA256,
                iterations,
                &parsed.salt,
                password.as_bytes(),
                &mut output,
            );
            output
        }
    };
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
