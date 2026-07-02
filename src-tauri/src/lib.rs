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
        .invoke_handler(tauri::generate_handler![
            app_version,
            storage::ensure_storage_directories,
            steamcmd::check_steamcmd,
            steamcmd::install_steamcmd,
        ])
        .run(tauri::generate_context!())
        .expect("启动 ASA 服务器管理器失败");
}
