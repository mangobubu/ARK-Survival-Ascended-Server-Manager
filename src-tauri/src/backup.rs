use crate::{
    app_state::current_timestamp_text,
    ark_config::saved_dir,
    models::{BackupItem, ServerInstance},
};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

pub fn create_instance_backup(
    backup_root: &Path,
    instance: &ServerInstance,
) -> Result<BackupItem, String> {
    let source = saved_dir(instance);
    if !source.is_dir() {
        return Err(format!(
            "未找到可备份的存档目录，请确认实例已安装并运行过：{}",
            source.display()
        ));
    }

    let instance_backup_dir = backup_root.join(sanitize_filename(&instance.name));
    fs::create_dir_all(&instance_backup_dir).map_err(|error| {
        format!(
            "无法创建实例备份目录 {}：{error}",
            instance_backup_dir.display()
        )
    })?;

    let created_at = current_timestamp_text();
    let backup_path = instance_backup_dir.join(format!(
        "{}-{}-{}.zip",
        sanitize_filename(&instance.name),
        instance.id,
        created_at
    ));
    zip_directory(&source, &backup_path)?;

    let size_bytes = fs::metadata(&backup_path)
        .map_err(|error| format!("无法读取备份文件信息 {}：{error}", backup_path.display()))?
        .len();

    Ok(BackupItem {
        instance_id: instance.id.clone(),
        instance_name: instance.name.clone(),
        path: backup_path.to_string_lossy().into_owned(),
        size_bytes,
        created_at,
    })
}

pub fn list_instance_backups(
    backup_root: &Path,
    instance: &ServerInstance,
) -> Result<Vec<BackupItem>, String> {
    let instance_backup_dir = backup_root.join(sanitize_filename(&instance.name));
    if !instance_backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&instance_backup_dir)
        .map_err(|error| format!("无法读取备份目录 {}：{error}", instance_backup_dir.display()))?
    {
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

pub fn restore_instance_backup(
    instance: &ServerInstance,
    backup_path: &Path,
) -> Result<(), String> {
    if !backup_path.is_file() {
        return Err(format!("备份文件不存在：{}", backup_path.display()));
    }
    let target = saved_dir(instance);
    fs::create_dir_all(&target)
        .map_err(|error| format!("无法创建存档目录 {}：{error}", target.display()))?;
    unzip_to_directory(backup_path, &target)
}

fn zip_directory(source: &Path, destination: &Path) -> Result<(), String> {
    let file = File::create(destination)
        .map_err(|error| format!("无法创建备份文件 {}：{error}", destination.display()))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    add_directory_entries(source, source, &mut writer, options)?;
    writer
        .finish()
        .map_err(|error| format!("无法完成备份压缩 {}：{error}", destination.display()))?;
    Ok(())
}

fn add_directory_entries(
    root: &Path,
    current: &Path,
    writer: &mut ZipWriter<File>,
    options: SimpleFileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("无法读取目录 {}：{error}", current.display()))?
    {
        let entry = entry.map_err(|error| format!("读取目录项失败：{error}"))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("生成备份相对路径失败：{error}"))?;
        let archive_name = relative.to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            writer
                .add_directory(format!("{archive_name}/"), options)
                .map_err(|error| format!("写入备份目录失败：{error}"))?;
            add_directory_entries(root, &path, writer, options)?;
        } else {
            writer
                .start_file(archive_name, options)
                .map_err(|error| format!("写入备份文件头失败：{error}"))?;
            let mut input = File::open(&path)
                .map_err(|error| format!("打开待备份文件 {} 失败：{error}", path.display()))?;
            io::copy(&mut input, writer)
                .map_err(|error| format!("压缩备份文件 {} 失败：{error}", path.display()))?;
        }
    }
    Ok(())
}

fn unzip_to_directory(archive_path: &Path, target: &Path) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("无法打开备份文件 {}：{error}", archive_path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("备份压缩包无效：{error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("读取备份压缩包条目失败：{error}"))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "备份压缩包包含不安全路径，已终止恢复".to_string())?;
        let output_path = target.join(enclosed);
        if entry.is_dir() {
            fs::create_dir_all(&output_path)
                .map_err(|error| format!("创建恢复目录失败：{error}"))?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("创建恢复目录失败：{error}"))?;
        }
        let mut output = File::create(&output_path)
            .map_err(|error| format!("创建恢复文件 {} 失败：{error}", output_path.display()))?;
        io::copy(&mut entry, &mut output)
            .map_err(|error| format!("恢复文件 {} 失败：{error}", output_path.display()))?;
        output
            .flush()
            .map_err(|error| format!("刷新恢复文件 {} 失败：{error}", output_path.display()))?;
    }
    Ok(())
}

fn sanitize_filename(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => ch,
        })
        .collect();
    if sanitized.trim().is_empty() {
        "未命名实例".to_string()
    } else {
        sanitized
    }
}

#[allow(dead_code)]
fn normalize(path: &Path) -> PathBuf {
    path.components().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ServerStatus;

    fn instance(root: &Path) -> ServerInstance {
        ServerInstance {
            id: "asa-test".to_string(),
            name: "测试/实例".to_string(),
            map: "The Island".to_string(),
            map_code: "TheIsland_WP".to_string(),
            mode: "PvE".to_string(),
            status: ServerStatus::Stopped,
            game_port: 7777,
            query_port: 27015,
            players: 0,
            max_players: 30,
            install_path: root.to_string_lossy().into_owned(),
            rcon_port: 32330,
            cluster_id: "Cluster".to_string(),
            description: String::new(),
            pid: None,
            last_started_at: None,
            last_stopped_at: None,
            version_state: "已安装".to_string(),
            last_error: None,
        }
    }

    #[test]
    fn 创建并恢复备份() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let install_root = temp.path().join("server");
        let saved = install_root.join("ShooterGame").join("Saved");
        fs::create_dir_all(&saved).expect("创建存档目录");
        fs::write(saved.join("world.ark"), "data").expect("写入存档");
        let backup_root = temp.path().join("backups");
        let instance = instance(&install_root);

        let backup = create_instance_backup(&backup_root, &instance).expect("创建备份");
        fs::remove_file(saved.join("world.ark")).expect("删除源文件");
        restore_instance_backup(&instance, Path::new(&backup.path)).expect("恢复备份");

        assert_eq!(fs::read_to_string(saved.join("world.ark")).unwrap(), "data");
    }
}

