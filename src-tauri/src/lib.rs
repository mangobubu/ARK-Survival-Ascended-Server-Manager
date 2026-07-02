use tauri::Manager;

mod app_state;
mod ark_config;
mod backup;
mod commands;
mod import_export;
mod models;
mod rcon;
mod steamcmd;
mod storage;

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
            app.manage(runtime);
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
            commands::create_instance,
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
            commands::query_logs,
            commands::clear_logs,
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::export_instance_config,
            commands::export_cluster,
            commands::import_instance_config,
            commands::open_instance_directory,
        ])
        .run(tauri::generate_context!())
        .expect("启动 ASA 服务器管理器失败");
}
