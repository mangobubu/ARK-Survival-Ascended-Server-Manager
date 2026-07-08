mod booleans;
mod derived;
mod numbers;
mod session;
mod text_fields;

use crate::{instance_config_import_ini::IniDocument, models::ModItem};
use serde_json::{Map, Value};

pub(crate) fn read_game_user_settings(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    mods: &mut Vec<ModItem>,
) {
    session::read_session_settings(document, config);
    text_fields::read_text_fields(document, config);
    numbers::read_numeric_settings(document, config);
    booleans::read_boolean_settings(document, config);
    derived::read_derived_settings(document, config, mods);
}
