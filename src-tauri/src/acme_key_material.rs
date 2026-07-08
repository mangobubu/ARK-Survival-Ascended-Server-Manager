use crate::acme_persistence::write_atomic;
use rand::rngs::OsRng;
use rsa::{
    RsaPrivateKey,
    pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding},
};
use std::{fs, path::Path};

const RSA_KEY_BITS: usize = 2048;

pub(crate) fn load_or_generate_rsa_private_key(path: &Path) -> Result<RsaPrivateKey, String> {
    if path.is_file() {
        let pem = fs::read_to_string(path)
            .map_err(|error| format!("读取 RSA2048 私钥 {} 失败：{error}", path.display()))?;
        return RsaPrivateKey::from_pkcs8_pem(&pem)
            .map_err(|error| format!("解析 RSA2048 私钥 {} 失败：{error}", path.display()));
    }

    let mut rng = OsRng;
    let key = RsaPrivateKey::new(&mut rng, RSA_KEY_BITS)
        .map_err(|error| format!("生成 RSA2048 私钥失败：{error}"))?;
    write_private_key_pem(path, &key)?;
    Ok(key)
}

pub(crate) fn write_private_key_pem(path: &Path, key: &RsaPrivateKey) -> Result<(), String> {
    let pem = key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|error| format!("编码 RSA2048 私钥失败：{error}"))?;
    write_atomic(path, pem.as_bytes())
}

pub(crate) fn validate_certificate_pem(value: &str) -> Result<(), String> {
    if value.contains("-----BEGIN CERTIFICATE-----") && value.contains("-----END CERTIFICATE-----")
    {
        Ok(())
    } else {
        Err("ACME 返回内容不是 PEM 证书链".to_string())
    }
}
