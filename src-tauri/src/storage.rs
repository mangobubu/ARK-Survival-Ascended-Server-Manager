use std::{fs, path::Path};

fn ensure_directory(path: &Path, label: &str) -> Result<(), String> {
    if path.exists() {
        return if path.is_dir() {
            Ok(())
        } else {
            Err(format!("{label}不是目录：{}", path.display()))
        };
    }

    fs::create_dir_all(path).map_err(|error| format!("无法创建{label} {}：{error}", path.display()))
}

fn ensure_storage_paths(
    server_storage_path: &str,
    backup_storage_path: &str,
) -> Result<(), String> {
    if server_storage_path.trim().is_empty() {
        return Err("服务器存储目录不能为空".to_string());
    }
    if backup_storage_path.trim().is_empty() {
        return Err("备份存储目录不能为空".to_string());
    }

    ensure_directory(Path::new(server_storage_path), "服务器存储目录")?;
    ensure_directory(Path::new(backup_storage_path), "备份存储目录")?;
    Ok(())
}

#[tauri::command]
pub fn ensure_storage_directories(
    server_storage_path: String,
    backup_storage_path: String,
) -> Result<(), String> {
    ensure_storage_paths(&server_storage_path, &backup_storage_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn 自动创建服务器与备份目录() {
        let temp = tempdir().expect("创建临时目录");
        let server_path = temp.path().join("服务器").join("实例");
        let backup_path = temp.path().join("备份").join("每日");

        ensure_storage_paths(
            server_path.to_str().expect("服务器目录路径"),
            backup_path.to_str().expect("备份目录路径"),
        )
        .expect("创建存储目录");

        assert!(server_path.is_dir());
        assert!(backup_path.is_dir());
    }

    #[test]
    fn 已有目录可重复检查() {
        let temp = tempdir().expect("创建临时目录");
        let server_path = temp.path().join("服务器");
        let backup_path = temp.path().join("备份");
        fs::create_dir_all(&server_path).expect("创建服务器目录");
        fs::create_dir_all(&backup_path).expect("创建备份目录");

        for _ in 0..2 {
            ensure_storage_paths(
                server_path.to_str().expect("服务器目录路径"),
                backup_path.to_str().expect("备份目录路径"),
            )
            .expect("重复检查存储目录");
        }
    }

    #[test]
    fn 拒绝空目录配置() {
        let temp = tempdir().expect("创建临时目录");
        let backup_path = temp.path().join("备份");

        let error = ensure_storage_paths("  ", backup_path.to_str().expect("备份目录路径"))
            .expect_err("应拒绝空服务器目录");

        assert_eq!(error, "服务器存储目录不能为空");
        assert!(!backup_path.exists());
    }

    #[test]
    fn 拒绝指向文件的路径() {
        let temp = tempdir().expect("创建临时目录");
        let server_path = temp.path().join("服务器");
        let backup_file = temp.path().join("备份文件");
        fs::create_dir_all(&server_path).expect("创建服务器目录");
        File::create(&backup_file).expect("创建占位文件");

        let error = ensure_storage_paths(
            server_path.to_str().expect("服务器目录路径"),
            backup_file.to_str().expect("备份路径"),
        )
        .expect_err("应拒绝文件路径");

        assert!(error.contains("备份存储目录不是目录"));
    }
}
