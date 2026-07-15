use crate::{
    app_state::{AppRuntime, ManagerData, normalize_required_rcon_config},
    instance_runtime_config::instance_with_config_metadata,
    models::{ModItem, ServerInstance},
};
use serde_json::{Value, json};

impl AppRuntime {
    pub fn get_config(&self, instance_id: &str) -> Result<Value, String> {
        let data = self.lock()?;
        Ok(data
            .configs
            .get(instance_id)
            .cloned()
            .unwrap_or_else(|| json!({})))
    }

    pub fn get_mods(&self, instance_id: &str) -> Result<Vec<ModItem>, String> {
        let data = self.lock()?;
        Ok(data.mods.get(instance_id).cloned().unwrap_or_default())
    }

    pub fn save_config_and_mods(
        &self,
        instance_id: &str,
        config: Value,
        mods: Vec<ModItem>,
    ) -> Result<ServerInstance, String> {
        let config = normalize_required_rcon_config(config)?;
        let updated = {
            let mut data = self.lock()?;
            let (instance_index, updated) =
                validated_instance_metadata(&data, instance_id, &config)?;
            data.instances[instance_index] = updated.clone();
            data.configs.insert(instance_id.to_string(), config);
            data.mods.insert(instance_id.to_string(), mods);
            updated
        };
        self.persist()?;
        Ok(updated)
    }

    pub(crate) fn validate_config_metadata(
        &self,
        instance_id: &str,
        config: &Value,
    ) -> Result<ServerInstance, String> {
        let data = self.lock()?;
        validated_instance_metadata(&data, instance_id, config).map(|(_, instance)| instance)
    }
}

fn validated_instance_metadata(
    data: &ManagerData,
    instance_id: &str,
    config: &Value,
) -> Result<(usize, ServerInstance), String> {
    let instance_index = data
        .instances
        .iter()
        .position(|item| item.id == instance_id)
        .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))?;
    let current = &data.instances[instance_index];
    let updated = instance_with_config_metadata(&data.instances, current, config)?;
    Ok((instance_index, updated))
}
