use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const SERVER_LOG_DEDUPE_WINDOW: Duration = Duration::from_millis(1_500);
const SERVER_LOG_DEDUPE_LIMIT: usize = 256;

pub(crate) type SharedServerLogDeduper = Arc<Mutex<VecDeque<(Instant, String)>>>;

pub(crate) fn new_server_log_deduper() -> SharedServerLogDeduper {
    Arc::new(Mutex::new(VecDeque::with_capacity(SERVER_LOG_DEDUPE_LIMIT)))
}

pub(crate) fn classify_server_log_level(line: &str, fallback_level: &'static str) -> &'static str {
    let normalized = line.to_ascii_lowercase();
    if is_error_server_log(&normalized) {
        return "error";
    }
    if is_warn_server_log(&normalized) {
        return "warn";
    }
    if is_plain_server_log(&normalized) {
        return "info";
    }
    fallback_level
}

fn is_plain_server_log(normalized: &str) -> bool {
    normalized.contains(" i ")
        || normalized.contains(" d ")
        || normalized.contains(" info/")
        || normalized.contains(" debug/")
        || normalized.contains(" log")
        || normalized.starts_with("none")
        || normalized.starts_with("cfcore")
}

fn is_warn_server_log(normalized: &str) -> bool {
    normalized.contains(" w ")
        || normalized.contains(" warn/")
        || normalized.contains(" warning")
        || normalized.contains("warning:")
        || normalized.contains("couldn't")
        || normalized.contains("could not")
        || normalized.contains("failed")
        || normalized.contains("failure")
}

fn is_error_server_log(normalized: &str) -> bool {
    normalized.contains(" error/")
        || normalized.contains(" error:")
        || normalized.contains(" error ")
        || normalized.starts_with("error")
        || normalized.contains(" fatal")
        || normalized.starts_with("fatal")
        || normalized.contains(" crash")
        || normalized.starts_with("crash")
        || normalized.contains("exception")
}

pub(crate) fn should_skip_duplicate_server_log(
    deduper: &SharedServerLogDeduper,
    line: &str,
) -> bool {
    let now = Instant::now();
    let Ok(mut recent_lines) = deduper.lock() else {
        return false;
    };

    while recent_lines
        .front()
        .is_some_and(|(seen_at, _)| now.duration_since(*seen_at) > SERVER_LOG_DEDUPE_WINDOW)
    {
        recent_lines.pop_front();
    }

    if recent_lines.iter().any(|(_, seen_line)| seen_line == line) {
        return true;
    }

    while recent_lines.len() >= SERVER_LOG_DEDUPE_LIMIT {
        recent_lines.pop_front();
    }
    recent_lines.push_back((now, line.to_string()));
    false
}
