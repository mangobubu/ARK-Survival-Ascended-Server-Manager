use base64ct::{Base64UrlUnpadded, Encoding};

const DPAPI_SECRET_PREFIX: &str = "asa-dpapi-v1$";

pub fn protect_secret(value: &str) -> Result<String, String> {
    if value.is_empty() || is_protected_secret(value) {
        return Ok(value.to_string());
    }
    let protected = protect_bytes(value.as_bytes())?;
    Ok(format!(
        "{DPAPI_SECRET_PREFIX}{}",
        Base64UrlUnpadded::encode_string(&protected)
    ))
}

pub fn unprotect_secret(value: &str) -> Result<String, String> {
    let Some(encoded) = value.strip_prefix(DPAPI_SECRET_PREFIX) else {
        return Ok(value.to_string());
    };
    let protected = decode_base64_url(encoded)?;
    let plaintext = unprotect_bytes(&protected)?;
    String::from_utf8(plaintext).map_err(|error| format!("本地加密凭据不是有效 UTF-8：{error}"))
}

pub fn is_protected_secret(value: &str) -> bool {
    value.starts_with(DPAPI_SECRET_PREFIX)
}

#[cfg(windows)]
fn protect_bytes(value: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData},
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: value
            .len()
            .try_into()
            .map_err(|_| "本地加密凭据过大".to_string())?,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };

    let ok = unsafe {
        CryptProtectData(
            &input,
            null(),
            null(),
            null_mut(),
            null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err(format!("Windows DPAPI 加密凭据失败：{}", last_os_error()));
    }

    let protected =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        let _ = LocalFree(output.pbData.cast());
    }
    Ok(protected)
}

#[cfg(windows)]
fn unprotect_bytes(value: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptUnprotectData,
        },
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: value
            .len()
            .try_into()
            .map_err(|_| "本地加密凭据过大".to_string())?,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &input,
            null_mut(),
            null(),
            null_mut(),
            null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err(format!("Windows DPAPI 解密凭据失败：{}", last_os_error()));
    }

    let plaintext =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        let _ = LocalFree(output.pbData.cast());
    }
    Ok(plaintext)
}

#[cfg(not(windows))]
fn protect_bytes(_value: &[u8]) -> Result<Vec<u8>, String> {
    Err("当前平台不支持 Windows DPAPI 本地凭据加密".to_string())
}

#[cfg(not(windows))]
fn unprotect_bytes(_value: &[u8]) -> Result<Vec<u8>, String> {
    Err("当前平台不支持 Windows DPAPI 本地凭据解密".to_string())
}

fn decode_base64_url(value: &str) -> Result<Vec<u8>, String> {
    let mut output = vec![0_u8; value.len().saturating_mul(3) / 4 + 4];
    let decoded = Base64UrlUnpadded::decode(value, &mut output)
        .map_err(|error| format!("本地加密凭据 Base64Url 格式无效：{error}"))?;
    Ok(decoded.to_vec())
}

fn last_os_error() -> std::io::Error {
    std::io::Error::last_os_error()
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn dpapi_可加密并解密_secret() {
        let protected = protect_secret("secret-key").expect("加密 secret");

        assert!(is_protected_secret(&protected));
        assert_ne!(protected, "secret-key");
        assert_eq!(
            unprotect_secret(&protected).expect("解密 secret"),
            "secret-key"
        );
    }

    #[test]
    fn 空_secret_不会写入加密前缀() {
        assert_eq!(protect_secret("").expect("处理空 secret"), "");
        assert_eq!(unprotect_secret("").expect("解密空 secret"), "");
    }
}
