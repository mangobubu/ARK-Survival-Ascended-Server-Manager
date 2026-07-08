use std::time::{Duration, Instant};

use super::{parser::percent_from_bytes, types::TransferSnapshot};

#[derive(Default)]
pub(crate) struct SteamCmdProgressTracker {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    bytes_per_second: u64,
    percent: Option<f64>,
    last_sample: Option<(Instant, u64)>,
    last_emit_at: Option<Instant>,
    last_log_phase: Option<String>,
    last_log_percent_bucket: Option<u8>,
}

impl SteamCmdProgressTracker {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update(
        &mut self,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        percent: Option<f64>,
        phase: &str,
        progress_message: &str,
        now: Instant,
        force_emit: bool,
    ) -> (TransferSnapshot, bool, Option<String>) {
        self.downloaded_bytes = downloaded_bytes;
        self.total_bytes = total_bytes;
        self.percent = match total_bytes {
            Some(total) if total > 0 => Some(percent_from_bytes(downloaded_bytes, total)),
            _ => percent,
        };

        let should_sample = self
            .last_sample
            .map(|(last_at, last_bytes)| {
                downloaded_bytes < last_bytes
                    || now.duration_since(last_at) >= Duration::from_millis(250)
            })
            .unwrap_or(true);

        if should_sample {
            if let Some((last_at, last_bytes)) = self.last_sample {
                let elapsed = now.duration_since(last_at).as_secs_f64();
                if elapsed > 0.0 && downloaded_bytes >= last_bytes {
                    self.bytes_per_second =
                        ((downloaded_bytes - last_bytes) as f64 / elapsed).round() as u64;
                } else if downloaded_bytes < last_bytes {
                    self.bytes_per_second = 0;
                }
            }
            self.last_sample = Some((now, downloaded_bytes));
        }

        let reached_total = total_bytes
            .map(|total| total > 0 && downloaded_bytes >= total)
            .unwrap_or(false);
        let should_emit = force_emit
            || self
                .last_emit_at
                .map(|last_at| now.duration_since(last_at) >= Duration::from_millis(200))
                .unwrap_or(true)
            || reached_total;

        if should_emit {
            self.last_emit_at = Some(now);
        }

        let snapshot = self.snapshot();
        let progress_log = if should_emit {
            self.next_progress_log(phase, progress_message, &snapshot)
        } else {
            None
        };

        (snapshot, should_emit, progress_log)
    }

    fn next_progress_log(
        &mut self,
        phase: &str,
        progress_message: &str,
        snapshot: &TransferSnapshot,
    ) -> Option<String> {
        let phase_changed = self.last_log_phase.as_deref() != Some(phase);
        let percent_bucket = snapshot
            .percent
            .map(|percent| (percent.clamp(0.0, 100.0) / 10.0).floor() as u8);
        let percent_changed =
            percent_bucket.is_some() && percent_bucket != self.last_log_percent_bucket;

        if !phase_changed && !percent_changed {
            return None;
        }

        self.last_log_phase = Some(phase.to_string());
        if let Some(bucket) = percent_bucket {
            self.last_log_percent_bucket = Some(bucket);
        }

        Some(match snapshot.percent {
            Some(percent) => format!("{progress_message}：{percent:.1}%"),
            None => progress_message.to_string(),
        })
    }

    pub(crate) fn snapshot(&self) -> TransferSnapshot {
        TransferSnapshot {
            percent: self.percent,
            downloaded_bytes: self.downloaded_bytes,
            total_bytes: self.total_bytes,
            bytes_per_second: self.bytes_per_second,
        }
    }
}
