use super::{
    SERVER_LOG_POLL_INTERVAL, ServerLogLineContext, append_log_bytes, is_instance_process_tracked,
};
use crate::{
    app_state::AppRuntime,
    ark_config,
    models::{ServerInstance, ServerLogKind},
    server_log::SharedServerLogDeduper,
    server_version::is_server_log_candidate,
};
use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    time::{MissedTickBehavior, interval},
};

const SERVER_LOG_INITIAL_BACKFILL_LIMIT: u64 = 512 * 1024;
const FILE_LOG_READ_BUFFER_BYTES: usize = 8192;

pub(crate) fn attach_server_log_file_reader(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance: &ServerInstance,
    deduper: SharedServerLogDeduper,
) {
    let app = app.clone();
    let runtime = runtime.clone();
    let instance_id = instance.id.clone();
    let instance_name = instance.name.clone();
    let log_dir = ark_config::saved_dir(instance).join("Logs");
    let watch_started_at = SystemTime::now();

    tokio::spawn(async move {
        tail_server_log_directory(
            app,
            runtime,
            instance_id,
            instance_name,
            log_dir,
            deduper,
            watch_started_at,
        )
        .await;
    });
}

async fn tail_server_log_directory(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
    instance_name: String,
    log_dir: PathBuf,
    deduper: SharedServerLogDeduper,
    watch_started_at: SystemTime,
) {
    let mut active_path: Option<PathBuf> = None;
    let mut offset = 0_u64;
    let mut pending = Vec::new();
    let mut ticker = interval(SERVER_LOG_POLL_INTERVAL);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;
        if !is_instance_process_tracked(&runtime, &instance_id) {
            break;
        }

        let Some(candidate) = latest_server_log_path(&log_dir).await else {
            continue;
        };
        let Ok(metadata) = tokio::fs::metadata(&candidate).await else {
            continue;
        };

        if active_path.as_ref() != Some(&candidate) {
            let is_rotation = active_path.is_some();
            offset = initial_server_log_offset(&metadata, watch_started_at, is_rotation);
            active_path = Some(candidate.clone());
            pending.clear();
        }

        let len = metadata.len();
        if len < offset {
            offset = 0;
            pending.clear();
        }
        if len <= offset {
            continue;
        }

        let context = ServerLogLineContext {
            app: &app,
            runtime: &runtime,
            instance_id: &instance_id,
            instance_name: &instance_name,
            deduper: &deduper,
        };
        match read_new_server_log_bytes(&candidate, offset, &mut pending, &context).await {
            Ok(new_offset) => offset = new_offset,
            Err(_) => {
                active_path = None;
                offset = 0;
                pending.clear();
            }
        }
    }
}

async fn latest_server_log_path(log_dir: &Path) -> Option<PathBuf> {
    let mut entries = tokio::fs::read_dir(log_dir).await.ok()?;
    let mut latest: Option<(SystemTime, PathBuf)> = None;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !is_server_log_candidate(&path) {
            continue;
        }
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        if latest
            .as_ref()
            .is_none_or(|(latest_modified, _)| modified > *latest_modified)
        {
            latest = Some((modified, path));
        }
    }

    latest.map(|(_, path)| path)
}

fn initial_server_log_offset(
    metadata: &std::fs::Metadata,
    watch_started_at: SystemTime,
    is_rotation: bool,
) -> u64 {
    if is_rotation {
        return 0;
    }
    if metadata.len() <= SERVER_LOG_INITIAL_BACKFILL_LIMIT {
        let threshold = watch_started_at
            .checked_sub(Duration::from_secs(2))
            .unwrap_or(watch_started_at);
        if metadata
            .modified()
            .map(|modified| modified >= threshold)
            .unwrap_or(false)
        {
            return 0;
        }
    }
    metadata.len()
}

async fn read_new_server_log_bytes(
    path: &Path,
    offset: u64,
    pending: &mut Vec<u8>,
    context: &ServerLogLineContext<'_>,
) -> Result<u64, String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|error| format!("打开服务端日志失败：{error}"))?;
    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(|error| format!("定位服务端日志失败：{error}"))?;

    let mut current_offset = offset;
    let mut buffer = [0_u8; FILE_LOG_READ_BUFFER_BYTES];
    loop {
        let read = file
            .read(&mut buffer)
            .await
            .map_err(|error| format!("读取服务端日志失败：{error}"))?;
        if read == 0 {
            break;
        }
        current_offset = current_offset.saturating_add(read as u64);
        append_log_bytes(
            context,
            pending,
            &buffer[..read],
            "info",
            ServerLogKind::File,
        );
    }
    Ok(current_offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn 初次监听近期小日志会从头回放() {
        let mut file = tempfile::NamedTempFile::new().expect("创建临时日志");
        file.write_all(b"line").expect("写入临时日志");
        let metadata = file.as_file().metadata().expect("读取元数据");

        assert_eq!(
            initial_server_log_offset(&metadata, SystemTime::now(), false),
            0
        );
    }

    #[test]
    fn 初次监听旧日志会跳过已有内容() {
        let mut file = tempfile::NamedTempFile::new().expect("创建临时日志");
        file.write_all(b"line").expect("写入临时日志");
        let metadata = file.as_file().metadata().expect("读取元数据");
        let future_watch_time = SystemTime::now() + Duration::from_secs(10);

        assert_eq!(
            initial_server_log_offset(&metadata, future_watch_time, false),
            metadata.len()
        );
    }

    #[test]
    fn 轮转日志总是从头读取() {
        let mut file = tempfile::NamedTempFile::new().expect("创建临时日志");
        file.write_all(b"line").expect("写入临时日志");
        let metadata = file.as_file().metadata().expect("读取元数据");
        let future_watch_time = SystemTime::now() + Duration::from_secs(10);

        assert_eq!(
            initial_server_log_offset(&metadata, future_watch_time, true),
            0
        );
    }
}
