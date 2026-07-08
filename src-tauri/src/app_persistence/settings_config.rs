use crate::{app_persistence::SETTINGS_FILE_NAME, models::GlobalSettings, secret_storage};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn settings_config_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SETTINGS_FILE_NAME)
}

pub(super) fn read_settings_config(data_dir: &Path) -> Result<GlobalSettings, String> {
    let path = settings_config_path(data_dir);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("无法读取应用配置文件 {}：{error}", path.display()))?;
    let value = toml::from_str::<serde_json::Value>(&content)
        .map_err(|error| format!("应用配置文件格式无效 {}：{error}", path.display()))?;
    let mut settings = merge_settings_with_defaults(value)?;
    restore_settings_from_storage(&mut settings)?;
    Ok(settings)
}

pub(super) fn write_settings_config(
    data_dir: &Path,
    settings: &GlobalSettings,
) -> Result<(), String> {
    let path = settings_config_path(data_dir);
    let temp_path = data_dir.join(format!("{SETTINGS_FILE_NAME}.tmp"));
    let settings = settings_for_storage(settings)?;
    let content = toml::to_string_pretty(&settings)
        .map_err(|error| format!("无法序列化应用配置：{error}"))?;
    fs::write(&temp_path, content)
        .map_err(|error| format!("无法写入临时配置文件 {}：{error}", temp_path.display()))?;
    fs::rename(&temp_path, &path)
        .map_err(|error| format!("无法替换配置文件 {}：{error}", path.display()))
}

pub(crate) fn settings_config_contains_unprotected_secret(data_dir: &Path) -> Result<bool, String> {
    let path = settings_config_path(data_dir);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("无法读取应用配置文件 {}：{error}", path.display()))?;
    let value = toml::from_str::<serde_json::Value>(&content)
        .map_err(|error| format!("应用配置文件格式无效 {}：{error}", path.display()))?;
    let Some(secret_key) = value
        .get("webAcmeTencentSecretKey")
        .and_then(Value::as_str)
        .map(str::trim)
    else {
        return Ok(false);
    };
    Ok(!secret_key.is_empty() && !secret_storage::is_protected_secret(secret_key))
}

fn settings_for_storage(settings: &GlobalSettings) -> Result<GlobalSettings, String> {
    let mut settings = settings.clone();
    if !settings.web_acme_tencent_secret_key.is_empty() {
        settings.web_acme_tencent_secret_key =
            secret_storage::protect_secret(&settings.web_acme_tencent_secret_key)?;
    }
    Ok(settings)
}

pub(super) fn restore_settings_from_storage(settings: &mut GlobalSettings) -> Result<(), String> {
    if !settings.web_acme_tencent_secret_key.is_empty() {
        settings.web_acme_tencent_secret_key =
            secret_storage::unprotect_secret(&settings.web_acme_tencent_secret_key)?;
    }
    Ok(())
}

fn merge_settings_with_defaults(incoming: serde_json::Value) -> Result<GlobalSettings, String> {
    let mut merged = serde_json::to_value(GlobalSettings::default())
        .map_err(|error| format!("无法生成默认应用配置：{error}"))?;

    if let (Some(base), Some(next)) = (merged.as_object_mut(), incoming.as_object()) {
        for (key, value) in next {
            base.insert(key.clone(), value.clone());
        }
    }

    serde_json::from_value(merged).map_err(|error| format!("应用配置字段无效：{error}"))
}
