use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn locate_config_dir(root: &Path) -> PathBuf {
    candidate_config_dirs_with_ancestors(root)
        .into_iter()
        .find(|path| path.join("GameUserSettings.ini").is_file() || path.join("Game.ini").is_file())
        .or_else(|| search_config_dir(root, 7))
        .unwrap_or_else(|| {
            root.join("ShooterGame")
                .join("Saved")
                .join("Config")
                .join("WindowsServer")
        })
}

fn candidate_config_dirs_with_ancestors(root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut current = Some(root);

    for _ in 0..=5 {
        let Some(path) = current else {
            break;
        };
        candidates.extend(candidate_config_dirs(path));
        current = path.parent();
    }

    dedupe_paths(candidates)
}

fn candidate_config_dirs(root: &Path) -> Vec<PathBuf> {
    vec![
        root.to_path_buf(),
        root.join("WindowsServer"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsServer"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("Win64"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("Windows"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsNoEditor"),
        root.join("Saved").join("Config").join("WindowsServer"),
        root.join("Saved").join("Config").join("Win64"),
        root.join("Saved").join("Config").join("Windows"),
        root.join("Saved").join("Config").join("WindowsNoEditor"),
        root.join("Config").join("WindowsServer"),
        root.join("Config").join("Win64"),
        root.join("Config").join("Windows"),
        root.join("Config").join("WindowsNoEditor"),
    ]
}

fn search_config_dir(root: &Path, max_depth: usize) -> Option<PathBuf> {
    let mut pending = vec![(root.to_path_buf(), 0_usize)];

    while let Some((directory, depth)) = pending.pop() {
        if directory.join("GameUserSettings.ini").is_file() || directory.join("Game.ini").is_file()
        {
            return Some(directory);
        }
        if depth >= max_depth {
            continue;
        }

        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        let mut children = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        children.sort_by_key(|path| config_search_priority(path));
        for child in children.into_iter().rev() {
            pending.push((child, depth + 1));
        }
    }

    None
}

fn config_search_priority(path: &Path) -> u8 {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match name.as_str() {
        "shootergame" => 0,
        "saved" => 1,
        "config" => 2,
        "windowsserver" => 3,
        "win64" | "windows" | "windowsnoeditor" => 4,
        _ => 9,
    }
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(normalize_path_for_dedupe(path)))
        .collect()
}

fn normalize_path_for_dedupe(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}
