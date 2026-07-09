use super::support::{required_arg, to_json};
use crate::{app_state::AppRuntime, instance_data_commands};
use serde_json::Value;
use tauri::AppHandle;

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

pub(super) fn export_instance_config(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(instance_data_commands::export_instance_config_for_runtime(
        runtime,
        required_arg(args, "instanceIds")?,
    )?)
}

pub(super) fn export_cluster(runtime: &AppRuntime) -> Result<Value, String> {
    to_json(instance_data_commands::export_cluster_for_runtime(runtime)?)
}

pub(super) fn import_instance_config(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(instance_data_commands::import_instance_config_for_runtime(
        app,
        runtime,
        required_arg(args, "path")?,
    )?)
}
