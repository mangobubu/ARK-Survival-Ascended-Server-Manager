use std::time::{Duration, Instant};

use crate::{
    app_state::AppRuntime,
    command_events::{emit_instance_log, emit_status},
    models::ServerStatus,
    server_rcon,
};
use tauri::AppHandle;
use tokio::time::{MissedTickBehavior, interval};

use super::process_exit::{apply_exited_instance_status, take_exited_instance_process};

const SERVER_STARTUP_PROBE_INTERVAL: Duration = Duration::from_secs(3);
const SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const SERVER_STARTUP_PROBE_LOG_INTERVAL: Duration = Duration::from_secs(30);

pub(crate) async fn monitor_startup_readiness(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
    started_at: Instant,
) {
    let mut ticker = interval(SERVER_STARTUP_PROBE_INTERVAL);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_probe_error_log_at: Option<Instant> = None;
    let mut timeout_reported = false;

    loop {
        ticker.tick().await;

        let instance = match runtime.get_instance(&instance_id) {
            Ok(instance) => instance,
            Err(_) => break,
        };
        if instance.status != ServerStatus::Starting {
            break;
        }

        match take_exited_instance_process(&runtime, &instance_id) {
            Ok(Some(exit_status)) => {
                let _ = apply_exited_instance_status(&app, &runtime, &instance_id, exit_status);
                break;
            }
            Ok(None) => {}
            Err(error) => {
                let _ = emit_instance_log(
                    &app,
                    &runtime,
                    &instance.name,
                    "error",
                    &format!("检查启动进程状态失败：{error}"),
                );
                break;
            }
        }

        let config = match runtime.get_config(&instance_id) {
            Ok(config) => config,
            Err(error) => {
                let _ = emit_instance_log(
                    &app,
                    &runtime,
                    &instance.name,
                    "error",
                    &format!("读取启动探测配置失败：{error}"),
                );
                break;
            }
        };

        match server_rcon::probe_server_readiness(&instance, &config).await {
            Ok(probe) => {
                let _ = emit_instance_log(
                    &app,
                    &runtime,
                    &instance.name,
                    "success",
                    &format!("启动探测通过：{}，实例已进入运行中", probe.method),
                );
                match runtime.set_instance_pid(&instance_id, instance.pid, ServerStatus::Running) {
                    Ok(_) => {
                        let _ = runtime.update_instance_players(&instance_id, probe.players);
                        let _ = emit_status(&app, &runtime, &instance_id);
                    }
                    Err(error) => {
                        let _ = emit_instance_log(
                            &app,
                            &runtime,
                            &instance.name,
                            "error",
                            &format!("更新运行状态失败：{error}"),
                        );
                    }
                }
                break;
            }
            Err(error) => {
                if last_probe_error_log_at
                    .map(|last| last.elapsed() >= SERVER_STARTUP_PROBE_LOG_INTERVAL)
                    .unwrap_or(true)
                {
                    let _ = emit_instance_log(
                        &app,
                        &runtime,
                        &instance.name,
                        "info",
                        &format!("启动探测未就绪：{error}"),
                    );
                    last_probe_error_log_at = Some(Instant::now());
                }

                if !timeout_reported && started_at.elapsed() >= SERVER_STARTUP_TIMEOUT {
                    timeout_reported = true;
                    let message = format!(
                        "启动探测超过 {} 分钟仍未就绪，进程仍在运行，将继续保持启动中并持续探测：{error}",
                        SERVER_STARTUP_TIMEOUT.as_secs() / 60
                    );
                    let _ = runtime.update_instance_status(
                        &instance_id,
                        ServerStatus::Starting,
                        Some(message.clone()),
                    );
                    let _ = emit_instance_log(&app, &runtime, &instance.name, "warn", &message);
                    let _ = emit_status(&app, &runtime, &instance_id);
                }
            }
        }
    }
}
