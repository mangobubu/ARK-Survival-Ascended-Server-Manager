use std::path::{Path, PathBuf};

pub(crate) fn infer_install_path(selected_path: &Path, config_dir: &Path) -> PathBuf {
    if path_tail_matches_config_platform(config_dir, &["shootergame", "saved", "config"]) {
        return ancestor(config_dir, 4).unwrap_or_else(|| selected_path.to_path_buf());
    }

    if path_tail_matches_config_platform(config_dir, &["saved", "config"]) {
        return ancestor(config_dir, 3).unwrap_or_else(|| selected_path.to_path_buf());
    }

    selected_path.to_path_buf()
}

fn path_tail_matches_config_platform(path: &Path, expected_prefix: &[&str]) -> bool {
    let names = path_tail_names(path, expected_prefix.len() + 1);
    if names.len() != expected_prefix.len() + 1 {
        return false;
    }
    let prefix_matches = names[..expected_prefix.len()]
        .iter()
        .map(String::as_str)
        .eq(expected_prefix.iter().copied());
    prefix_matches && is_config_platform_dir(&names[expected_prefix.len()])
}

fn is_config_platform_dir(name: &str) -> bool {
    matches!(
        name,
        "windowsserver" | "win64" | "windows" | "windowsnoeditor"
    )
}

fn path_tail_names(path: &Path, count: usize) -> Vec<String> {
    let mut names = path
        .iter()
        .rev()
        .take(count)
        .map(|part| part.to_string_lossy().to_ascii_lowercase())
        .collect::<Vec<_>>();
    names.reverse();
    names
}

fn ancestor(path: &Path, levels: usize) -> Option<PathBuf> {
    let mut current = path;
    for _ in 0..levels {
        current = current.parent()?;
    }
    Some(current.to_path_buf())
}
