use super::WEB_IP_WHITELIST_CHINA_MAINLAND;
use serde::{Deserialize, Deserializer, Serialize};

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
    #[serde(default)]
    pub web_https_enabled: bool,
    #[serde(default)]
    pub web_acme_auto_issue_enabled: bool,
    #[serde(default = "default_web_acme_directory_url")]
    pub web_acme_directory_url: String,
    #[serde(default)]
    pub web_acme_account_email: String,
    #[serde(default)]
    pub web_acme_tencent_secret_id: String,
    #[serde(default)]
    pub web_acme_tencent_secret_key: String,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WindowCloseBehavior {
    #[default]
    AskEveryTime,
    MinimizeToTray,
    ExitApp,
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

pub fn default_web_acme_directory_url() -> String {
    "https://acme-v02.api.letsencrypt.org/directory".to_string()
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
            web_https_enabled: false,
            web_acme_auto_issue_enabled: false,
            web_acme_directory_url: default_web_acme_directory_url(),
            web_acme_account_email: String::new(),
            web_acme_tencent_secret_id: String::new(),
            web_acme_tencent_secret_key: String::new(),
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
