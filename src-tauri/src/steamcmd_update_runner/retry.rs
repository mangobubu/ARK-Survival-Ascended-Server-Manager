use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crate::{
    app_state::AppRuntime,
    command_events::{emit_instance_log, emit_progress},
    models::{JobProgress, ServerInstance},
    steamcmd_progress::is_retryable_steamcmd_configuration_error,
};
use tauri::{AppHandle, ipc::Channel};
use tokio::time::sleep;

use super::{UPDATE_CANCELLED_MESSAGE, execution::run_steamcmd_update};

pub(crate) async fn run_steamcmd_update_with_retry(
    app: &AppHandle,
    runtime: &AppRuntime,
    steamcmd: &Path,
    install_path: &Path,
    progress: &Channel<JobProgress>,
    instance: &ServerInstance,
    cancel: Arc<AtomicBool>,
) -> Result<String, String> {
    let mut attempted_retry = false;

    loop {
        let result = run_steamcmd_update(
            app,
            runtime,
            steamcmd,
            install_path,
            progress,
            instance,
            Arc::clone(&cancel),
        )
        .await;

        match result {
            Err(error) if !attempted_retry && is_retryable_steamcmd_configuration_error(&error) => {
                attempted_retry = true;
                emit_instance_log(
                    app,
                    runtime,
                    &instance.name,
                    "warn",
                    "SteamCMD 配置尚未就绪，正在等待后自动重试安装/更新",
                )?;
                emit_progress(
                    app,
                    progress,
                    &instance.id,
                    "preparing",
                    Some(8.0),
                    "SteamCMD 配置刷新后准备重试安装/更新",
                    Some(error),
                )?;
                sleep(Duration::from_secs(2)).await;
                if cancel.load(Ordering::SeqCst) {
                    return Err(UPDATE_CANCELLED_MESSAGE.to_string());
                }
            }
            other => return other,
        }
    }
}
