use crate::{
    app_state::AppRuntime,
    ark_config,
    command_events::emit_instance_log,
    models::{LogSource, ServerInstance},
    server_log_events::emit_logs_cleared,
    server_log_reader::SERVER_LOG_POLL_INTERVAL,
    server_version::is_server_log_candidate,
};
use tauri::AppHandle;
use tokio::time::sleep;

pub(crate) async fn clear_instance_server_logs_before_start(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance: &ServerInstance,
) {
    sleep(SERVER_LOG_POLL_INTERVAL).await;

    let truncate_result = truncate_instance_game_log_files(instance);
    let runtime_result = clear_instance_runtime_server_logs(app, runtime, &instance.name);

    match (runtime_result, truncate_result) {
        (Ok(()), Ok(truncated_count)) => {
            let _ = emit_instance_log(
                app,
                runtime,
                &instance.name,
                "info",
                &format!("启动前已清空服务端窗口日志，并清空 {truncated_count} 个游戏日志文件"),
            );
        }
        (Ok(()), Err(error)) => {
            let _ = emit_instance_log(
                app,
                runtime,
                &instance.name,
                "warn",
                &format!("启动前已清空服务端窗口日志，但清空游戏日志文件失败：{error}"),
            );
        }
        (Err(error), Ok(truncated_count)) => {
            let _ = emit_instance_log(
                app,
                runtime,
                &instance.name,
                "warn",
                &format!(
                    "启动前已清空 {truncated_count} 个游戏日志文件，但清空服务端窗口日志失败：{error}"
                ),
            );
        }
        (Err(runtime_error), Err(file_error)) => {
            let _ = emit_instance_log(
                app,
                runtime,
                &instance.name,
                "warn",
                &format!(
                    "启动前清空服务端窗口日志失败：{runtime_error}；清空游戏日志文件失败：{file_error}"
                ),
            );
        }
    }
}

fn truncate_instance_game_log_files(instance: &ServerInstance) -> Result<usize, String> {
    let log_dir = ark_config::saved_dir(instance).join("Logs");
    if !log_dir.exists() {
        return Ok(0);
    }

    let entries = std::fs::read_dir(&log_dir)
        .map_err(|error| format!("无法读取游戏日志目录 {}：{error}", log_dir.display()))?;
    let mut truncated_count = 0_usize;
    let mut errors = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("读取游戏日志目录项失败：{error}"));
                continue;
            }
        };
        let path = entry.path();
        if !is_server_log_candidate(&path) {
            continue;
        }
        match entry.metadata() {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => continue,
            Err(error) => {
                errors.push(format!("读取 {} 元数据失败：{error}", path.display()));
                continue;
            }
        }
        match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
        {
            Ok(_) => truncated_count += 1,
            Err(error) => errors.push(format!("{}：{error}", path.display())),
        }
    }

    if errors.is_empty() {
        Ok(truncated_count)
    } else {
        Err(errors.join("；"))
    }
}

fn clear_instance_runtime_server_logs(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
) -> Result<(), String> {
    runtime.clear_logs_by_scope(LogSource::Server, Some(instance_name), None)?;
    emit_logs_cleared(app, LogSource::Server, Some(instance_name), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ServerStatus;

    #[test]
    fn 清空游戏日志文件只截断实例日志目录中的候选文件() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let log_dir = temp.path().join("ShooterGame").join("Saved").join("Logs");
        std::fs::create_dir_all(&log_dir).expect("创建日志目录");
        let log_path = log_dir.join("server.log");
        let txt_path = log_dir.join("game.txt");
        let keep_path = log_dir.join("readme.md");
        std::fs::write(&log_path, "旧服务端日志").expect("写入 log");
        std::fs::write(&txt_path, "旧游戏日志").expect("写入 txt");
        std::fs::write(&keep_path, "保留内容").expect("写入非日志文件");

        let instance = ServerInstance {
            id: "island".to_string(),
            name: "孤岛".to_string(),
            map: "The Island".to_string(),
            map_code: "TheIsland_WP".to_string(),
            mode: "PvE".to_string(),
            status: ServerStatus::Stopped,
            game_port: 7777,
            query_port: 27015,
            players: 0,
            max_players: 70,
            install_path: temp.path().to_string_lossy().into_owned(),
            rcon_port: 27020,
            cluster_id: "cluster".to_string(),
            description: String::new(),
            pid: None,
            last_started_at: None,
            last_stopped_at: None,
            server_version: String::new(),
            version_state: "已安装".to_string(),
            last_error: None,
            skip_auto_update_on_start_once: false,
        };

        let truncated_count = truncate_instance_game_log_files(&instance).expect("清空日志文件");

        assert_eq!(truncated_count, 2);
        assert_eq!(
            std::fs::metadata(log_path).expect("读取 log 元数据").len(),
            0
        );
        assert_eq!(
            std::fs::metadata(txt_path).expect("读取 txt 元数据").len(),
            0
        );
        assert_eq!(
            std::fs::read_to_string(keep_path).expect("读取非日志文件"),
            "保留内容"
        );
    }
}
