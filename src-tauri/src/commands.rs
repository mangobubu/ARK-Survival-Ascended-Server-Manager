mod install_update;
mod open_paths;

pub(crate) use install_update::install_or_update_instance_inner;
pub(crate) use open_paths::open_directory;

use crate::{
    app_state::AppRuntime,
    ark_config,
    command_events::{
        emit_instance_log, publish_instances_changed, publish_sync_event_best_effort,
    },
    instance_config_commands,
    models::{
        AddInstancePayload, JobProgress, LogLine, LogSource, ModItem, ServerInstance, ServerLogKind,
    },
    path_security,
    server_lifecycle::{
        refresh_status_for_runtime, restart_instance_for_runtime, start_instance_for_runtime,
        stop_instance_for_runtime,
    },
    server_rcon,
    server_version::with_current_server_version,
    sync_events::{
        ADD_INSTANCE_CREATED_EVENT, INSTANCE_DELETED_EVENT, LOGS_CLEARED_EVENT, LOGS_RESET_EVENT,
    },
    windows_firewall,
};
use serde_json::{Value, json};
use std::path::Path;
use tauri::{AppHandle, State, ipc::Channel};

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogClearScope {
    pub(crate) source: LogSource,
    pub(crate) instance: Option<String>,
    pub(crate) server_log_kind: Option<ServerLogKind>,
}

#[tauri::command]
pub fn create_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    payload: AddInstancePayload,
) -> Result<ServerInstance, String> {
    let auto_install = payload.auto_install;
    let instance = runtime
        .create_instance(payload)
        .map(with_current_server_version)?;
    emit_instance_created_log(&app, runtime.inner(), &instance)?;
    let firewall_rules = windows_firewall::ensure_instance_firewall_rules(&instance)?;
    if !firewall_rules.is_empty() {
        let _ = emit_instance_log(
            &app,
            runtime.inner(),
            &instance.name,
            "success",
            &format!(
                "Windows 防火墙规则已确认：{}",
                windows_firewall::format_rule_summaries(&firewall_rules)
            ),
        );
    }
    publish_sync_event_best_effort(
        &app,
        ADD_INSTANCE_CREATED_EVENT,
        json!({
            "instance": instance,
            "autoInstall": auto_install,
        }),
    );
    publish_instances_changed(&app);
    Ok(instance)
}

#[tauri::command]
pub async fn apply_instance_config(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    config: Value,
    mods: Vec<ModItem>,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    instance_config_commands::save_config_for_runtime(&app, &runtime, &instance_id, config, mods)?;
    restart_instance_for_runtime(app, runtime, instance_id).await
}

#[tauri::command]
pub async fn install_or_update_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    progress: Channel<JobProgress>,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    install_or_update_instance_inner(app, runtime, instance_id, progress).await
}

#[tauri::command]
pub async fn start_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    start_instance_for_runtime(app, runtime, instance_id).await
}

#[tauri::command]
pub async fn stop_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    stop_instance_for_runtime(app, runtime, instance_id)
}

#[tauri::command]
pub async fn restart_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    restart_instance_for_runtime(app, runtime, instance_id).await
}

#[tauri::command]
pub async fn refresh_instance_status(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    refresh_status_for_runtime(&app, &runtime, &instance_id).await
}

#[tauri::command]
pub async fn execute_rcon_command(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    command: String,
) -> Result<String, String> {
    let runtime = runtime.inner().clone();
    server_rcon::execute_rcon_command(&app, &runtime, instance_id, command).await
}

#[tauri::command]
pub fn query_logs(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<LogLine>, String> {
    runtime.query_logs(limit)
}

#[tauri::command]
pub fn clear_logs(app: AppHandle, runtime: State<'_, AppRuntime>) -> Result<(), String> {
    runtime.clear_logs()?;
    publish_sync_event_best_effort(&app, LOGS_RESET_EVENT, json!({}));
    Ok(())
}

#[tauri::command]
pub fn clear_scoped_logs(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    source: LogSource,
    instance: Option<String>,
    server_log_kind: Option<ServerLogKind>,
) -> Result<(), String> {
    runtime.clear_logs_by_scope(source.clone(), instance.as_deref(), server_log_kind.clone())?;
    publish_sync_event_best_effort(
        &app,
        LOGS_CLEARED_EVENT,
        LogClearScope {
            source,
            instance,
            server_log_kind,
        },
    );
    Ok(())
}

#[tauri::command]
pub fn delete_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let removed = runtime.delete_instance(&instance_id)?;
    emit_instance_log(
        &app,
        runtime.inner(),
        &removed.name,
        "warn",
        &format!("已删除实例记录，实例文件仍保留在：{}", removed.install_path),
    )?;
    publish_sync_event_best_effort(&app, INSTANCE_DELETED_EVENT, removed.clone());
    publish_instances_changed(&app);
    Ok(removed)
}

pub(crate) fn emit_instance_created_log(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance: &ServerInstance,
) -> Result<(), String> {
    let config_dir = ark_config::config_dir(instance);
    emit_instance_log(
        app,
        runtime,
        &instance.name,
        "success",
        &format!(
            "已创建服务器实例，初始 ARK 配置已写入：{}、{}、{}",
            config_dir.join("GameUserSettings.ini").to_string_lossy(),
            config_dir.join("Game.ini").to_string_lossy(),
            config_dir.join("Engine.ini").to_string_lossy()
        ),
    )
}

#[tauri::command]
pub fn open_instance_directory(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<(), String> {
    let instance = runtime.get_instance(&instance_id)?;
    let path = Path::new(&instance.install_path);
    if !path.exists() {
        return Err(format!("实例目录不存在：{}", path.display()));
    }
    let path = path_security::ensure_managed_path_allowed(&runtime, path)?;
    open_directory(&path)
}

#[tauri::command]
pub fn open_directory_path(runtime: State<'_, AppRuntime>, path: String) -> Result<(), String> {
    let path = Path::new(&path);
    let path = path_security::ensure_managed_path_allowed(&runtime, path)?;
    if !path.exists() {
        return Err(format!("目录不存在：{}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("不是可打开的目录：{}", path.display()));
    }
    open_directory(&path)
}

pub async fn handle_web_invoke(
    app: AppHandle,
    runtime: AppRuntime,
    command: String,
    args: Value,
    high_risk_confirmed: bool,
) -> Result<Value, String> {
    crate::commands_web::handle_web_invoke(app, runtime, command, args, high_risk_confirmed).await
}
#[cfg(test)]
mod tests;
