use std::process::ExitStatus;

use crate::{
    app_state::AppRuntime,
    models::{ServerInstance, ServerStatus},
};

use super::config::bool_from_config;

pub(crate) fn should_auto_restart_after_exit(
    runtime: &AppRuntime,
    instance_id: &str,
    exit_status: &ExitStatus,
) -> bool {
    if exit_status.success() {
        return false;
    }

    runtime
        .get_instance(instance_id)
        .map(|instance| {
            instance.status == ServerStatus::Running
                && should_auto_restart_after_crash(runtime, &instance)
        })
        .unwrap_or(false)
}

fn should_auto_restart_after_crash(runtime: &AppRuntime, instance: &ServerInstance) -> bool {
    let Ok(settings) = runtime.settings() else {
        return false;
    };
    if !settings.auto_restart_on_crash {
        return false;
    }

    runtime
        .get_config(&instance.id)
        .map(|config| bool_from_config(&config, "restartOnCrash", true))
        .unwrap_or(false)
}
