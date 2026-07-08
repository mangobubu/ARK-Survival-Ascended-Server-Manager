use crate::{app_state::AppRuntime, models::ServerInstance};

impl AppRuntime {
    pub fn list_instances(&self) -> Result<Vec<ServerInstance>, String> {
        Ok(self.lock()?.instances.clone())
    }

    pub fn get_instance(&self, instance_id: &str) -> Result<ServerInstance, String> {
        self.lock()?
            .instances
            .iter()
            .find(|instance| instance.id == instance_id)
            .cloned()
            .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))
    }
}
