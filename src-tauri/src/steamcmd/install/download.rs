use std::{
    path::Path,
    time::{Duration, Instant},
};

use crate::steamcmd::SteamCmdProgress;
use futures_util::StreamExt;
use tauri::ipc::Channel;
use tokio::io::AsyncWriteExt;

use super::progress::{calculate_speed, emit_progress};

const STEAMCMD_DOWNLOAD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";

pub(super) async fn download_archive(
    archive_path: &Path,
    progress: &Channel<SteamCmdProgress>,
) -> Result<(u64, Option<u64>), String> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(300))
        .user_agent("ASA-Server-Manager/0.1")
        .build()
        .map_err(|error| format!("创建下载客户端失败：{error}"))?;

    let response = client
        .get(STEAMCMD_DOWNLOAD_URL)
        .send()
        .await
        .map_err(|error| format!("连接 SteamCMD 下载服务器失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("SteamCMD 下载服务器返回错误：{error}"))?;

    let total = response.content_length();
    let mut file = tokio::fs::File::create(archive_path)
        .await
        .map_err(|error| format!("无法创建临时下载文件：{error}"))?;
    let mut stream = response.bytes_stream();
    let mut downloaded = 0_u64;
    let mut sample_bytes = 0_u64;
    let mut sample_time = Instant::now();
    let mut current_speed = 0_u64;

    emit_progress(progress, "downloading", 0, total, 0, "正在下载 SteamCMD")?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("下载 SteamCMD 时连接中断：{error}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|error| format!("写入 SteamCMD 下载文件失败：{error}"))?;
        downloaded += chunk.len() as u64;
        sample_bytes += chunk.len() as u64;

        let elapsed = sample_time.elapsed();
        if elapsed >= Duration::from_millis(250) {
            current_speed = calculate_speed(sample_bytes, elapsed);
            emit_progress(
                progress,
                "downloading",
                downloaded,
                total,
                current_speed,
                "正在下载 SteamCMD",
            )?;
            sample_bytes = 0;
            sample_time = Instant::now();
        }
    }

    if sample_bytes > 0 {
        current_speed = calculate_speed(sample_bytes, sample_time.elapsed());
    }

    file.flush()
        .await
        .map_err(|error| format!("刷新 SteamCMD 下载文件失败：{error}"))?;
    emit_progress(
        progress,
        "downloading",
        downloaded,
        total,
        current_speed,
        "SteamCMD 下载完成",
    )?;
    Ok((downloaded, total))
}
