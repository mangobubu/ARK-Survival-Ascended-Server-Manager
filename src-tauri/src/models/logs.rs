use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LogSource {
    #[default]
    Application,
    Server,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerLogKind {
    Console,
    File,
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
