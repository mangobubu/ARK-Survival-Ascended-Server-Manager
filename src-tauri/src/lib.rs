use tauri::{Manager, RunEvent};

mod app_state;
mod ark_config;
mod backup;
mod commands;
mod import_export;
mod instance_config_import;
mod models;
mod rcon;
mod steamcmd;
mod storage;
mod sync_events;
mod web_server;
mod window_controls;

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
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?;
            let web_server_port = runtime
                .settings()
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?
                .web_server_port;
            app.manage(sync_events::SyncEventBus::default());
            app.manage(runtime.clone());
            window_controls::setup_window_controls(app, &runtime)
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?;
            web_server::start(app.handle().clone(), runtime, web_server_port);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            storage::ensure_storage_directories,
            steamcmd::check_steamcmd,
            steamcmd::install_steamcmd,
            commands::get_settings,
            commands::save_settings,
            commands::list_instances,
            commands::check_instance_port,
            commands::create_instance,
            commands::read_server_directory_config,
            commands::get_instance_config,
            commands::get_instance_mods,
            commands::save_instance_config,
            commands::apply_instance_config,
            commands::update_instance_mods,
            commands::check_mod_updates,
            commands::install_or_update_instance,
            commands::start_instance,
            commands::stop_instance,
            commands::restart_instance,
            commands::refresh_instance_status,
            commands::execute_rcon_command,
            commands::query_logs,
            commands::clear_logs,
            commands::clear_scoped_logs,
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::export_instance_config,
            commands::export_cluster,
            commands::import_instance_config,
            commands::delete_instance,
            commands::open_instance_directory,
            commands::open_directory_path,
        ])
        .build(tauri::generate_context!())
        .expect("启动 ASA 服务端管理器失败")
        .run(move |app_handle, event| {
            if let RunEvent::WindowEvent { label, event, .. } = event {
                if label == MAIN_WINDOW_LABEL {
                    let Some(state) = app_handle
                        .try_state::<std::sync::Arc<window_controls::WindowControlState>>()
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
            }
        });
}
