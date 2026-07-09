use crate::app_state::AppRuntime;
use std::path::{Path, PathBuf};

pub fn ensure_managed_path_allowed(runtime: &AppRuntime, path: &Path) -> Result<PathBuf, String> {
    let target = canonicalize_existing_path(path)?;
    for root in managed_roots(runtime)? {
        if root.as_os_str().is_empty() || !root.exists() {
            continue;
        }
        let root = canonicalize_existing_path(&root)?;
        if path_starts_with_case_insensitive(&target, &root) {
            return Ok(target);
        }
    }

    Err(format!(
        "路径访问被拒绝：{}；仅允许访问服务器存储目录、备份目录或已登记实例目录",
        target.display()
    ))
}

pub fn ensure_backup_path_allowed(runtime: &AppRuntime, path: &Path) -> Result<PathBuf, String> {
    let target = canonicalize_existing_path(path)?;
    let settings = runtime.settings()?;
    let backup_root = PathBuf::from(settings.backup_storage_path);
    if backup_root.as_os_str().is_empty() {
        return Err("备份存储目录尚未配置，无法校验备份路径边界".to_string());
    }
    let backup_root = canonicalize_existing_path(&backup_root)?;
    if path_starts_with_case_insensitive(&target, &backup_root) {
        Ok(target)
    } else {
        Err(format!(
            "备份路径访问被拒绝：{}；仅允许使用备份存储目录内的文件",
            target.display()
        ))
    }
}

fn managed_roots(runtime: &AppRuntime) -> Result<Vec<PathBuf>, String> {
    let settings = runtime.settings()?;
    let mut roots = vec![
        PathBuf::from(settings.server_storage_path),
        PathBuf::from(settings.backup_storage_path),
    ];
    roots.extend(
        runtime
            .list_instances()?
            .into_iter()
            .map(|instance| PathBuf::from(instance.install_path)),
    );
    Ok(roots)
}

fn canonicalize_existing_path(path: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(path).map_err(|error| format!("无法解析路径 {}：{error}", path.display()))
}

pub(crate) fn path_starts_with_case_insensitive(path: &Path, root: &Path) -> bool {
    let path_text = path.to_string_lossy().to_ascii_lowercase();
    let root_exact = root.to_string_lossy().to_ascii_lowercase();
    let mut root_prefix = root_exact.clone();
    if !root_prefix.ends_with(std::path::MAIN_SEPARATOR) {
        root_prefix.push(std::path::MAIN_SEPARATOR);
    }
    path_text == root_exact || path_text.starts_with(&root_prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 路径前缀比较区分完整目录边界() {
        assert!(path_starts_with_case_insensitive(
            Path::new("D:\\ASA\\Server\\Saved"),
            Path::new("D:\\ASA\\Server")
        ));
        assert!(!path_starts_with_case_insensitive(
            Path::new("D:\\ASA\\Server2\\Saved"),
            Path::new("D:\\ASA\\Server")
        ));
    }
}
