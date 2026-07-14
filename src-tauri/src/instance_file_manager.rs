use crate::{
    app_state::AppRuntime, instance_config_import_paths::path_text,
    path_security::path_starts_with_case_insensitive,
};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const MAX_DIRECTORY_ENTRIES: usize = 1_000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstanceFileEntry {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) entry_type: String,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) modified_at: Option<u64>,
    pub(crate) has_children: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstanceDirectoryListing {
    pub(crate) root_path: String,
    pub(crate) current_path: String,
    pub(crate) parent_path: Option<String>,
    pub(crate) entries: Vec<InstanceFileEntry>,
    pub(crate) total_entries: usize,
    pub(crate) truncated: bool,
}

pub(crate) fn list_instance_directory(
    runtime: &AppRuntime,
    instance_id: &str,
    path: Option<&str>,
) -> Result<InstanceDirectoryListing, String> {
    let root = instance_root(runtime, instance_id)?;
    list_instance_directory_in_root(&root, path.map(Path::new))
}

pub(crate) fn create_entry(
    runtime: &AppRuntime,
    instance_id: &str,
    parent_path: &str,
    name: &str,
    entry_type: &str,
) -> Result<String, String> {
    let root = instance_root(runtime, instance_id)?;
    let parent = managed_existing_path(&root, Path::new(parent_path), true)?;
    ensure_directory(&parent)?;
    let destination = parent.join(validate_entry_name(name)?);
    if destination.exists() {
        return Err(format!(
            "同名文件或文件夹已存在：{}",
            path_text(&destination)
        ));
    }

    match entry_type {
        "directory" => fs::create_dir(&destination)
            .map_err(|error| format!("无法新建文件夹 {}：{error}", path_text(&destination)))?,
        "file" => {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|error| format!("无法新建文件 {}：{error}", path_text(&destination)))?;
        }
        _ => return Err(format!("不支持的目录项类型：{entry_type}")),
    }

    Ok(path_text(&destination))
}

pub(crate) fn rename_entry(
    runtime: &AppRuntime,
    instance_id: &str,
    source_path: &str,
    new_name: &str,
) -> Result<String, String> {
    let root = instance_root(runtime, instance_id)?;
    let source = managed_existing_path(&root, Path::new(source_path), false)?;
    let parent = source
        .parent()
        .ok_or_else(|| format!("无法确定目录项上级路径：{}", path_text(&source)))?;
    let destination = parent.join(validate_entry_name(new_name)?);
    if destination == source {
        return Ok(path_text(&source));
    }
    if destination.exists() {
        return Err(format!(
            "同名文件或文件夹已存在：{}",
            path_text(&destination)
        ));
    }

    fs::rename(&source, &destination).map_err(|error| {
        format!(
            "无法重命名 {} 为 {}：{error}",
            path_text(&source),
            path_text(&destination)
        )
    })?;
    Ok(path_text(&destination))
}

pub(crate) fn copy_entry(
    runtime: &AppRuntime,
    instance_id: &str,
    source_path: &str,
    target_directory: &str,
) -> Result<String, String> {
    let root = instance_root(runtime, instance_id)?;
    let source = managed_existing_path(&root, Path::new(source_path), false)?;
    let target = managed_existing_path(&root, Path::new(target_directory), true)?;
    ensure_directory(&target)?;

    if source.is_dir() && path_starts_with_case_insensitive(&target, &source) {
        return Err("不能把文件夹粘贴到自身或其子目录中".to_string());
    }

    let source_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("无法识别目录项名称：{}", path_text(&source)))?;
    let destination = available_copy_path(&target, source_name, source.is_dir())?;
    if let Err(error) = copy_entry_recursively(&source, &destination) {
        remove_partial_copy(&destination);
        return Err(error);
    }

    Ok(path_text(&destination))
}

pub(crate) fn delete_entry(
    runtime: &AppRuntime,
    instance_id: &str,
    target_path: &str,
) -> Result<(), String> {
    let root = instance_root(runtime, instance_id)?;
    let target = managed_existing_path(&root, Path::new(target_path), false)?;
    if target.is_dir() {
        fs::remove_dir_all(&target)
            .map_err(|error| format!("无法删除文件夹 {}：{error}", path_text(&target)))
    } else if target.is_file() {
        fs::remove_file(&target)
            .map_err(|error| format!("无法删除文件 {}：{error}", path_text(&target)))
    } else {
        Err(format!("不支持删除此类型的目录项：{}", path_text(&target)))
    }
}

fn instance_root(runtime: &AppRuntime, instance_id: &str) -> Result<PathBuf, String> {
    let instance = runtime.get_instance(instance_id)?;
    canonical_directory(Path::new(&instance.install_path), "实例目录")
}

fn list_instance_directory_in_root(
    root: &Path,
    requested: Option<&Path>,
) -> Result<InstanceDirectoryListing, String> {
    let root = canonical_directory(root, "实例目录")?;
    let current = managed_existing_path(&root, requested.unwrap_or(&root), true)?;
    ensure_directory(&current)?;
    let parent_path = current
        .parent()
        .filter(|parent| path_starts_with_case_insensitive(parent, &root))
        .filter(|parent| *parent != current)
        .map(path_text);

    let mut entries = fs::read_dir(&current)
        .map_err(|error| format!("无法读取实例目录 {}：{error}", path_text(&current)))?
        .filter_map(|entry| entry.ok())
        .map(|entry| build_entry(&entry.path()))
        .collect::<Result<Vec<_>, _>>()?;

    entries.sort_by(|left, right| {
        entry_sort_order(&left.entry_type)
            .cmp(&entry_sort_order(&right.entry_type))
            .then_with(|| {
                left.name
                    .to_ascii_lowercase()
                    .cmp(&right.name.to_ascii_lowercase())
            })
            .then_with(|| left.name.cmp(&right.name))
    });

    let total_entries = entries.len();
    let truncated = total_entries > MAX_DIRECTORY_ENTRIES;
    entries.truncate(MAX_DIRECTORY_ENTRIES);

    Ok(InstanceDirectoryListing {
        root_path: path_text(&root),
        current_path: path_text(&current),
        parent_path,
        entries,
        total_entries,
        truncated,
    })
}

fn build_entry(path: &Path) -> Result<InstanceFileEntry, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("无法读取目录项信息 {}：{error}", path_text(path)))?;
    let file_type = metadata.file_type();
    let entry_type = if file_type.is_symlink() {
        "other"
    } else if metadata.is_dir() {
        "directory"
    } else if metadata.is_file() {
        "file"
    } else {
        "other"
    };
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64);

    Ok(InstanceFileEntry {
        name: path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: path_text(path),
        entry_type: entry_type.to_string(),
        size_bytes: metadata.is_file().then_some(metadata.len()),
        modified_at,
        has_children: metadata.is_dir() && has_child_entry(path),
    })
}

fn managed_existing_path(root: &Path, path: &Path, allow_root: bool) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!("目录项路径必须是绝对路径：{}", path_text(path)));
    }
    let link_metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("无法读取目录项 {}：{error}", path_text(path)))?;
    if link_metadata.file_type().is_symlink() {
        return Err(format!("不允许操作符号链接：{}", path_text(path)));
    }
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("无法解析目录项 {}：{error}", path_text(path)))?;
    if !path_starts_with_case_insensitive(&canonical, root) {
        return Err(format!(
            "目录访问被拒绝：{}；仅允许操作当前实例目录 {} 内的文件",
            path_text(&canonical),
            path_text(root)
        ));
    }
    if !allow_root && canonical == root {
        return Err("不允许重命名、复制或删除实例根目录".to_string());
    }
    Ok(canonical)
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("无法解析{label} {}：{error}", path_text(path)))?;
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(format!("{label}不是文件夹：{}", path_text(&canonical)))
    }
}

fn ensure_directory(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(format!("目标不是文件夹：{}", path_text(path)))
    }
}

fn validate_entry_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("文件或文件夹名称不能为空".to_string());
    }
    if name == "." || name == ".." {
        return Err("文件或文件夹名称不能是 . 或 ..".to_string());
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err("文件或文件夹名称不能以句点或空格结尾".to_string());
    }
    if name
        .chars()
        .any(|character| character.is_control() || r#"\/:*?"<>|"#.contains(character))
    {
        return Err("文件或文件夹名称包含 Windows 不允许的字符".to_string());
    }
    let base_name = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    let reserved = matches!(base_name.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (base_name.len() == 4
            && (base_name.starts_with("COM") || base_name.starts_with("LPT"))
            && base_name
                .chars()
                .last()
                .is_some_and(|character| matches!(character, '1'..='9')));
    if reserved {
        return Err(format!("文件或文件夹名称 {name} 是 Windows 保留名称"));
    }
    Ok(name)
}

fn available_copy_path(
    target_directory: &Path,
    source_name: &str,
    source_is_directory: bool,
) -> Result<PathBuf, String> {
    let original = target_directory.join(source_name);
    if !original.exists() {
        return Ok(original);
    }

    for index in 1..=10_000 {
        let suffix = if index == 1 {
            " - 副本".to_string()
        } else {
            format!(" - 副本 ({index})")
        };
        let candidate_name = if source_is_directory {
            format!("{source_name}{suffix}")
        } else {
            let path = Path::new(source_name);
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(source_name);
            match path.extension().and_then(|value| value.to_str()) {
                Some(extension) if !extension.is_empty() => {
                    format!("{stem}{suffix}.{extension}")
                }
                _ => format!("{stem}{suffix}"),
            }
        };
        let candidate = target_directory.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("无法为粘贴内容生成不冲突的副本名称".to_string())
}

fn copy_entry_recursively(source: &Path, destination: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| format!("无法读取复制源 {}：{error}", path_text(source)))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("不允许复制符号链接：{}", path_text(source)));
    }
    if metadata.is_file() {
        fs::copy(source, destination).map_err(|error| {
            format!(
                "无法复制文件 {} 到 {}：{error}",
                path_text(source),
                path_text(destination)
            )
        })?;
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!("不支持复制此类型的目录项：{}", path_text(source)));
    }

    fs::create_dir(destination)
        .map_err(|error| format!("无法创建副本目录 {}：{error}", path_text(destination)))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("无法读取目录 {}：{error}", path_text(source)))?
    {
        let entry = entry.map_err(|error| format!("读取待复制目录项失败：{error}"))?;
        copy_entry_recursively(&entry.path(), &destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn remove_partial_copy(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn has_child_entry(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|entries| entries.flatten().next().is_some())
        .unwrap_or(false)
}

fn entry_sort_order(entry_type: &str) -> u8 {
    match entry_type {
        "directory" => 0,
        "file" => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 目录列表按文件夹优先并限制在实例根目录() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let root = temp.path().join("实例");
        fs::create_dir_all(root.join("Saved")).expect("创建文件夹");
        fs::write(root.join("Server.txt"), "ASA").expect("写入文件");

        let listing = list_instance_directory_in_root(&root, None).expect("读取实例目录");

        assert_eq!(listing.entries.len(), 2);
        assert_eq!(listing.entries[0].entry_type, "directory");
        assert_eq!(listing.entries[1].entry_type, "file");

        let outside = temp.path().join("外部");
        fs::create_dir_all(&outside).expect("创建外部目录");
        let error = list_instance_directory_in_root(&root, Some(&outside))
            .expect_err("应拒绝实例目录之外的路径");
        assert!(error.contains("目录访问被拒绝"));
    }

    #[test]
    fn 复制文件自动生成副本名称且不覆盖原文件() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let root = temp.path().join("实例");
        fs::create_dir_all(&root).expect("创建实例目录");
        let source = root.join("Game.ini");
        fs::write(&source, "配置").expect("写入源文件");

        let destination = available_copy_path(&root, "Game.ini", false).expect("生成副本路径");
        copy_entry_recursively(&source, &destination).expect("复制文件");

        assert_eq!(
            destination.file_name().and_then(|value| value.to_str()),
            Some("Game - 副本.ini")
        );
        assert_eq!(fs::read_to_string(source).expect("读取源文件"), "配置");
        assert_eq!(
            fs::read_to_string(destination).expect("读取副本文件"),
            "配置"
        );
    }

    #[test]
    fn 拒绝_windows_非法名称和保留名称() {
        for name in ["", "..", "bad/name", "bad:name", "CON", "LPT1.txt", "末尾."] {
            assert!(validate_entry_name(name).is_err(), "{name:?} 应被拒绝");
        }
        assert_eq!(
            validate_entry_name("GameUserSettings.ini"),
            Ok("GameUserSettings.ini")
        );
        assert_eq!(validate_entry_name("新建文件夹"), Ok("新建文件夹"));
    }
}
