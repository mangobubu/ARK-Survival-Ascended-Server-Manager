use crate::{instance_config_import_ini::IniDocument, instance_config_import_mapping::map_u32};
use serde_json::{Map, Value};

use super::ENGINE_IP_NET_DRIVER_SETTINGS;
pub(crate) fn read_engine_ini(document: &IniDocument, config: &mut Map<String, Value>) {
    for (ini_key, config_key) in [
        ("NetServerMaxTickRate", "networkTickRate"),
        ("MaxClientRate", "maxClientRate"),
    ] {
        map_u32(
            document,
            config,
            ENGINE_IP_NET_DRIVER_SETTINGS,
            ini_key,
            config_key,
        );
    }
}
