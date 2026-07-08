use crate::{
    instance_config_import_ini::IniDocument,
    instance_config_import_mapping::{map_text, map_u16},
};
use serde_json::{Map, Value};

use super::super::SERVER_SETTINGS;

pub(super) fn read_text_fields(document: &IniDocument, config: &mut Map<String, Value>) {
    for (ini_key, config_key) in [
        ("ServerPassword", "serverPassword"),
        ("ServerAdminPassword", "adminPassword"),
        ("SpectatorPassword", "spectatorPassword"),
        ("ClusterID", "clusterId"),
        ("ClusterId", "clusterId"),
    ] {
        map_text(document, config, SERVER_SETTINGS, ini_key, config_key);
    }

    map_u16(document, config, SERVER_SETTINGS, "RCONPort", "rconPort");
}
