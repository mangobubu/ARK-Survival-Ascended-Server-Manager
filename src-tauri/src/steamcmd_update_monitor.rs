use crate::{
    models::ASA_DEDICATED_SERVER_APP_ID,
    server_version::{parse_manifest_progress, server_appmanifest_path},
    steamcmd_process::{ProcessTransferCounters, process_transfer_counters},
    steamcmd_progress::{ParsedSteamCmdProgress, percent_from_bytes},
    steamcmd_update_output::{SteamCmdProgressSink, emit_steamcmd_transfer_progress},
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};

pub(crate) fn spawn_manifest_progress_monitor(
    install_path: &Path,
    process_id: Option<u32>,
    progress: SteamCmdProgressSink,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let manifest_path = server_appmanifest_path(install_path);
    let downloading_path = install_path
        .join("steamapps")
        .join("downloading")
        .join(ASA_DEDICATED_SERVER_APP_ID);

    tokio::spawn(async move {
        let mut io_baseline: Option<ProcessTransferCounters> = None;
        let mut downloading_size_baseline: Option<u64> = None;
        let mut manifest_baseline = 0_u64;
        let mut ticker = interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        ticker.tick().await;

        while !stop.load(Ordering::SeqCst) {
            if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await
                && let Some(mut manifest) = parse_manifest_progress(&content)
            {
                if manifest.phase == "downloading" {
                    let manifest_changed = manifest.downloaded_bytes != manifest_baseline;
                    let mut estimated_delta = 0_u64;

                    if let Some(current_transfer) = process_transfer_counters(process_id) {
                        if io_baseline.is_none() || manifest_changed {
                            io_baseline = Some(current_transfer);
                        }

                        if let Some(base_transfer) = io_baseline {
                            estimated_delta = estimated_delta.max(
                                current_transfer.estimated_download_delta_since(base_transfer),
                            );
                        }
                    }

                    if let Some(current_size) = directory_size_bytes(&downloading_path) {
                        if downloading_size_baseline.is_none() || manifest_changed {
                            downloading_size_baseline = Some(current_size);
                        }

                        if let Some(base_size) = downloading_size_baseline {
                            estimated_delta =
                                estimated_delta.max(current_size.saturating_sub(base_size));
                        }
                    }

                    manifest_baseline = manifest.downloaded_bytes;

                    if let Some(total) = manifest.total_bytes {
                        manifest.downloaded_bytes = manifest
                            .downloaded_bytes
                            .saturating_add(estimated_delta)
                            .min(total);
                    }
                }

                let percent = manifest
                    .total_bytes
                    .map(|total| percent_from_bytes(manifest.downloaded_bytes, total));
                emit_steamcmd_transfer_progress(
                    &progress,
                    ParsedSteamCmdProgress {
                        phase: manifest.phase,
                        message: "SteamCMD 正在下载/更新服务端文件".to_string(),
                        percent,
                        downloaded_bytes: manifest.downloaded_bytes,
                        total_bytes: manifest.total_bytes,
                    },
                    Some(format!("manifest: {}", manifest_path.display())),
                    true,
                );
            }
            ticker.tick().await;
        }
    })
}

fn directory_size_bytes(path: &Path) -> Option<u64> {
    if !path.is_dir() {
        return None;
    }

    let mut total = 0_u64;
    let mut pending = vec![path.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(current) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                total = total.saturating_add(metadata.len());
            }
        }
    }

    Some(total)
}
