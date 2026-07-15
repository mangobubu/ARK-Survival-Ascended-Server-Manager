use super::support::{required_arg, to_json};
use crate::{
    app_state::AppRuntime, curseforge, instance_config_commands, instance_data_commands,
    models::ModItem, server_lifecycle::apply_instance_config_and_restart_for_runtime,
    server_version::with_current_server_version,
};
use serde_json::Value;
use tauri::AppHandle;

pub(super) fn read_server_directory_config(
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(
        instance_data_commands::read_server_directory_config_for_runtime(
            runtime,
            required_arg(args, "path")?,
        )?,
    )
}

pub(super) fn list_host_directories(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(instance_data_commands::list_host_directories_for_runtime(
        runtime,
        super::support::optional_arg(args, "path")?,
    )?)
}

pub(super) fn get_instance_config(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(instance_config_commands::get_instance_config_for_runtime(
        runtime,
        &required_arg::<String>(args, "instanceId")?,
    )?)
}

pub(super) fn get_instance_mods(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(instance_config_commands::get_instance_mods_for_runtime(
        runtime,
        &required_arg::<String>(args, "instanceId")?,
    )?)
}

pub(super) fn save_instance_config(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let instance_id: String = required_arg(args, "instanceId")?;
    let config: Value = required_arg(args, "config")?;
    let mods: Vec<ModItem> = required_arg(args, "mods")?;
    to_json(with_current_server_version(
        instance_config_commands::save_config_with_operation_for_runtime(
            app,
            runtime,
            &instance_id,
            config,
            mods,
        )?,
    ))
}

pub(super) async fn apply_instance_config(
    app: AppHandle,
    runtime: AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let instance_id: String = required_arg(args, "instanceId")?;
    let config: Value = required_arg(args, "config")?;
    let mods: Vec<ModItem> = required_arg(args, "mods")?;
    to_json(
        apply_instance_config_and_restart_for_runtime(app, runtime, instance_id, config, mods)
            .await?,
    )
}

pub(super) fn update_instance_mods(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let instance_id: String = required_arg(args, "instanceId")?;
    let mods: Vec<ModItem> = required_arg(args, "mods")?;
    to_json(
        instance_config_commands::update_instance_mods_with_operation_for_runtime(
            app,
            runtime,
            &instance_id,
            mods,
        )?,
    )
}

pub(super) fn check_mod_updates(args: &Value) -> Result<Value, String> {
    to_json(instance_config_commands::check_mod_updates_for_runtime(
        required_arg(args, "mods")?,
    )?)
}

pub(super) async fn search_curseforge_mods(
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(
        curseforge::search_curseforge_mods_for_runtime(
            runtime,
            required_arg(args, "query")?,
            required_arg(args, "index")?,
            required_arg(args, "pageSize")?,
        )
        .await?,
    )
}
