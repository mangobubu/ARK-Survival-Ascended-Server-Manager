mod file_tail;

pub(crate) use file_tail::attach_server_log_file_reader;

use crate::{
    app_state::AppRuntime,
    command_events::{emit_instance_log, publish_sync_event_best_effort},
    models::{ServerInstance, ServerLogKind},
    server_log::{
        SharedServerLogDeduper, classify_server_log_level, should_skip_duplicate_server_log,
    },
    server_log_events::emit_server_log,
    server_player_events::parse_player_server_event,
    server_version::{parse_asa_server_version, with_current_server_version},
    sync_events::INSTANCE_STATUS_EVENT,
};
use std::time::Duration;
use tauri::AppHandle;
use tokio::io::{AsyncRead, AsyncReadExt};

pub(crate) const SERVER_LOG_POLL_INTERVAL: Duration = Duration::from_millis(500);
const SERVER_LOG_MAX_PENDING_LINE_BYTES: usize = 8192;
const PROCESS_LOG_READ_BUFFER_BYTES: usize = 4096;

struct ServerLogLineContext<'a> {
    app: &'a AppHandle,
    runtime: &'a AppRuntime,
    instance_id: &'a str,
    instance_name: &'a str,
    deduper: &'a SharedServerLogDeduper,
}

pub(crate) fn attach_process_log_reader<R>(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance: &ServerInstance,
    stream: Option<R>,
    level: &'static str,
    deduper: SharedServerLogDeduper,
) where
    R: AsyncRead + Unpin + Send + 'static,
{
    let Some(stream) = stream else {
        return;
    };
    let app = app.clone();
    let runtime = runtime.clone();
    let instance_id = instance.id.clone();
    let instance_name = instance.name.clone();
    tokio::spawn(async move {
        let mut stream = stream;
        let mut buffer = [0_u8; PROCESS_LOG_READ_BUFFER_BYTES];
        let mut pending = Vec::new();

        loop {
            let read = match stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };

            let context = ServerLogLineContext {
                app: &app,
                runtime: &runtime,
                instance_id: &instance_id,
                instance_name: &instance_name,
                deduper: &deduper,
            };
            append_log_bytes(
                &context,
                &mut pending,
                &buffer[..read],
                level,
                ServerLogKind::Console,
            );
        }

        if !pending.is_empty() {
            let context = ServerLogLineContext {
                app: &app,
                runtime: &runtime,
                instance_id: &instance_id,
                instance_name: &instance_name,
                deduper: &deduper,
            };
            flush_pending_line(&context, &mut pending, level, ServerLogKind::Console);
        }
    });
}

fn append_log_bytes(
    context: &ServerLogLineContext<'_>,
    pending: &mut Vec<u8>,
    bytes: &[u8],
    fallback_level: &'static str,
    server_log_kind: ServerLogKind,
) {
    for byte in bytes {
        if *byte == b'\r' || *byte == b'\n' {
            flush_pending_line(context, pending, fallback_level, server_log_kind.clone());
        } else {
            pending.push(*byte);
            if pending.len() > SERVER_LOG_MAX_PENDING_LINE_BYTES {
                flush_pending_line(context, pending, fallback_level, server_log_kind.clone());
            }
        }
    }
}

fn flush_pending_line(
    context: &ServerLogLineContext<'_>,
    pending: &mut Vec<u8>,
    fallback_level: &'static str,
    server_log_kind: ServerLogKind,
) {
    let line = String::from_utf8_lossy(pending);
    handle_server_log_line(context, fallback_level, server_log_kind, &line);
    pending.clear();
}

fn handle_server_log_line(
    context: &ServerLogLineContext<'_>,
    fallback_level: &'static str,
    server_log_kind: ServerLogKind,
    line: &str,
) {
    let line = line.trim();
    if line.is_empty() || should_skip_duplicate_server_log(context.deduper, line) {
        return;
    }
    if let Some(server_version) = parse_asa_server_version(line)
        && let Ok(updated) = context
            .runtime
            .update_instance_server_version(context.instance_id, server_version)
    {
        publish_sync_event_best_effort(
            context.app,
            INSTANCE_STATUS_EVENT,
            with_current_server_version(updated),
        );
    }
    let level = classify_server_log_level(line, fallback_level);
    let _ = emit_server_log(
        context.app,
        context.runtime,
        context.instance_name,
        level,
        line,
        server_log_kind,
    );
    if let Some(player_event) = parse_player_server_event(line) {
        let _ = emit_instance_log(
            context.app,
            context.runtime,
            context.instance_name,
            player_event.level(),
            &player_event.application_message(),
        );
    }
}

fn is_instance_process_tracked(runtime: &AppRuntime, instance_id: &str) -> bool {
    runtime
        .lock_processes()
        .map(|processes| processes.contains_key(instance_id))
        .unwrap_or(false)
}
