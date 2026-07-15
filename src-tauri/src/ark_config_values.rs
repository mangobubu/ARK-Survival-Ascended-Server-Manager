use serde_json::Value;
use std::collections::HashSet;

pub(crate) const CUSTOM_INI_MARKER_PREFIX: &str = "; ASA-SERVER-MANAGER:CUSTOM-INI:";
pub(crate) const CUSTOM_INI_BEGIN_SUFFIX: &str = ":BEGIN";
pub(crate) const CUSTOM_INI_END_SUFFIX: &str = ":END";

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

pub(crate) fn append_custom_ini_settings(
    lines: &mut Vec<String>,
    config: &Value,
    config_key: &str,
) {
    let managed_keys = lines
        .iter()
        .filter_map(|line| line.split_once('=').map(|(key, _)| normalize_ini_key(key)))
        .filter(|key| !key.is_empty())
        .collect::<HashSet<_>>();
    lines.push(format!(
        "{CUSTOM_INI_MARKER_PREFIX}{config_key}{CUSTOM_INI_BEGIN_SUFFIX}"
    ));

    if let Some(custom) = config.get(config_key).and_then(Value::as_str) {
        for raw_line in custom.lines() {
            let line = raw_line.trim().trim_start_matches('\u{feff}');
            if line.is_empty() || line.starts_with([';', '#', '[']) || line.contains('\0') {
                continue;
            }
            let Some((raw_key, raw_value)) = line.split_once('=') else {
                continue;
            };
            let key = raw_key.trim();
            let normalized_key = normalize_ini_key(key);
            if normalized_key.is_empty()
                || managed_keys.contains(&normalized_key)
                || key.chars().any(char::is_control)
            {
                continue;
            }
            let value = raw_value.replace(['\r', '\n', '\0'], "").trim().to_string();
            lines.push(format!("{key}={value}"));
        }
    }

    lines.push(format!(
        "{CUSTOM_INI_MARKER_PREFIX}{config_key}{CUSTOM_INI_END_SUFFIX}"
    ));
}

fn normalize_ini_key(value: &str) -> String {
    value
        .trim()
        .trim_start_matches(['+', '-'])
        .trim()
        .to_ascii_lowercase()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn 自定义_ini_仅追加安全赋值并跳过托管键冲突() {
        let mut lines = vec!["[ServerSettings]".to_string(), "Known=True".to_string()];
        let config = json!({
            "custom": "Known=False\n+Known=False\nCustomArray=(A=1,B=2)\nCustomArray=(A=3,B=4)\nPerLevelStatsMultiplier_Player[0]=2.5\n[Injected]\nNoEquals\nAfter=Safe"
        });

        append_custom_ini_settings(&mut lines, &config, "custom");

        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("Known="))
                .count(),
            1
        );
        assert!(!lines.iter().any(|line| line.starts_with("+Known=")));
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("CustomArray="))
                .count(),
            2
        );
        assert!(!lines.iter().any(|line| line == "[Injected]"));
        assert!(
            lines
                .iter()
                .any(|line| line == "PerLevelStatsMultiplier_Player[0]=2.5")
        );
        assert!(lines.iter().any(|line| line == "After=Safe"));
    }
}
