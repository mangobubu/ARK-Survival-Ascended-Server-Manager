use super::ports::{instance_uses_port, system_port_unavailable_reason};
use crate::models::ServerInstance;
use serde_json::Value;

pub(crate) fn instance_with_config_metadata(
    instances: &[ServerInstance],
    current: &ServerInstance,
    config: &Value,
) -> Result<ServerInstance, String> {
    let mut updated = current.clone();

    if let Some(session_name) = trimmed_config_string(config, "sessionName") {
        if session_name.is_empty() {
            return Err("服务器名称不能为空".to_string());
        }
        updated.name = session_name;
    }
    if let Some(game_port) = config_u16(config, "gamePort") {
        updated.game_port = game_port;
    }
    if let Some(query_port) = config_u16(config, "queryPort") {
        updated.query_port = query_port;
    }
    if let Some(rcon_port) = config_u16(config, "rconPort") {
        updated.rcon_port = rcon_port;
    }
    if let Some(max_players) = config_u32(config, "maxPlayers") {
        updated.max_players = max_players;
    }
    if let Some(cluster_id) = trimmed_config_string(config, "clusterId") {
        updated.cluster_id = cluster_id;
    }
    if let Some(pve) = config.get("pve").and_then(Value::as_bool) {
        updated.mode = if pve { "PvE" } else { "PvP" }.to_string();
    }

    validate_updated_instance_metadata(instances, current, &updated)?;
    Ok(updated)
}

fn validate_updated_instance_metadata(
    instances: &[ServerInstance],
    current: &ServerInstance,
    updated: &ServerInstance,
) -> Result<(), String> {
    if updated.name.trim().is_empty() {
        return Err("服务器名称不能为空".to_string());
    }
    if instances
        .iter()
        .any(|item| item.id != current.id && item.name.eq_ignore_ascii_case(updated.name.trim()))
    {
        return Err(format!("实例名称已存在：{}", updated.name.trim()));
    }
    if [updated.game_port, updated.query_port, updated.rcon_port]
        .iter()
        .any(|port| *port < 1024)
    {
        return Err("端口必须在 1024-65535 范围内".to_string());
    }
    if updated.game_port == updated.query_port
        || updated.game_port == updated.rcon_port
        || updated.query_port == updated.rcon_port
    {
        return Err("游戏端口、查询端口和 RCON 端口不能重复".to_string());
    }

    let other_instances: Vec<ServerInstance> = instances
        .iter()
        .filter(|item| item.id != current.id)
        .cloned()
        .collect();
    for (port, label) in [
        (updated.game_port, "游戏端口"),
        (updated.query_port, "查询端口"),
        (updated.rcon_port, "RCON 端口"),
    ] {
        if instance_uses_port(&other_instances, port) {
            return Err(format!("{label} {port} 已被其他实例占用"));
        }
    }
    for (next_port, previous_port, label) in [
        (updated.game_port, current.game_port, "游戏端口"),
        (updated.query_port, current.query_port, "查询端口"),
        (updated.rcon_port, current.rcon_port, "RCON 端口"),
    ] {
        if next_port != previous_port
            && let Some(reason) = system_port_unavailable_reason(next_port)
        {
            return Err(format!("{label} {next_port} 不可用：{reason}"));
        }
    }
    Ok(())
}

fn trimmed_config_string(config: &Value, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .map(ToString::to_string)
}

fn config_u16(config: &Value, key: &str) -> Option<u16> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
}

fn config_u32(config: &Value, key: &str) -> Option<u32> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}
