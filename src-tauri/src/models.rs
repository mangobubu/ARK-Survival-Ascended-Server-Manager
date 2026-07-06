use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub const ASA_DEDICATED_SERVER_APP_ID: &str = "2430930";
pub const WEB_IP_WHITELIST_CHINA_MAINLAND: &str = "CN_MAINLAND";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebIpWhitelistEntry {
    pub value: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub note: String,
}

impl WebIpWhitelistEntry {
    pub fn new(
        value: impl Into<String>,
        group: impl Into<String>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            value: value.into(),
            group: group.into(),
            note: note.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum WebIpWhitelistEntryInput {
    Text(String),
    Entry(WebIpWhitelistEntry),
}

fn deserialize_web_ip_whitelist<'de, D>(
    deserializer: D,
) -> Result<Vec<WebIpWhitelistEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries =
        Option::<Vec<WebIpWhitelistEntryInput>>::deserialize(deserializer)?.unwrap_or_default();
    let normalized: Vec<WebIpWhitelistEntry> = entries
        .into_iter()
        .map(|entry| match entry {
            WebIpWhitelistEntryInput::Text(value) => WebIpWhitelistEntry::new(value, "", ""),
            WebIpWhitelistEntryInput::Entry(entry) => entry,
        })
        .collect();
    Ok(if normalized.is_empty() {
        default_web_ip_whitelist()
    } else {
        normalized
    })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebSecurityBanRecord {
    pub ip: String,
    pub reason: String,
    pub source: String,
    pub banned_at_ms: u64,
    pub remaining_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebSecurityUnbanResult {
    pub ip: String,
    pub existed: bool,
}

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
    #[serde(default)]
    pub web_management_enabled: bool,
    #[serde(default = "default_web_server_port")]
    pub web_server_port: u16,
    #[serde(default = "default_web_admin_username")]
    pub web_admin_username: String,
    #[serde(default)]
    pub web_admin_password: String,
    #[serde(default)]
    pub web_reverse_proxy_enabled: bool,
    #[serde(default)]
    pub web_reverse_proxy_domain: String,
    #[serde(default = "default_web_reverse_proxy_port")]
    pub web_reverse_proxy_port: u16,
    #[serde(
        default,
        rename = "webReverseProxyOpenRestyPath",
        alias = "webReverseProxyNginxPath"
    )]
    pub web_reverse_proxy_openresty_path: String,
    #[serde(default = "default_web_login_failure_ban_threshold")]
    pub web_login_failure_ban_threshold: u32,
    #[serde(default = "default_web_login_failure_ban_seconds")]
    pub web_login_failure_ban_seconds: u32,
    #[serde(default = "default_web_captcha_charset")]
    pub web_captcha_charset: String,
    #[serde(default = "default_web_captcha_font_size")]
    pub web_captcha_font_size: u32,
    #[serde(default = "default_web_captcha_noise_points")]
    pub web_captcha_noise_points: u32,
    #[serde(default = "default_web_captcha_length")]
    pub web_captcha_length: u32,
    #[serde(
        default = "default_web_ip_whitelist",
        deserialize_with = "deserialize_web_ip_whitelist"
    )]
    pub web_ip_whitelist: Vec<WebIpWhitelistEntry>,
    #[serde(default = "default_window_close_behavior")]
    pub window_close_behavior: WindowCloseBehavior,
    #[serde(default = "default_global_toggle_shortcut_key")]
    pub global_toggle_shortcut_key: String,
    #[serde(default)]
    pub hide_tray_icon_when_minimized: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WindowCloseBehavior {
    AskEveryTime,
    MinimizeToTray,
    ExitApp,
}

impl Default for WindowCloseBehavior {
    fn default() -> Self {
        Self::AskEveryTime
    }
}

pub fn default_window_close_behavior() -> WindowCloseBehavior {
    WindowCloseBehavior::AskEveryTime
}

pub fn default_global_toggle_shortcut_key() -> String {
    "A".to_string()
}

pub fn default_web_server_port() -> u16 {
    18080
}

pub fn default_web_admin_username() -> String {
    "admin".to_string()
}

pub fn default_web_reverse_proxy_port() -> u16 {
    18081
}

pub fn default_web_login_failure_ban_threshold() -> u32 {
    5
}

pub fn default_web_login_failure_ban_seconds() -> u32 {
    30 * 60
}

pub fn default_web_captcha_charset() -> String {
    "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".to_string()
}

pub fn default_web_captcha_font_size() -> u32 {
    32
}

pub fn default_web_captcha_noise_points() -> u32 {
    24
}

pub fn default_web_captcha_length() -> u32 {
    4
}

pub fn default_web_ip_whitelist() -> Vec<WebIpWhitelistEntry> {
    vec![WebIpWhitelistEntry::new(
        WEB_IP_WHITELIST_CHINA_MAINLAND,
        "默认",
        "内置中国大陆 IPv4 CIDR",
    )]
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
            web_management_enabled: false,
            web_server_port: 18080,
            web_admin_username: default_web_admin_username(),
            web_admin_password: String::new(),
            web_reverse_proxy_enabled: false,
            web_reverse_proxy_domain: String::new(),
            web_reverse_proxy_port: default_web_reverse_proxy_port(),
            web_reverse_proxy_openresty_path: String::new(),
            web_login_failure_ban_threshold: default_web_login_failure_ban_threshold(),
            web_login_failure_ban_seconds: default_web_login_failure_ban_seconds(),
            web_captcha_charset: default_web_captcha_charset(),
            web_captcha_font_size: default_web_captcha_font_size(),
            web_captcha_noise_points: default_web_captcha_noise_points(),
            web_captcha_length: default_web_captcha_length(),
            web_ip_whitelist: default_web_ip_whitelist(),
            window_close_behavior: WindowCloseBehavior::AskEveryTime,
            global_toggle_shortcut_key: default_global_toggle_shortcut_key(),
            hide_tray_icon_when_minimized: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerStatus {
    Running,
    Stopped,
    Stopping,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LogSource {
    Application,
    Server,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerLogKind {
    Console,
    File,
}

impl Default for LogSource {
    fn default() -> Self {
        Self::Application
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub id: u64,
    pub time: String,
    #[serde(default)]
    pub source: LogSource,
    #[serde(default)]
    pub server_log_kind: Option<ServerLogKind>,
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
