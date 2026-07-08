use crate::{
    app_state::{AppRuntime, normalize_required_rcon_config},
    command_events::{emit_instance_log, emit_progress, emit_status},
    instance_config_commands,
    models::{JobProgress, ServerInstance, ServerStatus},
    server_version::{read_installed_server_version, with_current_server_version},
    steamcmd_update_runner::{UPDATE_CANCELLED_MESSAGE, run_steamcmd_update_with_retry},
};
use std::{path::Path, sync::Arc};
use tauri::{AppHandle, ipc::Channel};

pub(crate) async fn install_or_update_instance_inner(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
    progress: Channel<JobProgress>,
) -> Result<ServerInstance, String> {
    let mut instance = runtime.get_instance(&instance_id)?;
    let settings = runtime.settings()?;
    let steamcmd = Path::new(&settings.steam_cmd_path).join("steamcmd.exe");
    if !steamcmd.is_file() {
        let error = format!("SteamCMD 不存在：{}", steamcmd.display());
        let _ =
            runtime.update_instance_status(&instance.id, ServerStatus::Error, Some(error.clone()));
        let _ = emit_instance_log(&app, &runtime, &instance.name, "error", &error);
        let _ = emit_status(&app, &runtime, &instance.id);
        return Err(error);
    }
    if let Err(error) = std::fs::create_dir_all(&instance.install_path) {
        let error = format!("无法创建实例目录 {}：{error}", instance.install_path);
        let _ =
            runtime.update_instance_status(&instance.id, ServerStatus::Error, Some(error.clone()));
        let _ = emit_instance_log(&app, &runtime, &instance.name, "error", &error);
        let _ = emit_status(&app, &runtime, &instance.id);
        return Err(error);
    }
    let update_cancel = runtime.begin_update(&instance.id)?;

    emit_progress(
        &app,
        &progress,
        &instance.id,
        "preparing",
        Some(5.0),
        "正在准备服务端安装/更新",
        None,
    )?;
    runtime.update_instance_status(&instance.id, ServerStatus::Updating, None)?;
    emit_status(&app, &runtime, &instance.id)?;
    emit_instance_log(
        &app,
        &runtime,
        &instance.name,
        "info",
        "开始安装/更新服务端文件",
    )?;

    let output = run_steamcmd_update_with_retry(
        &app,
        &runtime,
        &steamcmd,
        Path::new(&instance.install_path),
        &progress,
        &instance,
        Arc::clone(&update_cancel),
    )
    .await;
    runtime.finish_update(&instance.id);
    match output {
        Ok(detail) => {
            emit_progress(
                &app,
                &progress,
                &instance.id,
                "completed",
                Some(100.0),
                "服务端安装/更新完成",
                Some(detail),
            )?;
            instance.server_version =
                read_installed_server_version(Path::new(&instance.install_path))
                    .unwrap_or_default();
            instance.version_state = "已安装/已更新".to_string();
            instance.status = ServerStatus::Stopped;
            instance.last_error = None;
            let mut updated = runtime.upsert_instance(instance.clone())?;
            let config = normalize_required_rcon_config(runtime.get_config(&updated.id)?)?;
            let mods = runtime.get_mods(&updated.id)?;
            updated = instance_config_commands::save_config_for_runtime(
                &app,
                &runtime,
                &updated.id,
                config,
                mods,
            )?;
            emit_instance_log(
                &app,
                &runtime,
                &updated.name,
                "success",
                "服务端安装/更新完成",
            )?;
            emit_status(&app, &runtime, &updated.id)?;
            Ok(with_current_server_version(updated))
        }
        Err(error) => {
            let cancelled = error == UPDATE_CANCELLED_MESSAGE;
            runtime.update_instance_status(
                &instance.id,
                if cancelled {
                    ServerStatus::Stopped
                } else {
                    ServerStatus::Error
                },
                if cancelled { None } else { Some(error.clone()) },
            )?;
            let log_level = if cancelled { "warn" } else { "error" };
            let log_message = if cancelled {
                UPDATE_CANCELLED_MESSAGE.to_string()
            } else {
                format!("服务端安装/更新失败：{error}")
            };
            emit_instance_log(&app, &runtime, &instance.name, log_level, &log_message)?;
            emit_status(&app, &runtime, &instance.id)?;
            Err(error)
        }
    }
}
