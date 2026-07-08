use std::path::{Path, PathBuf};

pub(super) fn sanitize_filename(value: &str) -> String {
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
pub(super) fn normalize(path: &Path) -> PathBuf {
    path.components().collect()
}
