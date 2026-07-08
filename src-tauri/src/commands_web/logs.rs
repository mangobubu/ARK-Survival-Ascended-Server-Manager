use super::support::{optional_arg, required_arg, to_json};
use crate::{
    app_state::AppRuntime,
    command_events::publish_sync_event_best_effort,
    commands::LogClearScope,
    models::{LogSource, ServerLogKind},
    sync_events::{LOGS_CLEARED_EVENT, LOGS_RESET_EVENT},
};
use serde_json::{Value, json};
use tauri::AppHandle;

pub(super) fn query_logs(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(runtime.query_logs(optional_arg(args, "limit")?)?)
}

pub(super) fn clear_logs(app: &AppHandle, runtime: &AppRuntime) -> Result<Value, String> {
    runtime.clear_logs()?;
    publish_sync_event_best_effort(app, LOGS_RESET_EVENT, json!({}));
    Ok(Value::Null)
}

pub(super) fn clear_scoped_logs(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let source: LogSource = required_arg(args, "source")?;
    let instance: Option<String> = optional_arg(args, "instance")?;
    let server_log_kind: Option<ServerLogKind> = optional_arg(args, "serverLogKind")?;
    runtime.clear_logs_by_scope(source.clone(), instance.as_deref(), server_log_kind.clone())?;
    publish_sync_event_best_effort(
        app,
        LOGS_CLEARED_EVENT,
        LogClearScope {
            source,
            instance,
            server_log_kind,
        },
    );
    Ok(Value::Null)
}
