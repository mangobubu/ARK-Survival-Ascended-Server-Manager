use serde_json::Value;

pub(crate) fn text(config: &Value, key: &str, fallback: &str) -> String {
    clean_config_text(config.get(key).and_then(Value::as_str).unwrap_or(fallback))
}

pub(crate) fn clean_config_text(value: &str) -> String {
    value
        .replace(['\r', '\n'], " ")
        .replace(['\0', '[', ']'], "")
        .trim()
        .to_string()
}

pub(crate) fn bool_value(config: &Value, key: &str, fallback: bool) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}

pub(crate) fn number_u16(config: &Value, key: &str, fallback: u16) -> u16 {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(fallback)
}

pub(crate) fn number_u32(config: &Value, key: &str, fallback: u32) -> u32 {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(fallback)
}

pub(crate) fn number_f64(config: &Value, key: &str, fallback: f64) -> f64 {
    config.get(key).and_then(Value::as_f64).unwrap_or(fallback)
}

pub(crate) fn ini_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

pub(crate) fn url_component(value: &str) -> String {
    value
        .replace('?', "%3F")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace(' ', "%20")
        .replace('"', "%22")
}

pub fn normalized_visibility(config: &Value) -> &'static str {
    match config
        .get("visibility")
        .and_then(Value::as_str)
        .unwrap_or("public")
    {
        "private" => "private",
        _ if text(config, "serverPassword", "").trim().is_empty() => "public",
        _ => "private",
    }
}

pub fn validate_visibility_access(config: &Value) -> Result<(), String> {
    let password_configured = !text(config, "serverPassword", "").trim().is_empty();
    let exclusive_access =
        bool_value(config, "whitelist", false) || bool_value(config, "exclusiveJoin", false);

    if normalized_visibility(config) == "private" && !password_configured && !exclusive_access {
        return Err(
            "私有服务器必须设置加入密码，或启用白名单 / Exclusive Join 准入控制".to_string(),
        );
    }
    Ok(())
}

pub(crate) fn split_custom_args(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter(|part| !part.trim().is_empty())
        .map(|part| part.trim().to_string())
        .collect()
}
