use serde::Serialize;
use serde_json::Value;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::broadcast;

pub const ADD_INSTANCE_CREATED_EVENT: &str = "asa-instance-created";
pub const SETTINGS_CHANGED_EVENT: &str = "asa-global-settings-changed";
pub const LOG_LINE_EVENT: &str = "asa:log-line";
pub const LOGS_CLEARED_EVENT: &str = "asa:logs-cleared";
pub const LOGS_RESET_EVENT: &str = "asa:logs-reset";
pub const INSTANCE_STATUS_EVENT: &str = "asa:instance-status";
pub const JOB_PROGRESS_EVENT: &str = "asa:job-progress";
pub const INSTANCE_CONFIG_CHANGED_EVENT: &str = "asa:instance-config-changed";
pub const INSTANCE_DELETED_EVENT: &str = "asa:instance-deleted";
pub const INSTANCES_CHANGED_EVENT: &str = "asa:instances-changed";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEvent {
    pub id: u64,
    pub name: String,
    pub payload: Value,
}

#[derive(Clone)]
pub struct SyncEventBus {
    sender: broadcast::Sender<SyncEvent>,
    next_id: Arc<AtomicU64>,
}

impl Default for SyncEventBus {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(512);
        Self {
            sender,
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl SyncEventBus {
    pub fn publish(&self, name: impl Into<String>, payload: Value) {
        let event = SyncEvent {
            id: self.next_id.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            payload,
        };
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.sender.subscribe()
    }
}
