use crate::{instance_config_import_ini::IniDocument, models::ModItem};
use serde_json::{Map, Value, json};
use std::collections::HashSet;

pub(crate) fn parse_item_stack_override(value: &str) -> Option<Value> {
    let item_class_string = extract_assignment(value, "ItemClassString")?;
    let max_item_quantity = extract_u32_assignment(value, "MaxItemQuantity").unwrap_or(1);
    let ignore_multiplier = extract_bool_assignment(value, "bIgnoreMultiplier").unwrap_or(true);

    Some(json!({
        "itemClassString": item_class_string,
        "maxItemQuantity": max_item_quantity,
        "ignoreMultiplier": ignore_multiplier,
    }))
}

fn extract_assignment(value: &str, key: &str) -> Option<String> {
    let start = value.find(key)? + key.len();
    let rest = value[start..].trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();

    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        let text = rest[..end].trim();
        return (!text.is_empty()).then(|| text.to_string());
    }

    let end = rest.find([',', ')', '(']).unwrap_or(rest.len());
    let text = rest[..end].trim().trim_matches('"');
    (!text.is_empty()).then(|| text.to_string())
}

fn extract_u32_assignment(value: &str, key: &str) -> Option<u32> {
    extract_assignment(value, key)?
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
}

fn extract_bool_assignment(value: &str, key: &str) -> Option<bool> {
    match extract_assignment(value, key)?
        .to_ascii_lowercase()
        .as_str()
    {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub(crate) fn map_text(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) -> Option<String> {
    let value = document.get(sections, ini_key)?.to_string();
    config.insert(config_key.to_string(), json!(value));
    Some(value)
}

pub(crate) fn map_bool(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_bool) {
        config.insert(config_key.to_string(), json!(value));
    }
}

pub(crate) fn map_bool_inverted(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_bool) {
        config.insert(config_key.to_string(), json!(!value));
    }
}

pub(crate) fn map_u16(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_u16) {
        config.insert(config_key.to_string(), json!(value));
    }
}

pub(crate) fn map_u32(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_u32) {
        config.insert(config_key.to_string(), json!(value));
    }
}

pub(crate) fn map_f64(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_f64) {
        config.insert(config_key.to_string(), json!(value));
    }
}

pub(crate) fn parse_active_mods(value: &str) -> Vec<ModItem> {
    let mut seen = HashSet::new();
    value
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty() && id.chars().all(|ch| ch.is_ascii_digit()))
        .filter(|id| seen.insert((*id).to_string()))
        .map(|id| ModItem {
            id: id.to_string(),
            name: format!("MOD {id}"),
            version: "配置导入".to_string(),
            size: "未知大小".to_string(),
            enabled: true,
            update_available: Some(false),
        })
        .collect()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_u16(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok()
}

fn parse_u32(value: &str) -> Option<u32> {
    let value = value.trim();
    value.parse::<u32>().ok().or_else(|| {
        let number = value.parse::<f64>().ok()?;
        (number.is_finite() && number >= 0.0 && number <= u32::MAX as f64 && number.fract() == 0.0)
            .then_some(number as u32)
    })
}

fn parse_f64(value: &str) -> Option<f64> {
    value.trim().parse::<f64>().ok()
}

pub(crate) fn text_from_config(config: &Map<String, Value>, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub(crate) fn bool_from_config(config: &Map<String, Value>, key: &str) -> Option<bool> {
    config.get(key).and_then(Value::as_bool)
}

pub(crate) fn u16_from_config(config: &Map<String, Value>, key: &str) -> Option<u16> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
}

pub(crate) fn u32_from_config(config: &Map<String, Value>, key: &str) -> Option<u32> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}
