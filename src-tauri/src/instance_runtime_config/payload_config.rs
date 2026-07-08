use crate::{
    asa_config_metadata,
    models::{AddInstancePayload, ServerInstance},
};
use serde_json::{Map, Value, json};

fn default_config_from_payload(payload: &AddInstancePayload, instance: &ServerInstance) -> Value {
    let mut map = Map::new();
    asa_config_metadata::apply_static_defaults(&mut map);
    map.insert("sessionName".to_string(), json!(instance.name));
    map.insert("serverPassword".to_string(), json!(payload.server_password));
    map.insert("adminPassword".to_string(), json!(payload.admin_password));
    map.insert("gamePort".to_string(), json!(payload.game_port));
    map.insert("queryPort".to_string(), json!(payload.query_port));
    map.insert("rconPort".to_string(), json!(payload.rcon_port));
    map.insert("clusterId".to_string(), json!(payload.cluster_id));
    map.insert("maxPlayers".to_string(), json!(payload.max_players));
    map.insert("pve".to_string(), json!(payload.mode == "PvE"));
    map.insert("autoUpdateServer".to_string(), json!(payload.auto_install));
    Value::Object(map)
}

pub(crate) fn config_from_payload(
    payload: &AddInstancePayload,
    instance: &ServerInstance,
) -> Value {
    let mut config = match default_config_from_payload(payload, instance) {
        Value::Object(map) => map,
        _ => Map::new(),
    };

    if let Some(Value::Object(imported_config)) = &payload.imported_config {
        for (key, value) in imported_config {
            config.insert(key.clone(), value.clone());
        }
    }

    apply_payload_config_overrides(&mut config, payload, instance);
    Value::Object(config)
}

fn apply_payload_config_overrides(
    config: &mut Map<String, Value>,
    payload: &AddInstancePayload,
    instance: &ServerInstance,
) {
    config.insert("sessionName".to_string(), json!(instance.name));
    config.insert("serverPassword".to_string(), json!(payload.server_password));
    config.insert("adminPassword".to_string(), json!(payload.admin_password));
    config.insert("gamePort".to_string(), json!(payload.game_port));
    config.insert("queryPort".to_string(), json!(payload.query_port));
    config.insert("rconEnabled".to_string(), json!(true));
    config.insert("rconPort".to_string(), json!(payload.rcon_port));
    config.insert("clusterId".to_string(), json!(payload.cluster_id));
    config.insert("maxPlayers".to_string(), json!(payload.max_players));
    config.insert("pve".to_string(), json!(payload.mode == "PvE"));
    config.insert("autoUpdateServer".to_string(), json!(payload.auto_install));
    config.insert(
        "visibility".to_string(),
        json!(if payload.server_password.trim().is_empty() {
            "public"
        } else {
            "private"
        }),
    );
}
