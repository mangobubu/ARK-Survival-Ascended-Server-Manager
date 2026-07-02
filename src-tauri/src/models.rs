use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ASA_DEDICATED_SERVER_APP_ID: &str = "2430930";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSettings {
    pub steam_cmd_path: String,
    pub server_storage_path: String,
    pub backup_storage_path: String,
    pub language: String,
    pub theme: String,
    pub auto_update_on_start: bool,
    pub auto_restart_on_crash: bool,
    pub max_backup_retention: u32,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            steam_cmd_path: "C:\\SteamCMD".to_string(),
            server_storage_path: "D:\\ASA-Server".to_string(),
            backup_storage_path: "D:\\ASA-Backups".to_string(),
            language: "zh-CN".to_string(),
            theme: "dark".to_string(),
            auto_update_on_start: true,
            auto_restart_on_crash: true,
            max_backup_retention: 7,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerStatus {
    Running,
    Stopped,
    Starting,
    Updating,
    BackingUp,
    Error,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInstance {
    pub id: String,
    pub name: String,
    pub map: String,
    pub map_code: String,
    pub mode: String,
    pub status: ServerStatus,
    pub game_port: u16,
    pub query_port: u16,
    pub players: u32,
    pub max_players: u32,
    pub install_path: String,
    pub rcon_port: u16,
    pub cluster_id: String,
    pub description: String,
    pub pid: Option<u32>,
    pub last_started_at: Option<String>,
    pub last_stopped_at: Option<String>,
    pub version_state: String,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddInstancePayload {
    pub id: Option<String>,
    pub name: String,
    pub map: String,
    pub map_code: String,
    pub mode: String,
    pub status: Option<ServerStatus>,
    pub game_port: u16,
    pub query_port: u16,
    pub players: Option<u32>,
    pub max_players: u32,
    pub install_path: String,
    pub rcon_port: u16,
    pub cluster_id: String,
    pub server_password: String,
    pub admin_password: String,
    pub auto_install: bool,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortCheckResult {
    pub port: u16,
    pub available: bool,
    pub exists: bool,
    pub suggested_port: Option<u16>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModItem {
    pub id: String,
    pub name: String,
    pub version: String,
    pub size: String,
    pub enabled: bool,
    pub update_available: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub id: u64,
    pub time: String,
    pub instance: String,
    pub level: String,
    pub message: String,
}

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
