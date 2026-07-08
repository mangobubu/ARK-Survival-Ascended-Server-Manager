use std::path::Path;

pub(crate) fn open_directory(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .map_err(|error| format!("无法打开目录 {}：{error}", path.display()))?;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        Err("打开目录目前仅支持 Windows".to_string())
    }
}
