use super::{ModItem, ServerInstance};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub job_id: String,
    pub instance_id: Option<String>,
    pub phase: String,
    pub percent: Option<f64>,
    pub message: String,
    pub detail: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub bytes_per_second: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupItem {
    pub instance_id: String,
    pub instance_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub path: String,
    pub exported_instances: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported_instances: usize,
    pub skipped_instances: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigBundle {
    pub instance: ServerInstance,
    pub config: Value,
    pub mods: Vec<ModItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedServerConfigPreview {
    pub install_path: String,
    pub name: Option<String>,
    pub map: Option<String>,
    pub map_code: Option<String>,
    pub mode: Option<String>,
    pub game_port: Option<u16>,
    pub query_port: Option<u16>,
    pub rcon_port: Option<u16>,
    pub max_players: Option<u32>,
    pub cluster_id: Option<String>,
    pub server_password: Option<String>,
    pub admin_password: Option<String>,
    pub config: Value,
    pub mods: Vec<ModItem>,
    pub found_files: Vec<String>,
    pub warnings: Vec<String>,
}
