use tauri::{Manager, RunEvent};

mod acme_certificate;
mod acme_client;
mod acme_crypto;
mod acme_dns;
mod acme_key_material;
mod acme_persistence;
mod app_logs;
mod app_persistence;
mod app_state;
mod ark_config;
mod ark_config_game_user_settings;
mod ark_config_ini;
mod ark_config_launch;
mod ark_config_mods;
mod ark_config_values;
mod asa_config_defaults;
mod asa_config_metadata;
mod asa_server_process;
mod backup;
mod command_events;
mod commands;
mod commands_web;
mod host_directory_browser;
mod import_export;
mod instance_config_commands;
mod instance_config_import;
mod instance_config_import_ini;
mod instance_config_import_mapping;
mod instance_config_import_paths;
mod instance_config_import_readers;
mod instance_data_commands;
mod instance_query_commands;
mod instance_runtime_config;
mod models;
mod path_security;
mod rcon;
mod rcon_players;
mod reverse_proxy;
mod reverse_proxy_admin;
mod reverse_proxy_config;
mod reverse_proxy_host;
mod reverse_proxy_ip_whitelist;
mod reverse_proxy_runtime;
mod reverse_proxy_security_gateway;
mod secret_storage;
mod server_lifecycle;
mod server_lifecycle_monitor;
mod server_lifecycle_stop;
mod server_log;
mod server_log_cleanup;
mod server_log_events;
mod server_log_reader;
mod server_player_events;
mod server_rcon;
mod server_version;
mod settings_commands;
mod steamcmd;
mod steamcmd_process;
mod steamcmd_progress;
mod steamcmd_update_monitor;
mod steamcmd_update_output;
mod steamcmd_update_runner;
mod storage;
mod sync_events;
mod tencent_dns;
mod web_auth;
mod web_auth_state;
mod web_auth_utils;
mod web_command_security;
mod web_http;
mod web_request_security;
mod web_server;
mod web_server_auth;
mod window_controls;
mod window_hotkey;
mod windows_firewall;

const MAIN_WINDOW_LABEL: &str = "main";

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let runtime = app_state::AppRuntime::load(app.handle())
                .map_err(Box::<dyn std::error::Error>::from)?;
            let initial_settings = runtime
                .settings()
                .map_err(Box::<dyn std::error::Error>::from)?;
            app.manage(sync_events::SyncEventBus::default());
            app.manage(reverse_proxy::ReverseProxyManager::default());
            app.manage(web_server::WebServerManager::default());
            app.manage(runtime.clone());
            window_controls::setup_window_controls(app, &runtime)
                .map_err(Box::<dyn std::error::Error>::from)?;
            let web_ready = if initial_settings.web_management_enabled {
                match web_server::apply_settings_from_app(app.handle(), &initial_settings) {
                    Ok(()) => true,
                    Err(error) => {
                        let _ = command_events::emit_instance_log(
                            app.handle(),
                            &runtime,
                            "Web服务",
                            "error",
                            &format!("应用启动时未能启动 Web 管理：{error}"),
                        );
                        false
                    }
                }
            } else {
                false
            };
            if web_ready
                && let Err(error) =
                    reverse_proxy::apply_settings_from_app(app.handle(), &initial_settings)
                && let Some(runtime) = app.handle().try_state::<app_state::AppRuntime>()
            {
                let _ = command_events::emit_instance_log(
                    app.handle(),
                    runtime.inner(),
                    "Web反代",
                    "warn",
                    &format!("应用启动时未能启用域名反向代理：{error}"),
                );
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            asa_config_metadata::get_asa_config_metadata,
            storage::ensure_storage_directories,
            steamcmd::check_steamcmd,
            steamcmd::install_steamcmd,
            settings_commands::get_settings,
            settings_commands::save_settings,
            settings_commands::list_web_security_bans,
            settings_commands::get_web_acme_certificate_status,
            settings_commands::unban_web_security_ip,
            instance_query_commands::list_instances,
            instance_query_commands::clear_startup_auto_update_skip_flags,
            instance_query_commands::check_instance_port,
            commands::create_instance,
            instance_data_commands::read_server_directory_config,
            instance_data_commands::list_host_directories,
            instance_config_commands::get_instance_config,
            instance_config_commands::get_instance_mods,
            instance_config_commands::save_instance_config,
            commands::apply_instance_config,
            instance_config_commands::update_instance_mods,
            instance_config_commands::check_mod_updates,
            commands::install_or_update_instance,
            commands::start_instance,
            commands::stop_instance,
            commands::restart_instance,
            commands::refresh_instance_status,
            commands::execute_rcon_command,
            commands::query_logs,
            commands::clear_logs,
            commands::clear_scoped_logs,
            instance_data_commands::create_backup,
            instance_data_commands::list_backups,
            instance_data_commands::restore_backup,
            instance_data_commands::export_instance_config,
            instance_data_commands::export_cluster,
            instance_data_commands::import_instance_config,
            commands::delete_instance,
            commands::open_instance_directory,
            commands::open_directory_path,
        ])
        .build(tauri::generate_context!())
        .expect("启动 ASA 服务端管理器失败")
        .run(move |app_handle, event| {
            if let RunEvent::WindowEvent { label, event, .. } = event
                && label == MAIN_WINDOW_LABEL
            {
                if matches!(event, tauri::WindowEvent::Destroyed) {
                    web_server::shutdown(app_handle);
                    reverse_proxy::shutdown(app_handle);
                }
                let Some(state) =
                    app_handle.try_state::<std::sync::Arc<window_controls::WindowControlState>>()
                else {
                    return;
                };
                if matches!(event, tauri::WindowEvent::Destroyed) {
                    window_controls::request_full_shutdown(app_handle, &state);
                    return;
                }
                let Some(window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) else {
                    return;
                };
                let Some(runtime) = app_handle.try_state::<app_state::AppRuntime>() else {
                    return;
                };
                window_controls::handle_main_window_event(&window, &event, &state, &runtime);
            }
        });
}
