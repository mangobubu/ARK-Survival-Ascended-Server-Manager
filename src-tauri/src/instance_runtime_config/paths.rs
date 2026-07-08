use std::path::PathBuf;

pub(crate) fn normalize_path_text(path: &str) -> PathBuf {
    PathBuf::from(clean_windows_path_text(path.trim().trim_matches('"')))
}

fn clean_windows_path_text(value: &str) -> String {
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        return format!("\\\\{rest}");
    }
    if let Some(rest) = value.strip_prefix("\\\\?\\") {
        return rest.to_string();
    }
    value.to_string()
}
