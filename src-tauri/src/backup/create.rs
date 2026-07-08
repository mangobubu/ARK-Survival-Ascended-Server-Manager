use std::{fs, path::Path};

use crate::{
    app_state::current_timestamp_text,
    ark_config::saved_dir,
    models::{BackupItem, ServerInstance},
};

use super::{archive::zip_directory, naming::sanitize_filename};

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
