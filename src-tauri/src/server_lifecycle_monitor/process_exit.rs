use crate::{
    app_state::AppRuntime,
    command_events::{emit_instance_log, emit_status},
    models::{ServerInstance, ServerStatus},
    server_version::with_current_server_version,
};
use std::process::ExitStatus;
use tauri::AppHandle;

pub(crate) fn take_exited_instance_process(
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<Option<ExitStatus>, String> {
    let exit_status = {
        let mut processes = runtime.lock_processes()?;
        if let Some(child) = processes.get_mut(instance_id) {
            child
                .try_wait()
                .map_err(|error| format!("刷新进程状态失败：{error}"))?
        } else {
            None
        }
    };
    if exit_status.is_some() {
        let mut processes = runtime.lock_processes()?;
        processes.remove(instance_id);
    }
    Ok(exit_status)
}

pub(crate) fn apply_exited_instance_status(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
    exit_status: ExitStatus,
) -> Result<ServerInstance, String> {
    let instance = runtime.get_instance(instance_id)?;
    emit_instance_log(
        app,
        runtime,
        &instance.name,
        "warn",
        &format!("检测到服务端进程已退出：{exit_status}"),
    )?;

    let updated = if !exit_status.success() {
        let error = if instance.status == ServerStatus::Starting {
            format!("服务端启动过程中退出：{exit_status}")
        } else {
            format!("服务端进程异常退出：{exit_status}")
        };
        emit_instance_log(app, runtime, &instance.name, "error", &error)?;
        runtime.set_instance_pid(instance_id, None, ServerStatus::Error)?;
        runtime.update_instance_status(instance_id, ServerStatus::Error, Some(error))?
    } else {
        runtime.set_instance_pid(instance_id, None, ServerStatus::Stopped)?
    };
    emit_status(app, runtime, instance_id)?;
    Ok(with_current_server_version(updated))
}
