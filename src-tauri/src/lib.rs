use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::{Manager, RunEvent, WindowEvent};

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
mod web_server;

const MAIN_WINDOW_LABEL: &str = "main";

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn request_full_shutdown<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    is_shutting_down: &AtomicBool,
) {
    if is_shutting_down.swap(true, Ordering::SeqCst) {
        return;
    }

    for (label, window) in app_handle.webview_windows() {
        if label != MAIN_WINDOW_LABEL {
            let _ = window.close();
        }
    }

    app_handle.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let is_shutting_down = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let runtime = app_state::AppRuntime::load(app.handle())
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?;
            let web_server_port = runtime
                .settings()
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?
                .web_server_port;
            app.manage(runtime.clone());
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
                if label == MAIN_WINDOW_LABEL
                    && matches!(
                        event,
                        WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed
                    )
                {
                    request_full_shutdown(app_handle, is_shutting_down.as_ref());
                }
            }
        });
}
