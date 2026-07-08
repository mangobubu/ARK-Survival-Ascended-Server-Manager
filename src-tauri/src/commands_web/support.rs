use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tauri::ipc::Channel;

pub(super) fn required_arg<T: DeserializeOwned>(args: &Value, name: &str) -> Result<T, String> {
    let value = args
        .get(name)
        .ok_or_else(|| format!("缺少参数：{name}"))?
        .clone();
    serde_json::from_value(value).map_err(|error| format!("参数 {name} 格式无效：{error}"))
}

pub(super) fn optional_arg<T: DeserializeOwned>(
    args: &Value,
    name: &str,
) -> Result<Option<T>, String> {
    match args.get(name) {
        Some(Value::Null) | None => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|error| format!("参数 {name} 格式无效：{error}")),
    }
}

pub(super) fn to_json<T: Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| format!("序列化 Web API 响应失败：{error}"))
}

pub(super) fn no_op_channel<T>() -> Channel<T> {
    Channel::new(|_| Ok(()))
}
