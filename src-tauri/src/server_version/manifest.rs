use super::types::ManifestProgress;

fn acf_string(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let mut parts = line.split('"');
        let _ = parts.next()?;
        let found_key = parts.next()?;
        let _ = parts.next()?;
        let value = parts.next()?;
        let value = value.trim();
        if found_key.eq_ignore_ascii_case(key) && !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn acf_u64(content: &str, key: &str) -> Option<u64> {
    acf_string(content, key).and_then(|value| value.parse::<u64>().ok())
}

pub(crate) fn parse_manifest_progress(content: &str) -> Option<ManifestProgress> {
    let bytes_to_download = acf_u64(content, "BytesToDownload").unwrap_or(0);
    let bytes_downloaded = acf_u64(content, "BytesDownloaded").unwrap_or(0);
    let bytes_to_stage = acf_u64(content, "BytesToStage").unwrap_or(0);
    let bytes_staged = acf_u64(content, "BytesStaged").unwrap_or(0);

    if bytes_to_download > 0 && bytes_downloaded < bytes_to_download {
        return Some(ManifestProgress {
            phase: "downloading".to_string(),
            downloaded_bytes: bytes_downloaded,
            total_bytes: Some(bytes_to_download),
        });
    }

    if bytes_to_stage > 0 {
        return Some(ManifestProgress {
            phase: if bytes_staged < bytes_to_stage {
                "committing"
            } else {
                "verifying"
            }
            .to_string(),
            downloaded_bytes: bytes_staged.min(bytes_to_stage),
            total_bytes: Some(bytes_to_stage),
        });
    }

    None
}
