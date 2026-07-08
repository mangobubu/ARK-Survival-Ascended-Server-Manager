use std::{path::Path, process::Stdio, time::Duration};

#[cfg(windows)]
use crate::steamcmd_process::CREATE_NO_WINDOW;
use tokio::{process::Command, time::timeout};

pub(super) async fn initialize_steamcmd(directory: &Path) -> Result<(), String> {
    let executable = directory.join("steamcmd.exe");
    let mut command = Command::new(&executable);
    command
        .current_dir(directory)
        .arg("+quit")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    #[cfg(not(windows))]
    return Err("SteamCMD 静默安装目前仅支持 Windows".to_string());

    #[cfg(windows)]
    {
        let output = timeout(Duration::from_secs(600), command.output())
            .await
            .map_err(|_| "SteamCMD 首次初始化超时（10 分钟）".to_string())?
            .map_err(|error| format!("无法启动 SteamCMD 初始化：{error}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");
        let initialization_verified = executable.is_file()
            && directory.join("steamclient.dll").is_file()
            && combined.contains("Loading Steam API...OK")
            && combined.contains("Unloading Steam API...OK");

        if !output.status.success() && !initialization_verified {
            let mut detail = stderr.trim().to_string();
            if detail.is_empty() {
                detail = stdout.trim().to_string();
            }
            if detail.len() > 500 {
                detail.truncate(500);
            }
            return Err(if detail.is_empty() {
                format!("SteamCMD 初始化失败，退出代码：{}", output.status)
            } else {
                format!("SteamCMD 初始化失败（{}）：{detail}", output.status)
            });
        }
    }

    Ok(())
}
