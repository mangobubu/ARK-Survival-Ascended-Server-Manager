use crate::{
    app_persistence::STATE_FILE_NAME,
    app_state::ManagerData,
    models::{GlobalSettings, LogLine, ModItem, ServerInstance},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManagerStateFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) settings: Option<GlobalSettings>,
    #[serde(default)]
    pub(super) instances: Vec<ServerInstance>,
    #[serde(default)]
    pub(super) configs: HashMap<String, Value>,
    #[serde(default)]
    pub(super) mods: HashMap<String, Vec<ModItem>>,
    #[serde(default)]
    pub(super) logs: Vec<LogLine>,
}

pub(super) fn read_state_file(data_dir: &Path) -> Result<ManagerStateFile, String> {
    let path = data_dir.join(STATE_FILE_NAME);
    if !path.exists() {
        return Ok(ManagerStateFile::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("无法读取管理器状态文件 {}：{error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("管理器状态文件格式无效 {}：{error}", path.display()))
}

pub(super) fn write_state_file(data_dir: &Path, data: &ManagerData) -> Result<(), String> {
    let path = data_dir.join(STATE_FILE_NAME);
    let temp_path = data_dir.join(format!("{STATE_FILE_NAME}.tmp"));
    let state = ManagerStateFile {
        settings: None,
        instances: data.instances.clone(),
        configs: data.configs.clone(),
        mods: data.mods.clone(),
        logs: data.logs.clone(),
    };
    let content = serde_json::to_string_pretty(&state)
        .map_err(|error| format!("无法序列化管理器状态：{error}"))?;
    fs::write(&temp_path, content)
        .map_err(|error| format!("无法写入临时状态文件 {}：{error}", temp_path.display()))?;
    fs::rename(&temp_path, &path)
        .map_err(|error| format!("无法替换状态文件 {}：{error}", path.display()))
}
