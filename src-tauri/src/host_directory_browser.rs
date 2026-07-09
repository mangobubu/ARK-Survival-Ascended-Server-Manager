use crate::{
    app_state::AppRuntime, instance_config_import_paths::path_text,
    instance_runtime_config::normalize_path_text, path_security::path_starts_with_case_insensitive,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

const MAX_DIRECTORY_ENTRIES: usize = 500;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HostDirectoryEntry {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) has_children: bool,
    pub(crate) server_config_detected: bool,
    pub(crate) server_executable_detected: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HostDirectoryListing {
    pub(crate) root_path: String,
    pub(crate) current_path: String,
    pub(crate) parent_path: Option<String>,
    pub(crate) entries: Vec<HostDirectoryEntry>,
    pub(crate) total_entries: usize,
    pub(crate) truncated: bool,
}

pub(crate) fn list_host_directories(
    runtime: &AppRuntime,
    path: Option<String>,
) -> Result<HostDirectoryListing, String> {
    let settings = runtime.settings()?;
    let root_text = settings.server_storage_path.trim();
    if root_text.is_empty() {
        return Err("服务器存储目录尚未配置，无法打开 Web 目录选择器".to_string());
    }

    let requested = path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_path_text);

    list_host_directories_in_root(Path::new(root_text), requested.as_deref())
}

fn list_host_directories_in_root(
    root: &Path,
    requested: Option<&Path>,
) -> Result<HostDirectoryListing, String> {
    let root = canonical_directory(root, "服务器存储目录")?;
    let current = resolve_current_directory(&root, requested)?;
    let parent_path = current
        .parent()
        .and_then(|parent| canonical_directory(parent, "上级目录").ok())
        .filter(|parent| parent != &current && path_starts_with_case_insensitive(parent, &root))
        .map(|parent| path_text(&parent));

    let mut entries = fs::read_dir(&current)
        .map_err(|error| format!("无法读取目录 {}：{error}", path_text(&current)))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| build_entry(&path))
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.name.cmp(&right.name))
    });

    let total_entries = entries.len();
    let truncated = total_entries > MAX_DIRECTORY_ENTRIES;
    entries.truncate(MAX_DIRECTORY_ENTRIES);

    Ok(HostDirectoryListing {
        root_path: path_text(&root),
        current_path: path_text(&current),
        parent_path,
        entries,
        total_entries,
        truncated,
    })
}

fn resolve_current_directory(root: &Path, requested: Option<&Path>) -> Result<PathBuf, String> {
    let requested = requested.unwrap_or(root);
    let current = if requested.exists() && requested.is_dir() {
        canonical_directory(requested, "当前目录")?
    } else {
        nearest_existing_directory(requested).unwrap_or_else(|| root.to_path_buf())
    };

    if path_starts_with_case_insensitive(&current, root) {
        Ok(current)
    } else {
        Err(format!(
            "目录访问被拒绝：{}；Web 目录选择器仅允许浏览服务器存储目录 {} 内的文件夹",
            path_text(&current),
            path_text(root)
        ))
    }
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("无法解析{label} {}：{error}", path.display()))?;
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(format!("{label}不是目录：{}", path_text(&canonical)))
    }
}

fn nearest_existing_directory(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|candidate| candidate.is_dir())
        .and_then(|candidate| fs::canonicalize(candidate).ok())
}

fn build_entry(path: &Path) -> HostDirectoryEntry {
    HostDirectoryEntry {
        name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string(),
        path: path_text(path),
        has_children: has_child_directory(path),
        server_config_detected: server_config_detected(path),
        server_executable_detected: server_executable_detected(path),
    }
}

fn has_child_directory(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .any(|child| child.is_dir())
        })
        .unwrap_or(false)
}

fn server_config_detected(path: &Path) -> bool {
    [
        path.join("GameUserSettings.ini"),
        path.join("Game.ini"),
        path.join("WindowsServer").join("GameUserSettings.ini"),
        path.join("WindowsServer").join("Game.ini"),
        path.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsServer")
            .join("GameUserSettings.ini"),
        path.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsServer")
            .join("Game.ini"),
    ]
    .iter()
    .any(|candidate| candidate.is_file())
}

fn server_executable_detected(path: &Path) -> bool {
    path.join("ShooterGame")
        .join("Binaries")
        .join("Win64")
        .join("ArkAscendedServer.exe")
        .is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn 列出服务器存储目录内的子目录并标记_asa_目录() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let root = temp.path().join("服务器");
        let config_dir = root
            .join("ASA-01")
            .join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsServer");
        let binary_dir = root
            .join("ASA-01")
            .join("ShooterGame")
            .join("Binaries")
            .join("Win64");
        fs::create_dir_all(&config_dir).expect("创建配置目录");
        fs::create_dir_all(&binary_dir).expect("创建程序目录");
        fs::write(config_dir.join("GameUserSettings.ini"), "").expect("写入配置");
        fs::write(binary_dir.join("ArkAscendedServer.exe"), "").expect("写入程序");
        fs::create_dir_all(root.join("ASA-02").join("Saved")).expect("创建普通目录");

        let listing = list_host_directories_in_root(&root, None).expect("读取目录");

        assert_eq!(listing.entries.len(), 2);
        let asa = listing
            .entries
            .iter()
            .find(|entry| entry.name == "ASA-01")
            .expect("找到 ASA-01");
        assert!(asa.has_children);
        assert!(asa.server_config_detected);
        assert!(asa.server_executable_detected);
    }

    #[test]
    fn 请求不存在的子路径时回退到最近存在的托管目录() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let root = temp.path().join("服务器");
        let asa = root.join("ASA-01");
        fs::create_dir_all(&asa).expect("创建实例目录");

        let listing =
            list_host_directories_in_root(&root, Some(&asa.join("尚未创建").join("子目录")))
                .expect("读取目录");

        assert_eq!(listing.current_path, path_text(&asa));
        assert_eq!(
            listing.parent_path.as_deref(),
            Some(path_text(&root).as_str())
        );
    }

    #[test]
    fn 拒绝浏览服务器存储目录之外的路径() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let root = temp.path().join("服务器");
        let outside = temp.path().join("其他目录");
        fs::create_dir_all(&root).expect("创建服务器目录");
        fs::create_dir_all(&outside).expect("创建外部目录");

        let error = list_host_directories_in_root(&root, Some(outside.as_path()))
            .expect_err("应拒绝外部路径");

        assert!(error.contains("目录访问被拒绝"));
    }
}
