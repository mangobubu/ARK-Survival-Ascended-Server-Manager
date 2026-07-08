use super::support::{no_op_channel, required_arg, to_json};
use crate::{
    app_state::AppRuntime,
    command_events::{publish_instances_changed, publish_sync_event_best_effort},
    commands::install_or_update_instance_inner,
    instance_query_commands,
    models::{AddInstancePayload, JobProgress},
    server_lifecycle::{
        refresh_status_for_runtime, restart_instance_for_runtime, start_instance_for_runtime,
        stop_instance_for_runtime,
    },
    server_rcon,
    server_version::with_current_server_version,
    sync_events::{ADD_INSTANCE_CREATED_EVENT, INSTANCE_DELETED_EVENT},
};
use serde_json::{Value, json};
use tauri::AppHandle;

pub(super) fn list_instances(runtime: &AppRuntime) -> Result<Value, String> {
    to_json(instance_query_commands::list_instances_for_runtime(
        runtime,
    )?)
}

pub(super) fn clear_startup_auto_update_skip_flags(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(
        instance_query_commands::clear_startup_auto_update_skip_flags_for_runtime(
            app,
            runtime,
            required_arg(args, "instanceIds")?,
        )?,
    )
}

pub(super) fn check_instance_port(runtime: &AppRuntime, args: &Value) -> Result<Value, String> {
    to_json(instance_query_commands::check_instance_port_for_runtime(
        runtime,
        required_arg(args, "port")?,
        &required_arg::<String>(args, "portKind")?,
    )?)
}

pub(super) fn create_instance(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let payload: AddInstancePayload = required_arg(args, "payload")?;
    let auto_install = payload.auto_install;
    let instance = with_current_server_version(runtime.create_instance(payload)?);
    publish_sync_event_best_effort(
        app,
        ADD_INSTANCE_CREATED_EVENT,
        json!({
            "instance": instance,
            "autoInstall": auto_install,
        }),
    );
    publish_instances_changed(app);
    to_json(instance)
}

pub(super) async fn install_or_update_instance(
    app: AppHandle,
    runtime: AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(
        install_or_update_instance_inner(
            app,
            runtime,
            required_arg(args, "instanceId")?,
            no_op_channel::<JobProgress>(),
        )
        .await?,
    )
}

pub(super) async fn start_instance(
    app: AppHandle,
    runtime: AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(start_instance_for_runtime(app, runtime, required_arg(args, "instanceId")?).await?)
}

pub(super) fn stop_instance(
    app: AppHandle,
    runtime: AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(stop_instance_for_runtime(
        app,
        runtime,
        required_arg(args, "instanceId")?,
    )?)
}

pub(super) async fn restart_instance(
    app: AppHandle,
    runtime: AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(restart_instance_for_runtime(app, runtime, required_arg(args, "instanceId")?).await?)
}

pub(super) async fn refresh_instance_status(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let instance_id: String = required_arg(args, "instanceId")?;
    to_json(refresh_status_for_runtime(app, runtime, &instance_id).await?)
}

pub(super) async fn execute_rcon_command(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    to_json(
        server_rcon::execute_rcon_command(
            app,
            runtime,
            required_arg(args, "instanceId")?,
            required_arg(args, "command")?,
        )
        .await?,
    )
}

pub(super) fn delete_instance(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let removed = runtime.delete_instance(&required_arg::<String>(args, "instanceId")?)?;
    publish_sync_event_best_effort(app, INSTANCE_DELETED_EVENT, removed.clone());
    publish_instances_changed(app);
    to_json(removed)
}
