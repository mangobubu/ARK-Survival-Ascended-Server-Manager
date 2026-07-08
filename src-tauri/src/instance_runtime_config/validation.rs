use crate::{ark_config, models::AddInstancePayload};
use serde_json::{Value, json};

pub(crate) fn validate_instance_payload(payload: &AddInstancePayload) -> Result<(), String> {
    if payload.name.trim().is_empty() {
        return Err("实例名称不能为空".to_string());
    }
    if payload.map_code.trim().is_empty() {
        return Err("地图代码不能为空".to_string());
    }
    if payload.install_path.trim().is_empty() {
        return Err("实例目录不能为空".to_string());
    }
    if payload.admin_password.trim().is_empty() {
        return Err("管理员密码不能为空".to_string());
    }
    if [payload.game_port, payload.query_port, payload.rcon_port]
        .iter()
        .any(|port| *port < 1024)
    {
        return Err("端口必须在 1024-65535 范围内".to_string());
    }
    if payload.game_port == payload.query_port
        || payload.game_port == payload.rcon_port
        || payload.query_port == payload.rcon_port
    {
        return Err("游戏端口、查询端口和 RCON 端口不能重复".to_string());
    }
    Ok(())
}

pub(crate) fn normalize_required_rcon_config(mut config: Value) -> Result<Value, String> {
    {
        let Some(config_map) = config.as_object_mut() else {
            return Err("实例配置格式无效".to_string());
        };
        let admin_password = config_map
            .get("adminPassword")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if admin_password.trim().is_empty() {
            return Err("管理员密码不能为空，RCON 必须启用并设置密码".to_string());
        }
        config_map.insert("rconEnabled".to_string(), json!(true));
    }

    ark_config::validate_visibility_access(&config)?;
    Ok(config)
}
