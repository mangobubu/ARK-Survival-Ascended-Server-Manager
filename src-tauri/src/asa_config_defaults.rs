use crate::asa_config_metadata::{
    AsaConfigDefault, AsaConfigDefaultValue as DefaultValue, AsaConfigTarget as Target,
};

mod engine_ini_ip_net_driver;
mod game_ini_shooter_game_mode;
mod game_user_settings_server_settings;
mod launch_argument;
mod manager_only;

const DEFAULT_GROUPS: &[&[AsaConfigDefault]] = &[
    manager_only::DEFAULTS,
    game_user_settings_server_settings::DEFAULTS,
    game_ini_shooter_game_mode::DEFAULTS,
    launch_argument::DEFAULTS,
    engine_ini_ip_net_driver::DEFAULTS,
];

pub fn config_defaults() -> impl Iterator<Item = &'static AsaConfigDefault> {
    DEFAULT_GROUPS.iter().flat_map(|group| group.iter())
}

#[cfg(test)]
pub fn config_default_count() -> usize {
    DEFAULT_GROUPS.iter().map(|group| group.len()).sum()
}

const fn default(key: &'static str, value: DefaultValue, target: Target) -> AsaConfigDefault {
    AsaConfigDefault { key, value, target }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::asa_config_metadata::apply_static_defaults;
    use serde_json::{Map, Value};
    use std::collections::HashSet;

    #[test]
    fn 静态默认配置键不重复且不包含创建实例动态字段() {
        let mut keys = HashSet::new();
        for item in config_defaults() {
            assert!(keys.insert(item.key), "默认配置键重复：{}", item.key);
            assert!(
                !matches!(
                    item.key,
                    "sessionName"
                        | "serverPassword"
                        | "adminPassword"
                        | "gamePort"
                        | "queryPort"
                        | "rconPort"
                        | "clusterId"
                        | "maxPlayers"
                        | "pve"
                        | "autoUpdateServer"
                ),
                "动态字段不应进入静态默认配置表：{}",
                item.key
            );
        }
    }

    #[test]
    fn 能应用静态默认配置值() {
        let mut map = Map::new();
        apply_static_defaults(&mut map);

        assert_eq!(
            map.get("spectatorPassword"),
            Some(&Value::String(String::new()))
        );
        assert_eq!(map.get("rconEnabled"), Some(&Value::Bool(true)));
        assert_eq!(
            map.get("itemStackOverrides"),
            Some(&Value::Array(Vec::new()))
        );
        assert_eq!(
            map.get("clusterDirOverride"),
            Some(&Value::String("ShooterGame/Saved/clusters".to_string()))
        );
    }
}
