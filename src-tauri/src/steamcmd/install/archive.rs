use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use zip::ZipArchive;

pub(in crate::steamcmd) fn cleanup_staging(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}

pub(in crate::steamcmd) fn extract_archive(
    archive_path: &Path,
    destination: &Path,
) -> Result<(), String> {
    let archive_file =
        File::open(archive_path).map_err(|error| format!("无法打开 SteamCMD 压缩包：{error}"))?;
    let mut archive =
        ZipArchive::new(archive_file).map_err(|error| format!("SteamCMD 压缩包无效：{error}"))?;

    fs::create_dir_all(destination).map_err(|error| format!("无法创建解压目录：{error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("读取压缩包条目失败：{error}"))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "SteamCMD 压缩包包含不安全路径，已终止安装".to_string())?;
        let output_path = destination.join(enclosed);

        if entry.is_dir() {
            fs::create_dir_all(&output_path)
                .map_err(|error| format!("创建解压目录失败：{error}"))?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建解压目录失败：{error}"))?;
        }
        let mut output =
            File::create(&output_path).map_err(|error| format!("创建解压文件失败：{error}"))?;
        io::copy(&mut entry, &mut output)
            .map_err(|error| format!("解压 SteamCMD 文件失败：{error}"))?;
        output
            .flush()
            .map_err(|error| format!("写入 SteamCMD 文件失败：{error}"))?;
    }

    if !destination.join("steamcmd.exe").is_file() {
        return Err("解压完成后未找到 steamcmd.exe".to_string());
    }
    Ok(())
}
