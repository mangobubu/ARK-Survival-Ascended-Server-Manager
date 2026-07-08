use std::{
    io::{Read, Seek},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::models::{ASA_DEDICATED_SERVER_APP_ID, ServerInstance};

use super::{
    SERVER_VERSION_SCAN_BYTES,
    parser::{normalize_server_version_value, parse_asa_server_version},
};

pub(crate) fn is_server_log_candidate(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            extension.eq_ignore_ascii_case("log") || extension.eq_ignore_ascii_case("txt")
        })
        .unwrap_or(false)
}

pub(crate) fn server_appmanifest_path(install_path: &Path) -> PathBuf {
    install_path
        .join("steamapps")
        .join(format!("appmanifest_{ASA_DEDICATED_SERVER_APP_ID}.acf"))
}

pub(crate) fn read_installed_server_version(install_path: &Path) -> Option<String> {
    read_known_server_version_file(install_path)
        .or_else(|| read_latest_saved_log_server_version(install_path))
}

pub(crate) fn with_current_server_version(mut instance: ServerInstance) -> ServerInstance {
    instance.server_version = read_installed_server_version(Path::new(&instance.install_path))
        .or_else(|| normalize_server_version_value(&instance.server_version))
        .unwrap_or_default();
    instance
}

fn read_known_server_version_file(install_path: &Path) -> Option<String> {
    [
        install_path
            .join("ShooterGame")
            .join("Binaries")
            .join("Win64")
            .join("ArkAscendedServer.version"),
        install_path
            .join("ShooterGame")
            .join("Binaries")
            .join("Win64")
            .join("Build.version"),
        install_path.join("ShooterGame").join("Build.version"),
        install_path.join("version.txt"),
    ]
    .into_iter()
    .find_map(|path| {
        if !path.is_file() {
            return None;
        }
        read_recent_file_text(&path, SERVER_VERSION_SCAN_BYTES)
            .ok()
            .and_then(|content| parse_asa_server_version(&content))
    })
}

fn read_latest_saved_log_server_version(install_path: &Path) -> Option<String> {
    let log_dir = install_path.join("ShooterGame").join("Saved").join("Logs");
    let entries = std::fs::read_dir(log_dir).ok()?;
    let latest = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !is_server_log_candidate(&path) {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            Some((metadata.modified().unwrap_or(UNIX_EPOCH), path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)?;

    read_recent_file_text(&latest, SERVER_VERSION_SCAN_BYTES)
        .ok()
        .and_then(|content| parse_asa_server_version(&content))
}

fn read_recent_file_text(path: &Path, max_bytes: u64) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| format!("无法打开版本候选文件 {}：{error}", path.display()))?;
    let len = file
        .metadata()
        .map_err(|error| format!("无法读取版本候选文件元数据 {}：{error}", path.display()))?
        .len();
    let start = len.saturating_sub(max_bytes);
    file.seek(std::io::SeekFrom::Start(start))
        .map_err(|error| format!("无法定位版本候选文件 {}：{error}", path.display()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|error| format!("无法读取版本候选文件 {}：{error}", path.display()))?;
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}
