use std::{cmp::Reverse, fs, path::Path};

use crate::models::{BackupItem, ServerInstance};

use super::naming::sanitize_filename;

pub fn list_instance_backups(
    backup_root: &Path,
    instance: &ServerInstance,
) -> Result<Vec<BackupItem>, String> {
    let instance_backup_dir = backup_root.join(sanitize_filename(&instance.name));
    if !instance_backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&instance_backup_dir).map_err(|error| {
        format!(
            "无法读取备份目录 {}：{error}",
            instance_backup_dir.display()
        )
    })? {
        let entry = entry.map_err(|error| format!("读取备份目录项失败：{error}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("zip") {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| format!("无法读取备份文件信息 {}：{error}", path.display()))?;
        let created_at = metadata
            .created()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|| "0".to_string());
        backups.push(BackupItem {
            instance_id: instance.id.clone(),
            instance_name: instance.name.clone(),
            path: path.to_string_lossy().into_owned(),
            size_bytes: metadata.len(),
            created_at,
        });
    }
    backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(backups)
}

pub fn prune_instance_backups(
    backup_root: &Path,
    instance: &ServerInstance,
    max_retention: u32,
) -> Result<usize, String> {
    let instance_backup_dir = backup_root.join(sanitize_filename(&instance.name));
    if !instance_backup_dir.exists() {
        return Ok(0);
    }

    let mut backup_files = Vec::new();
    for entry in fs::read_dir(&instance_backup_dir).map_err(|error| {
        format!(
            "无法读取备份目录 {}：{error}",
            instance_backup_dir.display()
        )
    })? {
        let entry = entry.map_err(|error| format!("读取备份目录项失败：{error}"))?;
        let path = entry.path();
        if !path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
        {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| format!("无法读取备份文件信息 {}：{error}", path.display()))?;
        if !metadata.is_file() {
            continue;
        }
        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        backup_files.push((created_at, path));
    }

    backup_files.sort_by_key(|right| Reverse(right.0));
    let mut removed = 0_usize;
    for (_, path) in backup_files.into_iter().skip(max_retention as usize) {
        fs::remove_file(&path)
            .map_err(|error| format!("无法删除过期备份 {}：{error}", path.display()))?;
        removed += 1;
    }
    Ok(removed)
}
