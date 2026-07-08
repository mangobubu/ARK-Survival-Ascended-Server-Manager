use super::support::required_arg;
use crate::{app_state::AppRuntime, commands::open_directory, path_security};
use serde_json::Value;
use std::path::Path;

pub(super) fn open_instance_directory(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    let instance = runtime.get_instance(&required_arg::<String>(args, "instanceId")?)?;
    let path = Path::new(&instance.install_path);
    if !path.exists() {
        return Err(format!("实例目录不存在：{}", path.display()));
    }
    let path = path_security::ensure_managed_path_allowed(runtime, path)?;
    open_directory(&path)?;
    Ok(Value::Null)
}

pub(super) fn open_directory_path(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    let path_text: String = required_arg(args, "path")?;
    let path = Path::new(&path_text);
    let path = path_security::ensure_managed_path_allowed(runtime, path)?;
    if !path.exists() {
        return Err(format!("目录不存在：{}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("不是可打开的目录：{}", path.display()));
    }
    open_directory(&path)?;
    Ok(Value::Null)
}
