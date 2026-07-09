use std::fs;

use crate::{
    app_state::{AppRuntime, normalize_required_rcon_config, now_millis},
    ark_config,
    instance_runtime_config::{
        config_from_payload, ensure_port_available, normalize_path_text, sanitize_imported_mods,
        validate_instance_payload,
    },
    models::{AddInstancePayload, ServerInstance, ServerStatus},
};

impl AppRuntime {
    pub fn upsert_instance(&self, instance: ServerInstance) -> Result<ServerInstance, String> {
        {
            let mut data = self.lock()?;
            if let Some(current) = data
                .instances
                .iter_mut()
                .find(|item| item.id == instance.id)
            {
                *current = instance.clone();
            } else {
                data.instances.push(instance.clone());
            }
        }
        self.persist()?;
        Ok(instance)
    }

    pub fn delete_instance(&self, instance_id: &str) -> Result<ServerInstance, String> {
        let instance = self.get_instance(instance_id)?;
        if matches!(
            instance.status,
            ServerStatus::Running
                | ServerStatus::Starting
                | ServerStatus::Stopping
                | ServerStatus::Updating
                | ServerStatus::BackingUp
        ) {
            return Err(format!(
                "{} 正在运行任务，请先停止实例后再删除",
                instance.name
            ));
        }
        if self.is_update_running(instance_id)? {
            return Err(format!(
                "{} 正在安装/更新，请先取消任务后再删除",
                instance.name
            ));
        }
        {
            let processes = self.lock_processes()?;
            if processes.contains_key(instance_id) {
                return Err(format!(
                    "{} 的服务端进程仍在运行，请先停止实例",
                    instance.name
                ));
            }
        }

        let removed = {
            let mut data = self.lock()?;
            let index = data
                .instances
                .iter()
                .position(|item| item.id == instance_id)
                .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))?;
            let removed = data.instances.remove(index);
            data.configs.remove(instance_id);
            data.mods.remove(instance_id);
            removed
        };
        self.persist()?;
        Ok(removed)
    }

    pub fn create_instance(&self, payload: AddInstancePayload) -> Result<ServerInstance, String> {
        validate_instance_payload(&payload)?;

        let id = payload
            .id
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| format!("asa-{}", now_millis()));
        let install_path = normalize_path_text(&payload.install_path);

        {
            let data = self.lock()?;
            if data.instances.iter().any(|item| item.id == id) {
                return Err(format!("实例 ID 已存在：{id}"));
            }
            if data
                .instances
                .iter()
                .any(|item| item.name.eq_ignore_ascii_case(payload.name.trim()))
            {
                return Err(format!("实例名称已存在：{}", payload.name.trim()));
            }
            ensure_port_available(&data.instances, payload.game_port, "游戏端口")?;
            ensure_port_available(&data.instances, payload.query_port, "查询端口")?;
            ensure_port_available(&data.instances, payload.rcon_port, "RCON 端口")?;
        }

        fs::create_dir_all(&install_path)
            .map_err(|error| format!("无法创建实例目录 {}：{error}", install_path.display()))?;

        let instance = ServerInstance {
            id: id.clone(),
            name: payload.name.trim().to_string(),
            map: payload.map.trim().to_string(),
            map_code: payload.map_code.trim().to_string(),
            mode: payload.mode.clone(),
            status: ServerStatus::Stopped,
            game_port: payload.game_port,
            query_port: payload.query_port,
            players: 0,
            max_players: payload.max_players,
            install_path: install_path.to_string_lossy().into_owned(),
            rcon_port: payload.rcon_port,
            cluster_id: payload.cluster_id.trim().to_string(),
            description: payload.description.trim().to_string(),
            pid: None,
            last_started_at: None,
            last_stopped_at: None,
            server_version: String::new(),
            version_state: "未安装".to_string(),
            last_error: None,
            skip_auto_update_on_start_once: false,
        };

        let config = normalize_required_rcon_config(config_from_payload(&payload, &instance))?;
        let mods = sanitize_imported_mods(&payload);
        ark_config::apply_instance_config(&instance, &config, &mods)?;

        {
            let mut data = self.lock()?;
            data.configs.insert(id.clone(), config);
            data.mods.insert(id.clone(), mods);
            data.instances.push(instance.clone());
        }
        self.persist()?;
        Ok(instance)
    }
}
