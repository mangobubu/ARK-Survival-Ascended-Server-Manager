use crate::{
    app_state::{AppRuntime, current_timestamp_text},
    models::{ServerInstance, ServerStatus},
};

impl AppRuntime {
    pub fn update_instance_status(
        &self,
        instance_id: &str,
        status: ServerStatus,
        last_error: Option<String>,
    ) -> Result<ServerInstance, String> {
        let updated = {
            let mut data = self.lock()?;
            let instance = data
                .instances
                .iter_mut()
                .find(|item| item.id == instance_id)
                .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))?;
            instance.status = status.clone();
            instance.last_error = last_error;
            if matches!(status, ServerStatus::Stopped | ServerStatus::Error) {
                instance.players = 0;
            }
            instance.clone()
        };
        self.persist()?;
        Ok(updated)
    }

    pub fn update_instance_players(
        &self,
        instance_id: &str,
        players: u32,
    ) -> Result<ServerInstance, String> {
        let (updated, changed) = {
            let mut data = self.lock()?;
            let instance = data
                .instances
                .iter_mut()
                .find(|item| item.id == instance_id)
                .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))?;
            let changed = instance.players != players;
            if changed {
                instance.players = players;
            }
            (instance.clone(), changed)
        };
        if changed {
            self.persist()?;
        }
        Ok(updated)
    }

    pub fn update_instance_server_version(
        &self,
        instance_id: &str,
        server_version: String,
    ) -> Result<ServerInstance, String> {
        let server_version = server_version.trim().to_string();
        if server_version.is_empty() {
            return self.get_instance(instance_id);
        }

        let updated = {
            let mut data = self.lock()?;
            let instance = data
                .instances
                .iter_mut()
                .find(|item| item.id == instance_id)
                .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))?;
            if instance.server_version == server_version {
                return Ok(instance.clone());
            }
            instance.server_version = server_version;
            instance.clone()
        };
        self.persist()?;
        Ok(updated)
    }

    pub fn set_instance_pid(
        &self,
        instance_id: &str,
        pid: Option<u32>,
        status: ServerStatus,
    ) -> Result<ServerInstance, String> {
        let timestamp = current_timestamp_text();
        let updated = {
            let mut data = self.lock()?;
            let instance = data
                .instances
                .iter_mut()
                .find(|item| item.id == instance_id)
                .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))?;
            instance.pid = pid;
            instance.status = status.clone();
            if status == ServerStatus::Running {
                instance.last_started_at = Some(timestamp.clone());
                instance.last_error = None;
            }
            if status == ServerStatus::Stopped {
                instance.last_stopped_at = Some(timestamp);
                instance.players = 0;
            }
            if status == ServerStatus::Error {
                instance.players = 0;
            }
            instance.clone()
        };
        self.persist()?;
        Ok(updated)
    }
}
