use std::{fs, path::Path, time::SystemTime};

const MAPS: &[(&str, &str)] = &[
    ("TheIsland_WP", "The Island"),
    ("ScorchedEarth_WP", "Scorched Earth"),
    ("TheCenter_WP", "The Center"),
    ("Aberration_WP", "Aberration"),
    ("Extinction_WP", "Extinction"),
    ("Astraeos_WP", "Astraeos"),
    ("Ragnarok_WP", "Ragnarok"),
    ("Valguero_WP", "Valguero"),
    ("LostColony_WP", "Lost Colony"),
];

pub(crate) fn infer_map_from_saved_arks(root: &Path) -> Option<(String, String)> {
    let save_dirs = [
        root.join("ShooterGame").join("Saved").join("SavedArks"),
        root.join("ShooterGame")
            .join("Saved")
            .join("SavedArksLocal"),
        root.join("Saved").join("SavedArks"),
        root.join("Saved").join("SavedArksLocal"),
    ];

    let mut best_match: Option<(String, String, SystemTime)> = None;
    for directory in save_dirs {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            let lower_name = file_name.to_ascii_lowercase();
            for (code, name) in MAPS {
                if !lower_name.starts_with(&code.to_ascii_lowercase())
                    || !lower_name.ends_with(".ark")
                {
                    continue;
                }
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let should_replace = best_match
                    .as_ref()
                    .map(|(_, _, current)| modified > *current)
                    .unwrap_or(true);
                if should_replace {
                    best_match = Some(((*code).to_string(), (*name).to_string(), modified));
                }
            }
        }
    }

    best_match.map(|(code, name, _)| (code, name))
}
