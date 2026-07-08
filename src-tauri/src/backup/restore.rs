use std::{fs, path::Path};

use crate::{ark_config::saved_dir, models::ServerInstance};

use super::archive::unzip_to_directory;

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
