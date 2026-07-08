#[cfg(test)]
use crate::asa_config_defaults::config_default_count;
use crate::asa_config_defaults::config_defaults;
use serde::Serialize;
use serde_json::{Map, Number, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AsaConfigTarget {
    ManagerOnly,
    LaunchArgument,
    GameUserSettingsServerSettings,
    GameIniShooterGameMode,
    EngineIniIpNetDriver,
}

#[derive(Clone, Copy, Debug)]
pub enum AsaConfigDefaultValue {
    Bool(bool),
    U32(u32),
    F64(f64),
    Text(&'static str),
    EmptyArray,
}

impl AsaConfigDefaultValue {
    pub(crate) fn to_json(self) -> Value {
        match self {
            Self::Bool(value) => Value::Bool(value),
            Self::U32(value) => Value::Number(Number::from(value)),
            Self::F64(value) => Value::Number(Number::from_f64(value).unwrap_or_else(|| 0.into())),
            Self::Text(value) => Value::String(value.to_string()),
            Self::EmptyArray => Value::Array(Vec::new()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AsaConfigDefault {
    pub key: &'static str,
    pub value: AsaConfigDefaultValue,
    pub target: AsaConfigTarget,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsaConfigFieldMetadata {
    pub key: &'static str,
    pub target: AsaConfigTarget,
    pub default_value: Value,
    pub sensitive_export: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsaConfigMetadataDocument {
    pub fields: Vec<AsaConfigFieldMetadata>,
    pub sensitive_export_keys: &'static [&'static str],
    pub dynamic_instance_keys: &'static [&'static str],
}

/// 导出实例配置时必须脱敏的字段。
///
/// 这些字段属于 ASA 服务端凭据或 Web 管理凭据，统一放在配置元数据模块中，
/// 避免导入导出、安全校验和配置默认值之间各自维护一份敏感字段清单。
pub const CONFIG_EXPORT_SENSITIVE_KEYS: &[&str] = &[
    "serverPassword",
    "adminPassword",
    "spectatorPassword",
    "webAdminPassword",
    "webAcmeTencentSecretKey",
];

/// 创建实例时由实例基础信息动态生成的配置字段。
///
/// 这些字段不进入静态默认值表，但属于实例配置元数据的一部分。
pub const DYNAMIC_INSTANCE_CONFIG_KEYS: &[&str] = &[
    "sessionName",
    "serverPassword",
    "adminPassword",
    "gamePort",
    "queryPort",
    "rconPort",
    "clusterId",
    "maxPlayers",
    "pve",
    "autoUpdateServer",
];

pub fn apply_static_defaults(map: &mut Map<String, Value>) {
    for item in config_defaults() {
        map.insert(item.key.to_string(), item.value.to_json());
    }
}

pub fn is_sensitive_export_key(key: &str) -> bool {
    CONFIG_EXPORT_SENSITIVE_KEYS.contains(&key)
}

pub fn config_metadata() -> AsaConfigMetadataDocument {
    AsaConfigMetadataDocument {
        fields: config_defaults()
            .map(|item| AsaConfigFieldMetadata {
                key: item.key,
                target: item.target,
                default_value: item.value.to_json(),
                sensitive_export: is_sensitive_export_key(item.key),
            })
            .collect(),
        sensitive_export_keys: CONFIG_EXPORT_SENSITIVE_KEYS,
        dynamic_instance_keys: DYNAMIC_INSTANCE_CONFIG_KEYS,
    }
}

#[tauri::command]
pub fn get_asa_config_metadata() -> AsaConfigMetadataDocument {
    config_metadata()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn 敏感导出字段元数据不重复且覆盖核心凭据() {
        let mut keys = HashSet::new();
        for key in CONFIG_EXPORT_SENSITIVE_KEYS {
            assert!(keys.insert(*key), "敏感导出字段重复：{key}");
        }

        for required in [
            "serverPassword",
            "adminPassword",
            "spectatorPassword",
            "webAdminPassword",
            "webAcmeTencentSecretKey",
        ] {
            assert!(
                is_sensitive_export_key(required),
                "敏感导出字段缺少核心凭据：{required}"
            );
        }

        assert!(!is_sensitive_export_key("sessionName"));
    }

    #[test]
    fn 动态实例字段元数据不重复且覆盖创建实例字段() {
        let mut keys = HashSet::new();
        for key in DYNAMIC_INSTANCE_CONFIG_KEYS {
            assert!(keys.insert(*key), "动态实例字段重复：{key}");
        }

        for required in [
            "sessionName",
            "serverPassword",
            "adminPassword",
            "gamePort",
            "queryPort",
            "rconPort",
            "clusterId",
            "maxPlayers",
            "pve",
            "autoUpdateServer",
        ] {
            assert!(
                DYNAMIC_INSTANCE_CONFIG_KEYS.contains(&required),
                "动态实例字段缺少创建实例字段：{required}"
            );
        }
    }

    #[test]
    fn 配置元数据文档包含静态默认值与敏感标记() {
        let metadata = config_metadata();
        let mut keys = HashSet::new();
        for field in &metadata.fields {
            assert!(keys.insert(field.key), "配置元数据字段重复：{}", field.key);
        }

        assert_eq!(metadata.fields.len(), config_default_count());
        assert_eq!(metadata.sensitive_export_keys, CONFIG_EXPORT_SENSITIVE_KEYS);
        assert_eq!(metadata.dynamic_instance_keys, DYNAMIC_INSTANCE_CONFIG_KEYS);
        assert!(
            metadata
                .fields
                .iter()
                .any(|field| field.key == "spectatorPassword"
                    && field.sensitive_export
                    && field.default_value == Value::String(String::new())
                    && field.target == AsaConfigTarget::GameUserSettingsServerSettings)
        );
        assert!(
            metadata
                .fields
                .iter()
                .any(|field| field.key == "rconEnabled"
                    && !field.sensitive_export
                    && field.default_value == Value::Bool(true))
        );
    }
}
