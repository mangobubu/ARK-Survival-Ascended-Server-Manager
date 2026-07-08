use super::types::ParsedSteamCmdProgress;

pub(crate) fn percent_from_bytes(downloaded_bytes: u64, total_bytes: u64) -> f64 {
    if total_bytes == 0 {
        return 0.0;
    }
    ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0)
}

fn parse_percent(value: &str) -> Option<f64> {
    let number = value.split_whitespace().next()?.parse::<f64>().ok()?;
    Some(number.clamp(0.0, 100.0))
}

fn parse_byte_count(value: &str) -> Option<u64> {
    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

fn normalize_steamcmd_phase(state: &str) -> String {
    let lower = state.to_ascii_lowercase();
    if lower.contains("download") {
        "downloading"
    } else if lower.contains("verify") || lower.contains("validat") {
        "verifying"
    } else if lower.contains("prealloc") {
        "preallocating"
    } else if lower.contains("commit") {
        "committing"
    } else {
        "running"
    }
    .to_string()
}

fn steamcmd_progress_message(phase: &str, state: &str) -> String {
    match phase {
        "downloading" => "SteamCMD 正在下载服务端文件".to_string(),
        "verifying" => "SteamCMD 正在校验服务端文件".to_string(),
        "preallocating" => "SteamCMD 正在预分配文件".to_string(),
        "committing" => "SteamCMD 正在提交更新".to_string(),
        _ if !state.trim().is_empty() => format!("SteamCMD {}", state.trim()),
        _ => "SteamCMD 正在处理更新".to_string(),
    }
}

pub(crate) fn parse_steamcmd_progress_line(line: &str) -> Option<ParsedSteamCmdProgress> {
    let lower_line = line.to_ascii_lowercase();
    let progress_index = lower_line.find("progress:")?;
    let after_progress = &line[progress_index + "progress:".len()..];
    let open_paren = after_progress.find('(')?;
    let close_paren = after_progress[open_paren + 1..].find(')')? + open_paren + 1;
    let percent = parse_percent(&after_progress[..open_paren]);
    let byte_pair = &after_progress[open_paren + 1..close_paren];
    let (downloaded_text, total_text) = byte_pair.split_once('/')?;
    let downloaded_bytes = parse_byte_count(downloaded_text)?;
    let total_bytes = parse_byte_count(total_text).filter(|total| *total > 0);

    let state_source = line[..progress_index].trim().trim_end_matches(',');
    let state = state_source
        .rsplit_once(')')
        .map(|(_, state)| state)
        .unwrap_or(state_source)
        .trim()
        .trim_start_matches(',')
        .trim();
    let phase = normalize_steamcmd_phase(state);
    let message = steamcmd_progress_message(&phase, state);
    let percent = total_bytes
        .map(|total| percent_from_bytes(downloaded_bytes, total))
        .or(percent);

    Some(ParsedSteamCmdProgress {
        phase,
        message,
        percent,
        downloaded_bytes,
        total_bytes,
    })
}
