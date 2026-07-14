use super::support::{optional_arg, required_arg, to_json};
use crate::{app_state::AppRuntime, instance_file_manager};
use serde_json::Value;

pub(super) fn list_instance_files(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(instance_file_manager::list_instance_directory(
        runtime,
        &required_arg::<String>(args, "instanceId")?,
        optional_arg::<String>(args, "path")?.as_deref(),
    )?)
}

pub(super) fn create_instance_file_entry(
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(instance_file_manager::create_entry(
        runtime,
        &required_arg::<String>(args, "instanceId")?,
        &required_arg::<String>(args, "parentPath")?,
        &required_arg::<String>(args, "name")?,
        &required_arg::<String>(args, "entryType")?,
    )?)
}

pub(super) fn rename_instance_file_entry(
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(instance_file_manager::rename_entry(
        runtime,
        &required_arg::<String>(args, "instanceId")?,
        &required_arg::<String>(args, "path")?,
        &required_arg::<String>(args, "newName")?,
    )?)
}

pub(super) fn copy_instance_file_entry(
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(instance_file_manager::copy_entry(
        runtime,
        &required_arg::<String>(args, "instanceId")?,
        &required_arg::<String>(args, "sourcePath")?,
        &required_arg::<String>(args, "targetDirectory")?,
    )?)
}

pub(super) fn delete_instance_file_entry(
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    instance_file_manager::delete_entry(
        runtime,
        &required_arg::<String>(args, "instanceId")?,
        &required_arg::<String>(args, "path")?,
    )?;
    Ok(Value::Null)
}
