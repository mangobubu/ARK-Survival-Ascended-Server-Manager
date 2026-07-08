use std::{path::Path, process::Stdio};

use crate::models::ASA_DEDICATED_SERVER_APP_ID;
#[cfg(windows)]
use crate::steamcmd_process::CREATE_NO_WINDOW;
use tokio::process::Command;

pub(super) fn command_log_message(steamcmd: &Path, install_path: &Path) -> String {
    format!(
        "SteamCMD 命令：{} +force_install_dir {} +login anonymous +app_update {} validate +quit",
        steamcmd.display(),
        install_path.display(),
        ASA_DEDICATED_SERVER_APP_ID
    )
}

pub(super) fn build_steamcmd_update_command(
    steamcmd: &Path,
    install_path: &Path,
) -> Result<Command, String> {
    #[cfg(not(windows))]
    {
        let _ = (steamcmd, install_path);
        return Err("ASA 服务端自动安装/更新目前仅支持 Windows".to_string());
    }

    #[cfg(windows)]
    {
        let steamcmd_dir = steamcmd
            .parent()
            .ok_or_else(|| format!("无法定位 SteamCMD 工作目录：{}", steamcmd.display()))?;
        let mut command = Command::new(steamcmd);
        command
            .current_dir(steamcmd_dir)
            .arg("+force_install_dir")
            .arg(install_path)
            .arg("+login")
            .arg("anonymous")
            .arg("+app_update")
            .arg(ASA_DEDICATED_SERVER_APP_ID)
            .arg("validate")
            .arg("+quit")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        command.creation_flags(CREATE_NO_WINDOW);
        Ok(command)
    }
}
