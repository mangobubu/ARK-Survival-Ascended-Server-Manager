use std::time::Duration;

use crate::steamcmd::SteamCmdProgress;
use tauri::ipc::Channel;

pub(super) fn emit_progress(
    channel: &Channel<SteamCmdProgress>,
    phase: &str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    bytes_per_second: u64,
    message: &str,
) -> Result<(), String> {
    channel
        .send(SteamCmdProgress {
            phase: phase.to_string(),
            downloaded_bytes,
            total_bytes,
            bytes_per_second,
            message: message.to_string(),
        })
        .map_err(|error| format!("发送下载进度失败：{error}"))
}

pub(in crate::steamcmd) fn calculate_speed(bytes_delta: u64, elapsed: Duration) -> u64 {
    let seconds = elapsed.as_secs_f64();
    if seconds <= f64::EPSILON {
        0
    } else {
        (bytes_delta as f64 / seconds).round() as u64
    }
}
