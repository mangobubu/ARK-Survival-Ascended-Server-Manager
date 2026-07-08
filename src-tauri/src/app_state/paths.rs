use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub(super) fn resolve_app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(any(test, debug_assertions))]
    {
        if let Some(data_dir) = debug_app_data_dir_override() {
            return Ok(data_dir);
        }
    }

    app.path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录：{error}"))
}

#[cfg(any(test, debug_assertions))]
pub(super) fn debug_app_data_dir_override() -> Option<PathBuf> {
    std::env::var_os("ASA_MANAGER_DATA_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
