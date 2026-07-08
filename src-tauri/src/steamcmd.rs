mod install;

use serde::Serialize;
use std::path::Path;
use tauri::ipc::Channel;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamCmdCheck {
    pub path: String,
    pub executable_path: String,
    pub valid: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamCmdProgress {
    pub phase: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub bytes_per_second: u64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamCmdInstallResult {
    pub path: String,
    pub executable_path: String,
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn inspect_steamcmd(path: &Path) -> SteamCmdCheck {
    let executable = path.join("steamcmd.exe");
    let reason = if !path.exists() {
        Some("目录不存在".to_string())
    } else if !path.is_dir() {
        Some("所选路径不是目录".to_string())
    } else if !executable.is_file() {
        Some("目录中未找到 steamcmd.exe".to_string())
    } else {
        None
    };

    SteamCmdCheck {
        path: path_text(path),
        executable_path: path_text(&executable),
        valid: reason.is_none(),
        reason,
    }
}

#[tauri::command]
pub fn check_steamcmd(path: String) -> Result<SteamCmdCheck, String> {
    if path.trim().is_empty() {
        return Ok(SteamCmdCheck {
            path,
            executable_path: String::new(),
            valid: false,
            reason: Some("SteamCMD 目录不能为空".to_string()),
        });
    }

    Ok(inspect_steamcmd(Path::new(&path)))
}

#[tauri::command]
pub async fn install_steamcmd(
    parent_path: String,
    progress: Channel<SteamCmdProgress>,
) -> Result<SteamCmdInstallResult, String> {
    #[cfg(not(windows))]
    {
        let _ = parent_path;
        let _ = progress;
        return Err("SteamCMD 静默安装目前仅支持 Windows".to_string());
    }

    #[cfg(windows)]
    {
        if parent_path.trim().is_empty() {
            return Err("SteamCMD 上级目录不能为空".to_string());
        }
        install::install_inner(Path::new(&parent_path), &progress).await
    }
}

#[cfg(test)]
mod tests;
