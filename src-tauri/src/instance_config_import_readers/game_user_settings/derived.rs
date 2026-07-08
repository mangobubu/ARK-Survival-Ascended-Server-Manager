use crate::{
    instance_config_import_ini::IniDocument,
    instance_config_import_mapping::{parse_active_mods, text_from_config},
    models::ModItem,
};
use serde_json::{Map, Value, json};

use super::super::SERVER_SETTINGS;

pub(super) fn read_derived_settings(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    mods: &mut Vec<ModItem>,
) {
    infer_visibility_from_server_password(config);
    read_active_mods(document, mods);
}

fn infer_visibility_from_server_password(config: &mut Map<String, Value>) {
    if let Some(server_password) = text_from_config(config, "serverPassword") {
        config.insert(
            "visibility".to_string(),
            json!(if server_password.is_empty() {
                "public"
            } else {
                "private"
            }),
        );
    }
}

fn read_active_mods(document: &IniDocument, mods: &mut Vec<ModItem>) {
    if let Some(active_mods) = document.get(SERVER_SETTINGS, "ActiveMods") {
        *mods = parse_active_mods(active_mods);
    }
}
