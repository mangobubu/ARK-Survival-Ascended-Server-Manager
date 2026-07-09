mod config;
mod data;
mod instances;
mod logs;
mod open_paths;
mod settings;
mod support;

use crate::{app_state::AppRuntime, web_command_security};
use serde_json::Value;
use tauri::AppHandle;

pub(crate) async fn handle_web_invoke(
    app: AppHandle,
    runtime: AppRuntime,
    command: String,
    args: Value,
    high_risk_confirmed: bool,
) -> Result<Value, String> {
    web_command_security::web_command_policy(&command)?
        .validate_confirmed(&command, high_risk_confirmed)?;

    match command.as_str() {
        "app_version" => support::to_json(env!("CARGO_PKG_VERSION")),
        "get_asa_config_metadata" => settings::get_asa_config_metadata(),
        "ensure_storage_directories" => settings::ensure_storage_directories(&args),
        "check_steamcmd" => settings::check_steamcmd(&args),
        "install_steamcmd" => settings::install_steamcmd(&args).await,
        "get_settings" => settings::get_settings(&runtime),
        "save_settings" => settings::save_settings(&app, &runtime, &args),
        "list_web_security_bans" => settings::list_web_security_bans(&app),
        "get_web_acme_certificate_status" => settings::get_web_acme_certificate_status(&runtime),
        "unban_web_security_ip" => settings::unban_web_security_ip(&app, &args),
        "list_instances" => instances::list_instances(&runtime),
        "clear_startup_auto_update_skip_flags" => {
            instances::clear_startup_auto_update_skip_flags(&app, &runtime, &args)
        }
        "check_instance_port" => instances::check_instance_port(&runtime, &args),
        "create_instance" => instances::create_instance(&app, &runtime, &args),
        "read_server_directory_config" => config::read_server_directory_config(&runtime, &args),
        "list_host_directories" => config::list_host_directories(&runtime, &args),
        "get_instance_config" => config::get_instance_config(&runtime, &args),
        "get_instance_mods" => config::get_instance_mods(&runtime, &args),
        "save_instance_config" => config::save_instance_config(&app, &runtime, &args),
        "apply_instance_config" => config::apply_instance_config(app, runtime, &args).await,
        "update_instance_mods" => config::update_instance_mods(&app, &runtime, &args),
        "check_mod_updates" => config::check_mod_updates(&args),
        "install_or_update_instance" => {
            instances::install_or_update_instance(app, runtime, &args).await
        }
        "start_instance" => instances::start_instance(app, runtime, &args).await,
        "stop_instance" => instances::stop_instance(app, runtime, &args),
        "restart_instance" => instances::restart_instance(app, runtime, &args).await,
        "refresh_instance_status" => {
            instances::refresh_instance_status(&app, &runtime, &args).await
        }
        "execute_rcon_command" => instances::execute_rcon_command(&app, &runtime, &args).await,
        "query_logs" => logs::query_logs(&runtime, &args),
        "clear_logs" => logs::clear_logs(&app, &runtime),
        "clear_scoped_logs" => logs::clear_scoped_logs(&app, &runtime, &args),
        "create_backup" => data::create_backup(&app, &runtime, &args),
        "list_backups" => data::list_backups(&runtime, &args),
        "restore_backup" => data::restore_backup(&app, &runtime, &args),
        "export_instance_config" => data::export_instance_config(&runtime, &args),
        "export_cluster" => data::export_cluster(&runtime),
        "import_instance_config" => data::import_instance_config(&app, &runtime, &args),
        "delete_instance" => instances::delete_instance(&app, &runtime, &args),
        "open_instance_directory" => open_paths::open_instance_directory(&runtime, &args),
        "open_directory_path" => open_paths::open_directory_path(&runtime, &args),
        _ => Err(format!("未知 Web API 命令：{command}")),
    }
}
