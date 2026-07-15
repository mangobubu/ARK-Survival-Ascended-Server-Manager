use crate::{
    ark_config_mods::active_mod_ids,
    ark_config_values::append_custom_ini_settings,
    models::{ModItem, ServerInstance},
};
use serde_json::Value;

mod map_specific;
mod server_balance;
mod server_basics;
mod session_settings;

pub(crate) fn render_game_user_settings(
    instance: &ServerInstance,
    config: &Value,
    mods: &[ModItem],
) -> String {
    let active_mods = active_mod_ids(mods).join(",");
    let mut lines = vec!["[ServerSettings]".to_string()];

    server_basics::append_basic_server_settings(&mut lines, instance, config);
    server_balance::append_balance_server_settings(&mut lines, config);
    map_specific::append_map_specific_settings(&mut lines, instance, config);
    lines.push(format!("ActiveMods={active_mods}"));
    append_custom_ini_settings(&mut lines, config, "customServerSettings");
    lines.push(String::new());
    session_settings::append_session_settings(&mut lines, instance, config);
    lines.push(String::new());

    lines.join("\r\n")
}
