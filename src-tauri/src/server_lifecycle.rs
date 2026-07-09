use crate::{
    app_state::{AppRuntime, normalize_required_rcon_config},
    ark_config,
    asa_server_process::{
        configure_asa_server_hidden_process, hide_asa_server_windows_after_spawn,
    },
    command_events::{emit_instance_log, emit_status},
    instance_config_commands,
    server_lifecycle_monitor::{
        apply_exited_instance_status, monitor_startup_readiness, should_auto_restart_after_exit,
        take_exited_instance_process,
    },
    server_log::new_server_log_deduper,
    server_log_cleanup::clear_instance_server_logs_before_start,
    server_log_reader::{attach_process_log_reader, attach_server_log_file_reader},
    server_rcon,
    server_version::with_current_server_version,
};
use std::{path::Path, process::Stdio, time::Instant};
use tauri::AppHandle;
use tokio::process::Command;

use crate::models::{ServerInstance, ServerStatus};

pub(crate) use crate::server_lifecycle_stop::{
    restart_instance_for_runtime, stop_instance_for_runtime,
};

pub(crate) async fn start_instance_for_runtime(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let instance = runtime.get_instance(&instance_id)?;
    if instance.status == ServerStatus::Running {
        return Ok(with_current_server_version(instance));
    }
    let config = normalize_required_rcon_config(runtime.get_config(&instance_id)?)?;
    let mods = runtime.get_mods(&instance_id)?;
    let instance = instance_config_commands::save_config_for_runtime(
        &app,
        &runtime,
        &instance_id,
        config.clone(),
        mods.clone(),
    )?;
    let executable = ark_config::server_executable(&instance).ok_or_else(|| {
        format!(
            "未找到 ASA 服务端可执行文件，请先安装/更新实例：{}",
            instance.install_path
        )
    })?;

    clear_instance_server_logs_before_start(&app, &runtime, &instance).await;

    runtime.update_instance_status(&instance_id, ServerStatus::Starting, None)?;
    emit_status(&app, &runtime, &instance_id)?;
    let args = ark_config::build_launch_arguments(&instance, &config, &mods);

    let mut command = Command::new(&executable);
    command
        .current_dir(
            executable
                .parent()
                .unwrap_or_else(|| Path::new(&instance.install_path)),
        )
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(false);

    configure_asa_server_hidden_process(&mut command);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let error = format!("启动 ASA 服务端失败：{error}");
            let _ = runtime.update_instance_status(
                &instance_id,
                ServerStatus::Error,
                Some(error.clone()),
            );
            let _ = emit_status(&app, &runtime, &instance_id);
            return Err(error);
        }
    };
    let pid = child.id();
    #[cfg(windows)]
    if let Some(pid) = pid {
        hide_asa_server_windows_after_spawn(pid);
    }
    let server_log_deduper = new_server_log_deduper();
    attach_process_log_reader(
        &app,
        &runtime,
        &instance,
        child.stdout.take(),
        "info",
        server_log_deduper.clone(),
    );
    attach_process_log_reader(
        &app,
        &runtime,
        &instance,
        child.stderr.take(),
        "error",
        server_log_deduper.clone(),
    );

    {
        let mut processes = runtime.lock_processes()?;
        processes.insert(instance_id.clone(), child);
    }
    attach_server_log_file_reader(&app, &runtime, &instance, server_log_deduper);

    emit_instance_log(
        &app,
        &runtime,
        &instance.name,
        "success",
        &format!("实例启动命令已执行，PID：{}", pid.unwrap_or_default()),
    )?;
    let updated = runtime.set_instance_pid(&instance_id, pid, ServerStatus::Starting)?;
    emit_status(&app, &runtime, &instance_id)?;
    tokio::spawn(monitor_startup_readiness(
        app.clone(),
        runtime.clone(),
        instance_id.clone(),
        Instant::now(),
    ));
    Ok(with_current_server_version(updated))
}

pub(crate) async fn refresh_status_for_runtime(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<ServerInstance, String> {
    if let Some(exit_status) = take_exited_instance_process(runtime, instance_id)? {
        let should_restart = should_auto_restart_after_exit(runtime, instance_id, &exit_status);
        let updated = apply_exited_instance_status(app, runtime, instance_id, exit_status)?;
        if should_restart {
            emit_instance_log(
                app,
                runtime,
                &updated.name,
                "warn",
                "已根据全局设置和实例配置触发崩溃后自动重启",
            )?;
            return start_instance_for_runtime(
                app.clone(),
                runtime.clone(),
                instance_id.to_string(),
            )
            .await;
        }
        return Ok(updated);
    }
    let instance = runtime.get_instance(instance_id)?;
    server_rcon::refresh_instance_players(runtime, instance)
        .await
        .map(with_current_server_version)
}
