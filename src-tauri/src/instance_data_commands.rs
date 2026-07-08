use crate::{
    app_state::AppRuntime,
    backup, import_export, instance_config_import,
    models::{BackupItem, ExportResult, ImportResult, ImportedServerConfigPreview},
    path_security,
    sync_events::INSTANCES_CHANGED_EVENT,
    window_controls,
};
use serde_json::json;
use std::path::Path;
use tauri::{AppHandle, State};

pub(crate) fn read_server_directory_config_for_runtime(
    runtime: &AppRuntime,
    path: String,
) -> Result<ImportedServerConfigPreview, String> {
    let path = path_security::ensure_managed_path_allowed(runtime, Path::new(&path))?;
    instance_config_import::read_server_directory_config(&path)
}

pub(crate) fn create_backup_for_runtime(
    runtime: &AppRuntime,
    instance_id: String,
) -> Result<BackupItem, String> {
    let settings = runtime.settings()?;
    let instance = runtime.get_instance(&instance_id)?;
    let backup_root = Path::new(&settings.backup_storage_path);
    let backup = backup::create_instance_backup(backup_root, &instance)?;
    let pruned_count =
        backup::prune_instance_backups(backup_root, &instance, settings.max_backup_retention)?;
    runtime.add_log(
        &instance.name,
        "success",
        &format!(
            "备份已创建：{}{}",
            backup.path,
            if pruned_count > 0 {
                format!("；已按全局保留数量清理 {pruned_count} 个旧备份")
            } else {
                String::new()
            }
        ),
    )?;
    Ok(backup)
}

pub(crate) fn list_backups_for_runtime(
    runtime: &AppRuntime,
    instance_id: String,
) -> Result<Vec<BackupItem>, String> {
    let settings = runtime.settings()?;
    let instance = runtime.get_instance(&instance_id)?;
    backup::list_instance_backups(Path::new(&settings.backup_storage_path), &instance)
}

pub(crate) fn restore_backup_for_runtime(
    runtime: &AppRuntime,
    instance_id: String,
    backup_path: String,
) -> Result<(), String> {
    let instance = runtime.get_instance(&instance_id)?;
    let backup_path = path_security::ensure_backup_path_allowed(runtime, Path::new(&backup_path))?;
    backup::restore_instance_backup(&instance, &backup_path)?;
    runtime.add_log(
        &instance.name,
        "warn",
        &format!("已恢复备份：{}", backup_path.display()),
    )?;
    Ok(())
}

pub(crate) fn export_instance_config_for_runtime(
    runtime: &AppRuntime,
    instance_ids: Vec<String>,
) -> Result<ExportResult, String> {
    import_export::export_instances(runtime, instance_ids)
}

pub(crate) fn export_cluster_for_runtime(runtime: &AppRuntime) -> Result<ExportResult, String> {
    import_export::export_instances(runtime, Vec::new())
}

pub(crate) fn import_instance_config_for_runtime(
    app: &AppHandle,
    runtime: &AppRuntime,
    path: String,
) -> Result<ImportResult, String> {
    let path = path_security::ensure_managed_path_allowed(runtime, Path::new(&path))?;
    let result = import_export::import_instances(runtime, &path)?;
    if result.imported_instances > 0 {
        publish_instances_changed(app);
    }
    Ok(result)
}

#[tauri::command]
pub fn read_server_directory_config(
    runtime: State<'_, AppRuntime>,
    path: String,
) -> Result<ImportedServerConfigPreview, String> {
    read_server_directory_config_for_runtime(runtime.inner(), path)
}

#[tauri::command]
pub fn create_backup(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<BackupItem, String> {
    create_backup_for_runtime(runtime.inner(), instance_id)
}

#[tauri::command]
pub fn list_backups(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<Vec<BackupItem>, String> {
    list_backups_for_runtime(runtime.inner(), instance_id)
}

#[tauri::command]
pub fn restore_backup(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    backup_path: String,
) -> Result<(), String> {
    restore_backup_for_runtime(runtime.inner(), instance_id, backup_path)
}

#[tauri::command]
pub fn export_instance_config(
    runtime: State<'_, AppRuntime>,
    instance_ids: Vec<String>,
) -> Result<ExportResult, String> {
    export_instance_config_for_runtime(runtime.inner(), instance_ids)
}

#[tauri::command]
pub fn export_cluster(runtime: State<'_, AppRuntime>) -> Result<ExportResult, String> {
    export_cluster_for_runtime(runtime.inner())
}

#[tauri::command]
pub fn import_instance_config(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    path: String,
) -> Result<ImportResult, String> {
    import_instance_config_for_runtime(&app, runtime.inner(), path)
}

fn publish_instances_changed(app: &AppHandle) {
    let _ = window_controls::publish_settings_changed_and_apply(
        app,
        INSTANCES_CHANGED_EVENT,
        json!({}),
    );
}
