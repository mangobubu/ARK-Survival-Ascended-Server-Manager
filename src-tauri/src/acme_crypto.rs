use base64ct::{Base64UrlUnpadded, Encoding};
use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    signature::{SignatureEncoding, Signer},
    traits::PublicKeyParts,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(crate) fn rsa_public_jwk(key: &RsaPrivateKey) -> Value {
    json!({
        "kty": "RSA",
        "n": base64_url(&key.n().to_bytes_be()),
        "e": base64_url(&key.e().to_bytes_be()),
    })
}

pub(crate) fn jwk_thumbprint(jwk: &Value) -> Result<String, String> {
    let n = jwk
        .get("n")
        .and_then(Value::as_str)
        .ok_or_else(|| "RSA JWK 缺少 n".to_string())?;
    let e = jwk
        .get("e")
        .and_then(Value::as_str)
        .ok_or_else(|| "RSA JWK 缺少 e".to_string())?;
    let canonical = format!(r#"{{"e":"{e}","kty":"RSA","n":"{n}"}}"#);
    Ok(base64_url(&Sha256::digest(canonical.as_bytes())))
}

pub(crate) fn dns01_txt_value(token: &str, account_thumbprint: &str) -> String {
    let key_authorization = format!("{token}.{account_thumbprint}");
    base64_url(&Sha256::digest(key_authorization.as_bytes()))
}

pub(crate) fn sign_rs256(key: &RsaPrivateKey, data: &[u8]) -> Result<Vec<u8>, String> {
    let signing_key = SigningKey::<Sha256>::new(key.clone());
    Ok(signing_key.sign(data).to_bytes().to_vec())
}

pub(crate) fn base64_url(bytes: &[u8]) -> String {
    Base64UrlUnpadded::encode_string(bytes)
}

pub(crate) fn build_csr_der(domain: &str, key: &RsaPrivateKey) -> Result<Vec<u8>, String> {
    let certification_request_info = build_certification_request_info_der(domain, key)?;
    let signature = sign_rs256(key, &certification_request_info)?;
    Ok(der_sequence(vec![
        certification_request_info,
        sha256_with_rsa_algorithm_identifier(),
        der_bit_string(&signature),
    ]))
}

fn build_certification_request_info_der(
    domain: &str,
    key: &RsaPrivateKey,
) -> Result<Vec<u8>, String> {
    Ok(der_sequence(vec![
        der_integer_u64(0),
        subject_name_der(domain)?,
        subject_public_key_info_der(key),
        der_context_constructed(0, extension_request_attribute_der(domain)?),
    ]))
}

fn subject_name_der(domain: &str) -> Result<Vec<u8>, String> {
    Ok(der_sequence(vec![der_set(vec![der_sequence(vec![
        der_oid("2.5.4.3")?,
        der_utf8_string(domain),
    ])])]))
}

fn subject_public_key_info_der(key: &RsaPrivateKey) -> Vec<u8> {
    let rsa_public_key = der_sequence(vec![
        der_integer_bytes(&key.n().to_bytes_be()),
        der_integer_bytes(&key.e().to_bytes_be()),
    ]);
    der_sequence(vec![
        rsa_encryption_algorithm_identifier(),
        der_bit_string(&rsa_public_key),
    ])
}

fn extension_request_attribute_der(domain: &str) -> Result<Vec<u8>, String> {
    let general_names = der_sequence(vec![der_context_primitive(2, domain.as_bytes())]);
    let san_extension = der_sequence(vec![
        der_oid("2.5.29.17")?,
        der_octet_string(&general_names),
    ]);
    let extensions = der_sequence(vec![san_extension]);
    Ok(der_sequence(vec![
        der_oid("1.2.840.113549.1.9.14")?,
        der_set(vec![extensions]),
    ]))
}

fn rsa_encryption_algorithm_identifier() -> Vec<u8> {
    der_sequence(vec![
        der_oid("1.2.840.113549.1.1.1").expect("固定 OID 有效"),
        der_null(),
    ])
}

fn sha256_with_rsa_algorithm_identifier() -> Vec<u8> {
    der_sequence(vec![
        der_oid("1.2.840.113549.1.1.11").expect("固定 OID 有效"),
        der_null(),
    ])
}

fn der_sequence(parts: Vec<Vec<u8>>) -> Vec<u8> {
    der_tag(0x30, concat_der(parts))
}

fn der_set(parts: Vec<Vec<u8>>) -> Vec<u8> {
    der_tag(0x31, concat_der(parts))
}

fn der_null() -> Vec<u8> {
    der_tag(0x05, Vec::new())
}

fn der_integer_u64(value: u64) -> Vec<u8> {
    if value == 0 {
        return der_tag(0x02, vec![0]);
    }
    let mut bytes = value.to_be_bytes().to_vec();
    while bytes.first() == Some(&0) {
        bytes.remove(0);
    }
    der_integer_bytes(&bytes)
}

fn der_integer_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut content = if bytes.is_empty() {
        vec![0]
    } else {
        bytes.to_vec()
    };
    if content.first().is_some_and(|byte| byte & 0x80 != 0) {
        content.insert(0, 0);
    }
    der_tag(0x02, content)
}

fn der_oid(value: &str) -> Result<Vec<u8>, String> {
    let numbers = value
        .split('.')
        .map(|part| {
            part.parse::<u32>()
                .map_err(|error| format!("OID {value} 包含非法数字：{error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if numbers.len() < 2 || numbers[0] > 2 || numbers[1] > 39 {
        return Err(format!("OID {value} 格式无效"));
    }
    let mut content = vec![(numbers[0] * 40 + numbers[1]) as u8];
    for number in numbers.into_iter().skip(2) {
        encode_base128(number, &mut content);
    }
    Ok(der_tag(0x06, content))
}

fn der_octet_string(value: &[u8]) -> Vec<u8> {
    der_tag(0x04, value.to_vec())
}

fn der_bit_string(value: &[u8]) -> Vec<u8> {
    let mut content = Vec::with_capacity(value.len() + 1);
    content.push(0);
    content.extend_from_slice(value);
    der_tag(0x03, content)
}

fn der_utf8_string(value: &str) -> Vec<u8> {
    der_tag(0x0c, value.as_bytes().to_vec())
}

fn der_context_primitive(tag_number: u8, value: &[u8]) -> Vec<u8> {
    der_tag(0x80 | tag_number, value.to_vec())
}

fn der_context_constructed(tag_number: u8, value: Vec<u8>) -> Vec<u8> {
    der_tag(0xa0 | tag_number, value)
}

fn der_tag(tag: u8, content: Vec<u8>) -> Vec<u8> {
    let mut output = Vec::with_capacity(content.len() + 8);
    output.push(tag);
    output.extend_from_slice(&der_length(content.len()));
    output.extend_from_slice(&content);
    output
}

fn der_length(length: usize) -> Vec<u8> {
    if length < 128 {
        return vec![length as u8];
    }
    let mut bytes = Vec::new();
    let mut value = length;
    while value > 0 {
        bytes.push((value & 0xff) as u8);
        value >>= 8;
    }
    bytes.reverse();
    let mut output = vec![0x80 | bytes.len() as u8];
    output.extend(bytes);
    output
}

fn concat_der(parts: Vec<Vec<u8>>) -> Vec<u8> {
    let total_len = parts.iter().map(Vec::len).sum();
    let mut output = Vec::with_capacity(total_len);
    for part in parts {
        output.extend(part);
    }
    output
}

fn encode_base128(mut value: u32, output: &mut Vec<u8>) {
    let mut stack = vec![(value & 0x7f) as u8];
    value >>= 7;
    while value > 0 {
        stack.push(((value & 0x7f) as u8) | 0x80);
        value >>= 7;
    }
    output.extend(stack.into_iter().rev());
}
