#[cfg(windows)]
use crate::steamcmd_process::kill_process_tree;
use crate::{
    app_state::AppRuntime,
    ark_config,
    command_events::{emit_instance_log, emit_progress, emit_progress_with_transfer},
    models::{JobProgress, ServerInstance},
    steamcmd_progress::tail_detail,
    steamcmd_update_monitor::spawn_manifest_progress_monitor,
    steamcmd_update_output::{
        SteamCmdProgressSink, spawn_command_log_reader, transfer_snapshot, wait_for_log_readers,
    },
};
use std::{
    collections::VecDeque,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, ipc::Channel};
use tokio::time::timeout;

use super::{
    UPDATE_CANCELLED_MESSAGE,
    command::{build_steamcmd_update_command, command_log_message},
};

pub(super) async fn run_steamcmd_update(
    app: &AppHandle,
    runtime: &AppRuntime,
    steamcmd: &Path,
    install_path: &Path,
    progress: &Channel<JobProgress>,
    instance: &ServerInstance,
    cancel: Arc<AtomicBool>,
) -> Result<String, String> {
    emit_progress(
        app,
        progress,
        &instance.id,
        "running",
        None,
        "正在调用 SteamCMD 安装/更新 ASA Dedicated Server",
        None,
    )?;
    emit_instance_log(
        app,
        runtime,
        &instance.name,
        "info",
        &command_log_message(steamcmd, install_path),
    )?;

    let mut command = build_steamcmd_update_command(steamcmd, install_path)?;

    #[cfg(not(windows))]
    {
        let _ = (app, runtime, progress, instance, cancel, command);
        unreachable!("非 Windows 平台会在 build_steamcmd_update_command 中提前返回错误");
    }

    #[cfg(windows)]
    {
        let mut child = command
            .spawn()
            .map_err(|error| format!("无法启动 SteamCMD：{error}"))?;
        let output_tail = Arc::new(Mutex::new(VecDeque::with_capacity(24)));
        let progress_sink = SteamCmdProgressSink::new(app, runtime, progress, instance);
        let child_pid = child.id();
        let manifest_monitor_stop = Arc::new(AtomicBool::new(false));
        let manifest_monitor = spawn_manifest_progress_monitor(
            install_path,
            child_pid,
            progress_sink.clone(),
            Arc::clone(&manifest_monitor_stop),
        );
        let readers = vec![
            spawn_command_log_reader(
                app,
                runtime,
                &instance.name,
                child.stdout.take(),
                "info",
                Arc::clone(&output_tail),
                Some(progress_sink.clone()),
            ),
            spawn_command_log_reader(
                app,
                runtime,
                &instance.name,
                child.stderr.take(),
                "error",
                Arc::clone(&output_tail),
                Some(progress_sink.clone()),
            ),
        ];

        let started_at = Instant::now();
        let status = loop {
            if cancel.load(Ordering::SeqCst) {
                emit_instance_log(
                    app,
                    runtime,
                    &instance.name,
                    "warn",
                    "已取消安装/更新，正在结束 SteamCMD 进程树",
                )?;
                if let Some(pid) = child_pid {
                    kill_process_tree(pid).await;
                }
                let _ = child.kill().await;
                let _ = child.wait().await;
                manifest_monitor_stop.store(true, Ordering::SeqCst);
                manifest_monitor.abort();
                wait_for_log_readers(readers).await;
                return Err(UPDATE_CANCELLED_MESSAGE.to_string());
            }

            match timeout(Duration::from_millis(500), child.wait()).await {
                Ok(Ok(status)) => break status,
                Ok(Err(error)) => {
                    manifest_monitor_stop.store(true, Ordering::SeqCst);
                    manifest_monitor.abort();
                    wait_for_log_readers(readers).await;
                    return Err(format!("等待 SteamCMD 结束失败：{error}"));
                }
                Err(_) if started_at.elapsed() >= Duration::from_secs(60 * 60) => {
                    if let Some(pid) = child_pid {
                        kill_process_tree(pid).await;
                    }
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    manifest_monitor_stop.store(true, Ordering::SeqCst);
                    manifest_monitor.abort();
                    wait_for_log_readers(readers).await;
                    return Err("SteamCMD 安装/更新超时（60 分钟）".to_string());
                }
                Err(_) => {}
            }
        };
        manifest_monitor_stop.store(true, Ordering::SeqCst);
        manifest_monitor.abort();
        wait_for_log_readers(readers).await;

        if !status.success() {
            let fallback = format!("SteamCMD 安装/更新失败，退出代码：{status}");
            return Err(tail_detail(&output_tail, &fallback));
        }
        let transfer = transfer_snapshot(&progress_sink);
        emit_progress_with_transfer(
            app,
            progress,
            &instance.id,
            "verifying",
            transfer.percent,
            "正在验证服务端文件",
            None,
            transfer,
        )?;
        if ark_config::server_executable(instance).is_none() {
            return Err(tail_detail(
                &output_tail,
                "SteamCMD 执行完成，但未找到 ASA 服务端可执行文件",
            ));
        }
        Ok(tail_detail(&output_tail, "SteamCMD 安装/更新完成"))
    }
}
