use crate::{
    app_state::{AppRuntime, normalize_required_rcon_config},
    models::{ServerInstance, ServerStatus},
    rcon,
    rcon_players::parse_list_players_count,
    sync_events::LOG_LINE_EVENT,
    window_controls,
};
use serde_json::Value;
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::timeout;

const PLAYER_COUNT_POLL_TIMEOUT: Duration = Duration::from_secs(4);

pub(crate) struct ReadinessProbe {
    pub(crate) method: String,
    pub(crate) players: u32,
}

pub(crate) async fn execute_rcon_command(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: String,
    command: String,
) -> Result<String, String> {
    let instance = runtime.get_instance(&instance_id)?;
    if matches!(instance.status, ServerStatus::Stopped | ServerStatus::Error) {
        return Err(format!(
            "{} 当前不是运行状态，无法执行 RCON 命令",
            instance.name
        ));
    }

    let command = command.trim().trim_start_matches('/').trim().to_string();
    if command.is_empty() {
        return Err("RCON 命令不能为空".to_string());
    }
    if command.len() > 1024 {
        return Err("RCON 命令过长，请控制在 1024 个字符以内".to_string());
    }

    let config = normalize_required_rcon_config(runtime.get_config(&instance_id)?)?;
    let admin_password = config
        .get("adminPassword")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if admin_password.is_empty() {
        return Err("管理员密码为空，无法执行 RCON 命令".to_string());
    }
    let rcon_port = u16_from_config(&config, "rconPort", instance.rcon_port);

    match rcon::execute("127.0.0.1", rcon_port, &admin_password, &command).await {
        Ok(response) => {
            let _ = emit_instance_log(
                app,
                runtime,
                &instance.name,
                "success",
                &format!("RCON 命令已执行：{command}"),
            );
            Ok(response)
        }
        Err(error) => {
            let _ = emit_instance_log(
                app,
                runtime,
                &instance.name,
                "warn",
                &format!("RCON 命令执行失败：{command}：{error}"),
            );
            Err(error)
        }
    }
}

pub(crate) async fn refresh_instance_players(
    runtime: &AppRuntime,
    instance: ServerInstance,
) -> Result<ServerInstance, String> {
    if matches!(instance.status, ServerStatus::Stopped | ServerStatus::Error) {
        return if instance.players == 0 {
            Ok(instance)
        } else {
            runtime.update_instance_players(&instance.id, 0)
        };
    }

    if !matches!(
        instance.status,
        ServerStatus::Running | ServerStatus::Starting
    ) {
        return Ok(instance);
    }

    let Ok(config) = normalize_required_rcon_config(runtime.get_config(&instance.id)?) else {
        return Ok(instance);
    };
    let admin_password = config
        .get("adminPassword")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if admin_password.is_empty() {
        return Ok(instance);
    }

    let rcon_port = u16_from_config(&config, "rconPort", instance.rcon_port);
    match timeout(
        PLAYER_COUNT_POLL_TIMEOUT,
        rcon::execute("127.0.0.1", rcon_port, &admin_password, "ListPlayers"),
    )
    .await
    {
        Ok(Ok(response)) => {
            runtime.update_instance_players(&instance.id, parse_list_players_count(&response))
        }
        Ok(Err(_)) | Err(_) => Ok(instance),
    }
}

pub(crate) async fn probe_server_readiness(
    instance: &ServerInstance,
    config: &Value,
) -> Result<ReadinessProbe, String> {
    let rcon_enabled = bool_from_config(config, "rconEnabled", true);
    let admin_password = config
        .get("adminPassword")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim();
    if !rcon_enabled {
        return Err("RCON 未启用，无法判定服务端运行状态".to_string());
    }
    if admin_password.is_empty() {
        return Err("管理员密码为空，无法执行 RCON 启动探测".to_string());
    }

    let rcon_port = u16_from_config(config, "rconPort", instance.rcon_port);
    let response = rcon::execute("127.0.0.1", rcon_port, admin_password, "ListPlayers")
        .await
        .map_err(|error| format!("RCON 未就绪：{error}"))?;
    Ok(ReadinessProbe {
        method: format!("RCON ListPlayers 127.0.0.1:{rcon_port}"),
        players: parse_list_players_count(&response),
    })
}

fn bool_from_config(config: &Value, key: &str, fallback: bool) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}

fn u16_from_config(config: &Value, key: &str, fallback: u16) -> u16 {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(fallback)
}

fn emit_instance_log(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    level: &str,
    message: &str,
) -> Result<(), String> {
    let line = runtime.add_log(instance_name, level, message)?;
    window_controls::publish_settings_changed_and_apply(app, LOG_LINE_EVENT, line)
        .map_err(|error| format!("发送日志事件失败：{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 从配置读取_rcon_端口时会校验_u16_边界() {
        assert_eq!(
            u16_from_config(&serde_json::json!({ "rconPort": 32330 }), "rconPort", 1),
            32330
        );
        assert_eq!(
            u16_from_config(&serde_json::json!({ "rconPort": 70000 }), "rconPort", 32330),
            32330
        );
        assert_eq!(
            u16_from_config(&serde_json::json!({}), "rconPort", 32330),
            32330
        );
    }
}
