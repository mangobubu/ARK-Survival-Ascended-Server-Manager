use crate::{
    app_state::{AppRuntime, normalize_required_rcon_config},
    ark_config,
    command_events::{emit_instance_log, emit_status},
    instance_config_commands,
    models::{ModItem, ServerInstance, ServerStatus},
    rcon,
    server_lifecycle::{start_instance_after_config_saved, start_instance_task},
    server_lifecycle_monitor::bool_from_config,
    server_version::with_current_server_version,
    steamcmd_process::{kill_process_tree, process_is_running, process_matches_executable},
};
use serde_json::Value;
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::{Instant, timeout};

const FORCE_STOP_TIMEOUT: Duration = Duration::from_secs(10);
const UPDATE_CANCEL_TIMEOUT: Duration = Duration::from_secs(30);

enum StopPreparation {
    AlreadyStopped(ServerInstance),
    Pending {
        previous_status: ServerStatus,
        updated: ServerInstance,
    },
}

pub(crate) fn stop_instance_for_runtime(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let operation = runtime.begin_stop_operation(&instance_id)?;
    let (stop_from_status, updated) = match prepare_instance_stop(
        &app,
        &runtime,
        &instance_id,
        "停止请求已提交，后台正在停止实例",
    )? {
        StopPreparation::AlreadyStopped(instance) => {
            return Ok(with_current_server_version(instance));
        }
        StopPreparation::Pending {
            previous_status,
            updated,
        } => (previous_status, updated),
    };

    let task_app = app.clone();
    let task_runtime = runtime.clone();
    let task_instance_id = instance_id.clone();
    tokio::spawn(async move {
        let _operation = operation;
        if let Err(error) = stop_instance_task(
            task_app.clone(),
            task_runtime.clone(),
            task_instance_id.clone(),
            stop_from_status,
        )
        .await
        {
            record_stop_failure(
                &task_app,
                &task_runtime,
                &task_instance_id,
                &error,
                "后台停止实例失败",
            );
        }
    });

    Ok(with_current_server_version(updated))
}

fn prepare_instance_stop(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
    message: &str,
) -> Result<StopPreparation, String> {
    let instance = runtime.get_instance(instance_id)?;
    let update_running = runtime.is_update_running(instance_id)?;
    if instance.status == ServerStatus::Stopped {
        let has_tracked_process = runtime.lock_processes()?.contains_key(instance_id);
        let persisted_process_running = instance
            .pid
            .map(process_is_running)
            .transpose()?
            .unwrap_or(false);
        if !update_running && !has_tracked_process && !persisted_process_running {
            let instance = if instance.pid.is_some() {
                runtime.set_instance_pid(instance_id, None, ServerStatus::Stopped)?
            } else {
                instance
            };
            return Ok(StopPreparation::AlreadyStopped(instance));
        }
    }

    let previous_status = if update_running {
        ServerStatus::Updating
    } else {
        instance.status.clone()
    };
    let updated = runtime.update_instance_status(instance_id, ServerStatus::Stopping, None)?;
    let _ = emit_instance_log(app, runtime, &instance.name, "warn", message);
    let _ = emit_status(app, runtime, instance_id);
    Ok(StopPreparation::Pending {
        previous_status,
        updated,
    })
}

fn record_stop_failure(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
    error: &str,
    prefix: &str,
) {
    let instance_name = runtime
        .get_instance(instance_id)
        .map(|instance| instance.name)
        .unwrap_or_else(|_| instance_id.to_string());
    let _ =
        runtime.update_instance_status(instance_id, ServerStatus::Error, Some(error.to_string()));
    let _ = emit_instance_log(
        app,
        runtime,
        &instance_name,
        "error",
        &format!("{prefix}：{error}"),
    );
    let _ = emit_status(app, runtime, instance_id);
}

async fn stop_instance_task(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
    stop_from_status: ServerStatus,
) -> Result<ServerInstance, String> {
    let instance = runtime.get_instance(&instance_id)?;
    if stop_from_status == ServerStatus::Updating {
        if runtime.cancel_update(&instance_id)? {
            emit_instance_log(
                &app,
                &runtime,
                &instance.name,
                "warn",
                "正在取消安装/更新任务",
            )?;
            let cancel_started_at = Instant::now();
            loop {
                if !runtime.is_update_running(&instance_id)? {
                    break;
                }
                if cancel_started_at.elapsed() >= UPDATE_CANCEL_TIMEOUT {
                    return Err(format!(
                        "等待安装/更新任务取消超时（{} 秒），已中止后续操作",
                        UPDATE_CANCEL_TIMEOUT.as_secs()
                    ));
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }

        let updated = runtime.set_instance_pid(&instance_id, None, ServerStatus::Stopped)?;
        emit_status(&app, &runtime, &instance_id)?;
        return Ok(with_current_server_version(updated));
    }

    let config = runtime.get_config(&instance_id)?;
    if bool_from_config(&config, "saveOnStop", true) {
        let password = config
            .get("adminPassword")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let rcon_port = config
            .get("rconPort")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(instance.rcon_port);
        match rcon::execute("127.0.0.1", rcon_port, password, "saveworld").await {
            Ok(_) => {
                emit_instance_log(
                    &app,
                    &runtime,
                    &instance.name,
                    "success",
                    "RCON 已执行 saveworld",
                )?;
            }
            Err(error) => {
                emit_instance_log(
                    &app,
                    &runtime,
                    &instance.name,
                    "warn",
                    &format!("RCON 保存失败，将继续停止进程：{error}"),
                )?;
            }
        }
        let _ = rcon::execute("127.0.0.1", rcon_port, password, "doexit").await;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    let mut child = {
        let mut processes = runtime.lock_processes()?;
        processes.remove(&instance_id)
    };
    if let Some(child) = child.as_mut() {
        stop_tracked_process(child).await?;
    } else {
        stop_recovered_process(&app, &runtime, &instance).await?;
    }

    emit_instance_log(&app, &runtime, &instance.name, "warn", "实例已停止")?;
    let updated = runtime.set_instance_pid(&instance_id, None, ServerStatus::Stopped)?;
    emit_status(&app, &runtime, &instance_id)?;
    Ok(with_current_server_version(updated))
}

async fn stop_tracked_process(child: &mut tokio::process::Child) -> Result<(), String> {
    match child.try_wait() {
        Ok(Some(_)) => return Ok(()),
        Ok(None) => {}
        Err(error) => return Err(format!("检查服务端进程状态失败：{error}")),
    }

    let pid = child.id();
    let tree_kill_error = if let Some(pid) = pid {
        kill_process_tree(pid).await.err()
    } else {
        Some("服务端进程缺少 PID".to_string())
    };

    if tree_kill_error.is_some() {
        timeout(FORCE_STOP_TIMEOUT, child.kill())
            .await
            .map_err(|_| "直接终止服务端进程超时".to_string())?
            .map_err(|error| {
                format!(
                    "无法直接终止服务端进程：{error}；{}",
                    tree_kill_error.as_deref().unwrap_or_default()
                )
            })?;
    }

    timeout(FORCE_STOP_TIMEOUT, child.wait())
        .await
        .map_err(|_| "等待服务端进程退出超时".to_string())?
        .map_err(|error| format!("等待服务端进程退出失败：{error}"))?;
    Ok(())
}

async fn stop_recovered_process(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance: &ServerInstance,
) -> Result<(), String> {
    let Some(pid) = instance.pid else {
        return Ok(());
    };
    if !process_is_running(pid)? {
        return Ok(());
    }

    let executable = ark_config::server_executable(instance).ok_or_else(|| {
        format!("实例记录的 PID {pid} 仍在运行，但无法找到 ASA 服务端可执行文件以核验进程身份")
    })?;
    let process_matches = match process_matches_executable(pid, &executable) {
        Ok(process_matches) => process_matches,
        Err(_) if !process_is_running(pid)? => return Ok(()),
        Err(error) => return Err(error),
    };
    if !process_matches {
        return Err(format!(
            "实例记录的 PID {pid} 仍在运行，但进程路径与 ASA 服务端不一致，已拒绝终止以避免误杀进程"
        ));
    }

    emit_instance_log(
        app,
        runtime,
        &instance.name,
        "warn",
        &format!("已恢复管理器重启前的服务端 PID {pid}，正在停止该进程"),
    )?;
    kill_process_tree(pid).await
}

async fn stop_instance_before_restart(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<(), String> {
    let previous_status = match prepare_instance_stop(
        app,
        runtime,
        instance_id,
        "正在停止实例，确认服务端进程退出后将重新启动",
    )? {
        StopPreparation::AlreadyStopped(_) => return Ok(()),
        StopPreparation::Pending {
            previous_status, ..
        } => previous_status,
    };

    if let Err(error) = stop_instance_task(
        app.clone(),
        runtime.clone(),
        instance_id.to_string(),
        previous_status,
    )
    .await
    {
        record_stop_failure(
            app,
            runtime,
            instance_id,
            &error,
            "停止实例失败，已取消重新启动",
        );
        return Err(error);
    }
    Ok(())
}

pub(crate) async fn restart_instance_for_runtime(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let _operation = runtime.begin_lifecycle_operation(&instance_id)?;
    stop_instance_before_restart(&app, &runtime, &instance_id).await?;
    start_instance_task(app, runtime, instance_id).await
}

pub(crate) async fn apply_instance_config_and_restart_for_runtime(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
    config: Value,
    mods: Vec<ModItem>,
) -> Result<ServerInstance, String> {
    let config = normalize_required_rcon_config(config)?;
    instance_config_commands::validate_mods(&mods)?;
    let _configuration_operation = runtime.begin_configuration_operation()?;
    let _operation = runtime.begin_lifecycle_operation(&instance_id)?;
    let validated_instance = runtime.validate_config_metadata(&instance_id, &config)?;
    ark_config::server_executable(&validated_instance).ok_or_else(|| {
        format!(
            "未找到 ASA 服务端可执行文件，请先安装/更新实例：{}",
            validated_instance.install_path
        )
    })?;

    stop_instance_before_restart(&app, &runtime, &instance_id).await?;
    instance_config_commands::save_config_for_runtime(&app, &runtime, &instance_id, config, mods)?;
    start_instance_after_config_saved(app, runtime, instance_id).await
}
