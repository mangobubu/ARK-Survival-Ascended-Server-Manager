use crate::{
    app_state::AppRuntime,
    models::{PortCheckResult, ServerInstance},
    server_version::with_current_server_version,
    sync_events::{INSTANCE_STATUS_EVENT, INSTANCES_CHANGED_EVENT},
    window_controls,
};
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, State};

pub(crate) fn list_instances_for_runtime(
    runtime: &AppRuntime,
) -> Result<Vec<ServerInstance>, String> {
    Ok(runtime
        .list_instances()?
        .into_iter()
        .map(with_current_server_version)
        .collect())
}

pub(crate) fn clear_startup_auto_update_skip_flags_for_runtime(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_ids: Vec<String>,
) -> Result<Vec<ServerInstance>, String> {
    let updated = runtime.clear_startup_auto_update_skip_flags(&instance_ids)?;
    for instance in &updated {
        publish_sync_event_best_effort(
            app,
            INSTANCE_STATUS_EVENT,
            with_current_server_version(instance.clone()),
        );
    }
    if !updated.is_empty() {
        publish_instances_changed(app);
    }
    Ok(updated
        .into_iter()
        .map(with_current_server_version)
        .collect())
}

pub(crate) fn check_instance_port_for_runtime(
    runtime: &AppRuntime,
    port: u16,
    port_kind: &str,
) -> Result<PortCheckResult, String> {
    runtime.check_instance_port(port, port_kind)
}

#[tauri::command]
pub fn list_instances(runtime: State<'_, AppRuntime>) -> Result<Vec<ServerInstance>, String> {
    list_instances_for_runtime(runtime.inner())
}

#[tauri::command]
pub fn clear_startup_auto_update_skip_flags(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_ids: Vec<String>,
) -> Result<Vec<ServerInstance>, String> {
    clear_startup_auto_update_skip_flags_for_runtime(&app, runtime.inner(), instance_ids)
}

#[tauri::command]
pub fn check_instance_port(
    runtime: State<'_, AppRuntime>,
    port: u16,
    port_kind: String,
) -> Result<PortCheckResult, String> {
    check_instance_port_for_runtime(runtime.inner(), port, &port_kind)
}

fn publish_instances_changed(app: &AppHandle) {
    publish_sync_event_best_effort(app, INSTANCES_CHANGED_EVENT, json!({}));
}

fn publish_sync_event<T: Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: T,
) -> Result<(), String> {
    window_controls::publish_settings_changed_and_apply(app, event_name, payload)
}

fn publish_sync_event_best_effort<T: Serialize>(app: &AppHandle, event_name: &str, payload: T) {
    let _ = publish_sync_event(app, event_name, payload);
}
