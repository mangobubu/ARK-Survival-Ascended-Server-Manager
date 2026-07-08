pub(crate) use crate::instance_runtime_config::normalize_required_rcon_config;
use crate::{
    app_logs::recover_stale_update_statuses,
    app_persistence::{
        migrate_manager_data_secrets, read_data, settings_config_contains_unprotected_secret,
        settings_config_path, write_data,
    },
    models::{GlobalSettings, LogLine, ModItem, ServerInstance},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicBool},
};
use tauri::AppHandle;

mod instances;
mod logs;
mod paths;
mod runtime_tasks;
mod time;

use paths::resolve_app_data_dir;
pub use time::{current_time_text, current_timestamp_text, now_millis};
use tokio::process::Child;

#[derive(Clone)]
pub struct AppRuntime {
    data_dir: Arc<PathBuf>,
    data: Arc<Mutex<ManagerData>>,
    processes: Arc<Mutex<HashMap<String, Child>>>,
    update_cancels: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerData {
    pub settings: GlobalSettings,
    pub instances: Vec<ServerInstance>,
    pub configs: HashMap<String, Value>,
    pub mods: HashMap<String, Vec<ModItem>>,
    pub logs: Vec<LogLine>,
}

impl AppRuntime {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let data_dir = resolve_app_data_dir(app)?;
        fs::create_dir_all(&data_dir)
            .map_err(|error| format!("无法创建应用数据目录 {}：{error}", data_dir.display()))?;

        let had_settings_config = settings_config_path(&data_dir).exists();
        let settings_config_needs_secret_protection =
            had_settings_config && settings_config_contains_unprotected_secret(&data_dir)?;
        let mut data = read_data(&data_dir)?;
        let migrated_secrets = migrate_manager_data_secrets(&mut data)?;
        let recovered_stale_updates = recover_stale_update_statuses(&mut data);
        if !had_settings_config
            || settings_config_needs_secret_protection
            || migrated_secrets
            || recovered_stale_updates
        {
            write_data(&data_dir, &data)?;
        }
        Ok(Self {
            data_dir: Arc::new(data_dir),
            data: Arc::new(Mutex::new(data)),
            processes: Arc::new(Mutex::new(HashMap::new())),
            update_cancels: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn settings(&self) -> Result<GlobalSettings, String> {
        Ok(self.lock()?.settings.clone())
    }

    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.as_ref().clone()
    }

    pub fn save_settings(&self, settings: GlobalSettings) -> Result<GlobalSettings, String> {
        {
            let mut data = self.lock()?;
            data.settings = settings.clone();
        }
        self.persist()?;
        Ok(settings)
    }

    pub fn snapshot(&self) -> Result<ManagerData, String> {
        Ok(self.lock()?.clone())
    }

    pub fn replace_from_import(
        &self,
        bundles: Vec<crate::models::InstanceConfigBundle>,
    ) -> Result<(usize, usize), String> {
        let mut imported = 0;
        let mut skipped = 0;
        {
            let mut data = self.lock()?;
            for bundle in bundles {
                if data
                    .instances
                    .iter()
                    .any(|item| item.id == bundle.instance.id)
                {
                    skipped += 1;
                    continue;
                }
                data.configs
                    .insert(bundle.instance.id.clone(), bundle.config);
                data.mods.insert(bundle.instance.id.clone(), bundle.mods);
                data.instances.push(bundle.instance);
                imported += 1;
            }
        }
        self.persist()?;
        Ok((imported, skipped))
    }

    pub fn persist(&self) -> Result<(), String> {
        let data = self.lock()?.clone();
        write_data(&self.data_dir, &data)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ManagerData>, String> {
        self.data
            .lock()
            .map_err(|_| "管理器状态锁已损坏".to_string())
    }
}

#[cfg(test)]
mod tests;
