use serde_json::Value;

pub(crate) fn bool_from_config(config: &Value, key: &str, fallback: bool) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}
