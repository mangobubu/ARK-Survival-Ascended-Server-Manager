use crate::{
    app_state::AppRuntime,
    command_events::{emit_instance_log, emit_status},
    models::{ServerInstance, ServerStatus},
    rcon,
    server_lifecycle::start_instance_for_runtime,
    server_lifecycle_monitor::bool_from_config,
    server_version::with_current_server_version,
    steamcmd_process::kill_process_tree,
};
use serde_json::Value;
use std::time::Duration;
use tauri::AppHandle;

pub(crate) fn stop_instance_for_runtime(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let instance = runtime.get_instance(&instance_id)?;
    if instance.status == ServerStatus::Stopped {
        return Ok(with_current_server_version(instance));
    }
    if instance.status == ServerStatus::Stopping {
        return Ok(with_current_server_version(instance));
    }

    let stop_from_status = instance.status.clone();
    let updated = runtime.update_instance_status(&instance_id, ServerStatus::Stopping, None)?;
    let _ = emit_instance_log(
        &app,
        &runtime,
        &instance.name,
        "warn",
        "停止请求已提交，后台正在停止实例",
    );
    let _ = emit_status(&app, &runtime, &instance_id);

    let task_app = app.clone();
    let task_runtime = runtime.clone();
    let task_instance_id = instance_id.clone();
    tokio::spawn(async move {
        if let Err(error) = stop_instance_task(
            task_app.clone(),
            task_runtime.clone(),
            task_instance_id.clone(),
            stop_from_status,
        )
        .await
        {
            let fallback_name = task_instance_id.clone();
            let instance_name = task_runtime
                .get_instance(&task_instance_id)
                .map(|instance| instance.name)
                .unwrap_or(fallback_name);
            let _ = task_runtime.update_instance_status(
                &task_instance_id,
                ServerStatus::Error,
                Some(error.clone()),
            );
            let _ = emit_instance_log(
                &task_app,
                &task_runtime,
                &instance_name,
                "error",
                &format!("后台停止实例失败：{error}"),
            );
            let _ = emit_status(&task_app, &task_runtime, &task_instance_id);
        }
    });

    Ok(with_current_server_version(updated))
}

async fn stop_instance_task(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
    stop_from_status: ServerStatus,
) -> Result<ServerInstance, String> {
    let instance = runtime.get_instance(&instance_id)?;
    if stop_from_status == ServerStatus::Stopped {
        return Ok(with_current_server_version(instance));
    }
    if stop_from_status == ServerStatus::Updating {
        if runtime.cancel_update(&instance_id)? {
            emit_instance_log(
                &app,
                &runtime,
                &instance.name,
                "warn",
                "正在取消安装/更新任务",
            )?;
            for _ in 0..20 {
                if !runtime.is_update_running(&instance_id)? {
                    break;
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
    if let Some(child) = child.as_mut()
        && child.try_wait().ok().flatten().is_none()
    {
        if let Some(pid) = child.id() {
            kill_process_tree(pid).await;
        }
        let _ = child.kill().await;
    }

    emit_instance_log(&app, &runtime, &instance.name, "warn", "实例已停止")?;
    let updated = runtime.set_instance_pid(&instance_id, None, ServerStatus::Stopped)?;
    emit_status(&app, &runtime, &instance_id)?;
    Ok(with_current_server_version(updated))
}

pub(crate) async fn restart_instance_for_runtime(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let stop_from_status = runtime.get_instance(&instance_id)?.status;
    stop_instance_task(
        app.clone(),
        runtime.clone(),
        instance_id.clone(),
        stop_from_status,
    )
    .await?;
    start_instance_for_runtime(app, runtime, instance_id).await
}
