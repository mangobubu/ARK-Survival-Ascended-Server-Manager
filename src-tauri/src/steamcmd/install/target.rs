use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::steamcmd::{SteamCmdInstallResult, inspect_steamcmd};

pub(in crate::steamcmd) fn ensure_install_target(
    parent: &Path,
) -> Result<(PathBuf, Option<SteamCmdInstallResult>), String> {
    if !parent.exists() {
        return Err("选择的上级目录不存在".to_string());
    }
    if !parent.is_dir() {
        return Err("选择的上级路径不是目录".to_string());
    }

    let target = parent.join("SteamCMD");
    let current = inspect_steamcmd(&target);
    if current.valid {
        return Ok((
            target,
            Some(SteamCmdInstallResult {
                path: current.path,
                executable_path: current.executable_path,
            }),
        ));
    }

    if target.exists() {
        let mut entries =
            fs::read_dir(&target).map_err(|error| format!("无法读取目标目录：{error}"))?;
        if entries.next().is_some() {
            return Err(format!(
                "目标目录 {} 已存在且不为空，请选择其他上级目录",
                target.display()
            ));
        }
    }

    Ok((target, None))
}
