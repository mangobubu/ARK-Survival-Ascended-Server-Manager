use super::support::{no_op_channel, required_arg, to_json};
use crate::{
    app_state::AppRuntime, asa_config_metadata, models::GlobalSettings, settings_commands,
    steamcmd, storage,
};
use serde_json::Value;
use tauri::AppHandle;

pub(super) fn get_asa_config_metadata() -> Result<Value, String> {
    to_json(asa_config_metadata::config_metadata())
}

pub(super) fn ensure_storage_directories(args: &Value) -> Result<Value, String> {
    storage::ensure_storage_directories(
        required_arg(args, "serverStoragePath")?,
        required_arg(args, "backupStoragePath")?,
    )?;
    Ok(Value::Null)
}

pub(super) fn check_steamcmd(args: &Value) -> Result<Value, String> {
    to_json(steamcmd::check_steamcmd(required_arg(args, "path")?)?)
}

pub(super) async fn install_steamcmd(args: &Value) -> Result<Value, String> {
    to_json(
        steamcmd::install_steamcmd(
            required_arg(args, "parentPath")?,
            no_op_channel::<steamcmd::SteamCmdProgress>(),
        )
        .await?,
    )
}

pub(super) fn get_settings(runtime: &AppRuntime) -> Result<Value, String> {
    settings_commands::public_settings(runtime.settings()?)
}

pub(super) fn save_settings(
    app: &AppHandle,
    runtime: &AppRuntime,
    args: &Value,
) -> Result<Value, String> {
    let settings: GlobalSettings = required_arg(args, "settings")?;
    settings_commands::save_settings_for_runtime(app, runtime, settings)
}

pub(super) fn list_web_security_bans(app: &AppHandle) -> Result<Value, String> {
    settings_commands::list_web_security_bans_for_app(app)
}

pub(super) fn get_web_acme_certificate_status(runtime: &AppRuntime) -> Result<Value, String> {
    settings_commands::get_web_acme_certificate_status_for_runtime(runtime)
}

pub(super) fn unban_web_security_ip(app: &AppHandle, args: &Value) -> Result<Value, String> {
    settings_commands::unban_web_security_ip_for_app(app, &required_arg::<String>(args, "ip")?)
}
