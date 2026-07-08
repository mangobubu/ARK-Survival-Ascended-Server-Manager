use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerStatus {
    Running,
    #[default]
    Stopped,
    Stopping,
    Starting,
    Updating,
    BackingUp,
    Error,
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
    #[serde(default)]
    pub server_version: String,
    pub version_state: String,
    pub last_error: Option<String>,
    #[serde(default)]
    pub skip_auto_update_on_start_once: bool,
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
    pub imported_config: Option<Value>,
    pub imported_mods: Option<Vec<ModItem>>,
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
