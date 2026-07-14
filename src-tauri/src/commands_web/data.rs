use super::support::{required_arg, to_json};
use crate::{app_state::AppRuntime, instance_data_commands};
use serde_json::Value;
use tauri::AppHandle;

const MAX_INSTANCE_CONFIG_UPLOAD_BYTES: usize = 8 * 1024 * 1024;

pub(super) fn create_backup(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(instance_data_commands::create_backup_for_runtime(
        app,
        runtime,
        required_arg(args, "instanceId")?,
    )?)
}

pub(super) fn list_backups(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(instance_data_commands::list_backups_for_runtime(
        runtime,
        required_arg(args, "instanceId")?,
    )?)
}

pub(super) fn restore_backup(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    instance_data_commands::restore_backup_for_runtime(
        app,
        runtime,
        required_arg(args, "instanceId")?,
        required_arg(args, "backupPath")?,
    )?;
    Ok(Value::Null)
}

pub(super) fn export_instance_config_for_download(
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(
        instance_data_commands::export_instance_config_for_web_transfer(
            runtime,
            required_arg(args, "instanceIds")?,
        )?,
    )
}

pub(super) fn export_cluster_for_download(runtime: &AppRuntime) -> Result<Value, String> {
    to_json(instance_data_commands::export_cluster_for_web_transfer(
        runtime,
    )?)
}

pub(super) fn import_instance_config_upload(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let file_name: String = required_arg(args, "fileName")?;
    if !file_name
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("json"))
    {
        return Err("Web 导入仅支持 .json 格式的 ASA 实例导出文件".to_string());
    }

    let content: String = required_arg(args, "content")?;
    if content.len() > MAX_INSTANCE_CONFIG_UPLOAD_BYTES {
        return Err(format!(
            "Web 导入文件不能超过 {} MB",
            MAX_INSTANCE_CONFIG_UPLOAD_BYTES / 1024 / 1024
        ));
    }

    to_json(
        instance_data_commands::import_instance_config_content_for_runtime(app, runtime, &content)?,
    )
}
