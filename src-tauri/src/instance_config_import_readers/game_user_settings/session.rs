use crate::{
    instance_config_import_ini::IniDocument,
    instance_config_import_mapping::{map_text, map_u16, map_u32},
};
use serde_json::{Map, Value};

use super::super::{GAME_SESSION_SETTINGS, SESSION_SETTINGS};

pub(super) fn read_session_settings(document: &IniDocument, config: &mut Map<String, Value>) {
    map_text(
        document,
        config,
        SESSION_SETTINGS,
        "SessionName",
        "sessionName",
    );

    for (ini_key, config_key) in [("Port", "gamePort"), ("QueryPort", "queryPort")] {
        map_u16(document, config, SESSION_SETTINGS, ini_key, config_key);
    }

    map_u32(
        document,
        config,
        GAME_SESSION_SETTINGS,
        "MaxPlayers",
        "maxPlayers",
    );
}
