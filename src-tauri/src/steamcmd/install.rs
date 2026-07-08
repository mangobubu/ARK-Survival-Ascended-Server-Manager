mod archive;
mod download;
mod initialize;
mod progress;
mod target;

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::steamcmd::{SteamCmdInstallResult, SteamCmdProgress, inspect_steamcmd};
use tauri::ipc::Channel;

#[cfg(test)]
pub(super) use archive::{cleanup_staging, extract_archive};
#[cfg(test)]
pub(super) use progress::calculate_speed;
#[cfg(test)]
pub(super) use target::ensure_install_target;

pub(super) async fn install_inner(
    parent: &Path,
    progress: &Channel<SteamCmdProgress>,
) -> Result<SteamCmdInstallResult, String> {
    let (target, existing) = target::ensure_install_target(parent)?;
    if let Some(existing) = existing {
        progress::emit_progress(progress, "completed", 0, None, 0, "已找到可用的 SteamCMD")?;
        return Ok(existing);
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let staging = parent.join(format!(
        ".steamcmd-installing-{}-{nonce}",
        std::process::id()
    ));
    let archive_path = staging.join("steamcmd.zip");
    let content_path = staging.join("content");
    fs::create_dir(&staging).map_err(|error| format!("无法创建安装临时目录：{error}"))?;

    let result = async {
        let (downloaded, total) = download::download_archive(&archive_path, progress).await?;
        progress::emit_progress(
            progress,
            "extracting",
            downloaded,
            total,
            0,
            "正在安全解压 SteamCMD",
        )?;

        let archive_for_extract = archive_path.clone();
        let content_for_extract = content_path.clone();
        tokio::task::spawn_blocking(move || {
            archive::extract_archive(&archive_for_extract, &content_for_extract)
        })
        .await
        .map_err(|error| format!("SteamCMD 解压任务异常：{error}"))??;

        progress::emit_progress(
            progress,
            "initializing",
            downloaded,
            total,
            0,
            "正在后台初始化 SteamCMD",
        )?;
        initialize::initialize_steamcmd(&content_path).await?;

        if target.exists() {
            fs::remove_dir(&target).map_err(|error| format!("无法使用空目标目录：{error}"))?;
        }
        fs::rename(&content_path, &target)
            .map_err(|error| format!("无法完成 SteamCMD 安装：{error}"))?;

        let check = inspect_steamcmd(&target);
        if !check.valid {
            return Err(check
                .reason
                .unwrap_or_else(|| "SteamCMD 安装验证失败".to_string()));
        }

        progress::emit_progress(
            progress,
            "completed",
            downloaded,
            total,
            0,
            "SteamCMD 安装完成",
        )?;
        Ok(SteamCmdInstallResult {
            path: check.path,
            executable_path: check.executable_path,
        })
    }
    .await;

    archive::cleanup_staging(&staging);
    result
}
