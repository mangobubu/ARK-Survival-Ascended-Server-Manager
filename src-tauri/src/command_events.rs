use crate::{
    app_state::{AppRuntime, now_millis},
    models::JobProgress,
    server_version::with_current_server_version,
    steamcmd_progress::TransferSnapshot,
    sync_events::{
        INSTANCE_STATUS_EVENT, INSTANCES_CHANGED_EVENT, JOB_PROGRESS_EVENT, LOG_LINE_EVENT,
    },
    window_controls,
};
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, ipc::Channel};

pub(crate) fn publish_sync_event<T: Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: T,
) -> Result<(), String> {
    window_controls::publish_settings_changed_and_apply(app, event_name, payload)
}

pub(crate) fn publish_sync_event_best_effort<T: Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: T,
) {
    let _ = publish_sync_event(app, event_name, payload);
}

pub(crate) fn publish_instances_changed(app: &AppHandle) {
    publish_sync_event_best_effort(app, INSTANCES_CHANGED_EVENT, json!({}));
}

pub(crate) fn emit_status(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<(), String> {
    let instance = with_current_server_version(runtime.get_instance(instance_id)?);
    publish_sync_event(app, INSTANCE_STATUS_EVENT, instance)
        .map_err(|error| format!("发送实例状态事件失败：{error}"))
}

pub(crate) fn emit_instance_log(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    level: &str,
    message: &str,
) -> Result<(), String> {
    let line = runtime.add_log(instance_name, level, message)?;
    publish_sync_event(app, LOG_LINE_EVENT, line)
        .map_err(|error| format!("发送日志事件失败：{error}"))
}

pub(crate) fn emit_progress(
    app: &AppHandle,
    channel: &Channel<JobProgress>,
    instance_id: &str,
    phase: &str,
    percent: Option<f64>,
    message: &str,
    detail: Option<String>,
) -> Result<(), String> {
    emit_progress_with_transfer(
        app,
        channel,
        instance_id,
        phase,
        percent,
        message,
        detail,
        TransferSnapshot {
            percent,
            downloaded_bytes: 0,
            total_bytes: None,
            bytes_per_second: 0,
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_progress_with_transfer(
    app: &AppHandle,
    channel: &Channel<JobProgress>,
    instance_id: &str,
    phase: &str,
    percent: Option<f64>,
    message: &str,
    detail: Option<String>,
    transfer: TransferSnapshot,
) -> Result<(), String> {
    let payload = JobProgress {
        job_id: format!("job-{}", now_millis()),
        instance_id: Some(instance_id.to_string()),
        phase: phase.to_string(),
        percent,
        message: message.to_string(),
        detail,
        downloaded_bytes: transfer.downloaded_bytes,
        total_bytes: transfer.total_bytes,
        bytes_per_second: transfer.bytes_per_second,
    };
    channel
        .send(payload.clone())
        .map_err(|error| format!("发送任务进度失败：{error}"))
        .map(|_| {
            publish_sync_event_best_effort(app, JOB_PROGRESS_EVENT, payload);
        })
}
