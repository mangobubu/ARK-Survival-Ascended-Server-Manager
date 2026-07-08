use crate::{
    ark_config_values::{number_u16, text},
    models::ServerInstance,
};
use serde_json::Value;

pub(super) fn append_session_settings(
    lines: &mut Vec<String>,
    instance: &ServerInstance,
    config: &Value,
) {
    lines.extend([
        "[SessionSettings]".to_string(),
        format!(
            "SessionName={}",
            text(config, "sessionName", &instance.name)
        ),
        format!(
            "Port={}",
            number_u16(config, "gamePort", instance.game_port)
        ),
        format!(
            "QueryPort={}",
            number_u16(config, "queryPort", instance.query_port)
        ),
    ]);
}
