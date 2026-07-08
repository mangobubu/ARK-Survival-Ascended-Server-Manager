use crate::{
    app_state::{AppRuntime, now_millis},
    command_events::{emit_instance_log, publish_sync_event_best_effort},
    models::{JobProgress, ServerInstance},
    steamcmd_progress::{
        ParsedSteamCmdProgress, SteamCmdProgressTracker, TransferSnapshot,
        parse_steamcmd_progress_line, remember_tail,
    },
    sync_events::JOB_PROGRESS_EVENT,
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{AppHandle, ipc::Channel};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    task::JoinHandle,
    time::timeout,
};

#[derive(Clone)]
pub(crate) struct SteamCmdProgressSink {
    app: AppHandle,
    runtime: AppRuntime,
    channel: Channel<JobProgress>,
    instance_id: String,
    instance_name: String,
    tracker: Arc<Mutex<SteamCmdProgressTracker>>,
}

impl SteamCmdProgressSink {
    pub(crate) fn new(
        app: &AppHandle,
        runtime: &AppRuntime,
        channel: &Channel<JobProgress>,
        instance: &ServerInstance,
    ) -> Self {
        Self {
            app: app.clone(),
            runtime: runtime.clone(),
            channel: channel.clone(),
            instance_id: instance.id.clone(),
            instance_name: instance.name.clone(),
            tracker: Arc::new(Mutex::new(SteamCmdProgressTracker::default())),
        }
    }
}

fn handle_steamcmd_progress_line(sink: &SteamCmdProgressSink, line: &str) -> bool {
    let Some(parsed) = parse_steamcmd_progress_line(line) else {
        return false;
    };
    emit_steamcmd_transfer_progress(sink, parsed, Some(line.to_string()), false);
    true
}

pub(crate) fn emit_steamcmd_transfer_progress(
    sink: &SteamCmdProgressSink,
    parsed: ParsedSteamCmdProgress,
    detail: Option<String>,
    force_emit: bool,
) {
    let now = Instant::now();
    let (snapshot, should_emit, progress_log) = match sink.tracker.lock() {
        Ok(mut tracker) => tracker.update(
            parsed.downloaded_bytes,
            parsed.total_bytes,
            parsed.percent,
            &parsed.phase,
            &parsed.message,
            now,
            force_emit,
        ),
        Err(_) => return,
    };

    if let Some(message) = progress_log {
        let _ = emit_instance_log(
            &sink.app,
            &sink.runtime,
            &sink.instance_name,
            "info",
            &message,
        );
    }

    if should_emit {
        let payload = JobProgress {
            job_id: format!("job-{}", now_millis()),
            instance_id: Some(sink.instance_id.clone()),
            phase: parsed.phase,
            percent: snapshot.percent,
            message: parsed.message,
            detail,
            downloaded_bytes: snapshot.downloaded_bytes,
            total_bytes: snapshot.total_bytes,
            bytes_per_second: snapshot.bytes_per_second,
        };
        let _ = sink.channel.send(payload.clone());
        publish_sync_event_best_effort(&sink.app, JOB_PROGRESS_EVENT, payload);
    }
}

pub(crate) fn transfer_snapshot(sink: &SteamCmdProgressSink) -> TransferSnapshot {
    sink.tracker
        .lock()
        .map(|tracker| tracker.snapshot())
        .unwrap_or_default()
}

fn handle_command_output_line(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    level: &'static str,
    tail: &Arc<Mutex<VecDeque<String>>>,
    progress: Option<&SteamCmdProgressSink>,
    line: &str,
) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    remember_tail(tail, line);

    if progress
        .map(|sink| handle_steamcmd_progress_line(sink, line))
        .unwrap_or(false)
    {
        return;
    }

    let _ = emit_instance_log(app, runtime, instance_name, level, line);
}

pub(crate) fn spawn_command_log_reader<R>(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    stream: Option<R>,
    level: &'static str,
    tail: Arc<Mutex<VecDeque<String>>>,
    progress: Option<SteamCmdProgressSink>,
) -> Option<JoinHandle<()>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let stream = stream?;
    let app = app.clone();
    let runtime = runtime.clone();
    let instance_name = instance_name.to_string();
    Some(tokio::spawn(async move {
        let mut stream = stream;
        let mut buffer = [0_u8; 4096];
        let mut pending = Vec::new();

        loop {
            let read = match stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };
            for byte in &buffer[..read] {
                if *byte == b'\r' || *byte == b'\n' {
                    let line = String::from_utf8_lossy(&pending);
                    handle_command_output_line(
                        &app,
                        &runtime,
                        &instance_name,
                        level,
                        &tail,
                        progress.as_ref(),
                        &line,
                    );
                    pending.clear();
                } else {
                    pending.push(*byte);
                    if pending.len() > 8192 {
                        let line = String::from_utf8_lossy(&pending);
                        handle_command_output_line(
                            &app,
                            &runtime,
                            &instance_name,
                            level,
                            &tail,
                            progress.as_ref(),
                            &line,
                        );
                        pending.clear();
                    }
                }
            }
        }

        if !pending.is_empty() {
            let line = String::from_utf8_lossy(&pending);
            handle_command_output_line(
                &app,
                &runtime,
                &instance_name,
                level,
                &tail,
                progress.as_ref(),
                &line,
            );
        }
    }))
}

pub(crate) async fn wait_for_log_readers(readers: Vec<Option<JoinHandle<()>>>) {
    for reader in readers.into_iter().flatten() {
        let _ = timeout(Duration::from_secs(2), reader).await;
    }
}
