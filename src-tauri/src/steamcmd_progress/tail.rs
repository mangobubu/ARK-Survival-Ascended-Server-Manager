use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

fn trim_detail(content: &str, fallback: &str) -> String {
    let detail = content
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or(fallback);
    let mut text = detail.to_string();
    if text.len() > 500 {
        text.truncate(500);
    }
    text
}

pub(crate) fn is_retryable_steamcmd_configuration_error(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("missing configuration")
        && (lower.contains("failed to install app") || lower.contains("app_update"))
}

fn is_steamcmd_failure_detail(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("error!")
        || lower.contains("failed to install app")
        || lower.contains("missing configuration")
}

pub(crate) fn remember_tail(tail: &Arc<Mutex<VecDeque<String>>>, line: &str) {
    if let Ok(mut tail) = tail.lock() {
        if tail.len() >= 24 {
            tail.pop_front();
        }
        tail.push_back(line.to_string());
    }
}

pub(crate) fn tail_detail(tail: &Arc<Mutex<VecDeque<String>>>, fallback: &str) -> String {
    let detail = tail.lock().ok().and_then(|tail| {
        tail.iter()
            .rev()
            .find(|line| is_steamcmd_failure_detail(line))
            .or_else(|| tail.iter().rev().find(|line| !line.trim().is_empty()))
            .cloned()
    });
    let detail = detail.unwrap_or_else(|| fallback.to_string());
    trim_detail(&detail, fallback)
}
