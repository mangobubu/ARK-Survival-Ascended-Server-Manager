use std::path::Path;

pub(crate) fn path_text(path: &Path) -> String {
    clean_windows_path_text(&path.to_string_lossy())
}

pub(super) fn clean_windows_path_text(value: &str) -> String {
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        return format!("\\\\{rest}");
    }
    if let Some(rest) = value.strip_prefix("\\\\?\\") {
        return rest.to_string();
    }
    value.to_string()
}
