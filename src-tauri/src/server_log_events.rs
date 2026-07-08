use crate::{
    app_state::AppRuntime,
    command_events::publish_sync_event,
    models::{LogSource, ServerLogKind},
    sync_events::{LOG_LINE_EVENT, LOGS_CLEARED_EVENT},
};
use serde::Serialize;
use tauri::AppHandle;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogClearScope {
    source: LogSource,
    instance: Option<String>,
    server_log_kind: Option<ServerLogKind>,
}

pub(crate) fn emit_server_log(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    level: &str,
    message: &str,
    server_log_kind: ServerLogKind,
) -> Result<(), String> {
    let line = runtime.add_server_log_with_kind(instance_name, level, message, server_log_kind)?;
    publish_sync_event(app, LOG_LINE_EVENT, line)
        .map_err(|error| format!("发送日志事件失败：{error}"))
}

pub(crate) fn emit_logs_cleared(
    app: &AppHandle,
    source: LogSource,
    instance: Option<&str>,
    server_log_kind: Option<ServerLogKind>,
) -> Result<(), String> {
    publish_sync_event(
        app,
        LOGS_CLEARED_EVENT,
        LogClearScope {
            source,
            instance: instance.map(str::to_string),
            server_log_kind,
        },
    )
    .map_err(|error| format!("发送日志清理事件失败：{error}"))
}
