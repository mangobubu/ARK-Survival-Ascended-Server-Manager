use super::AppRuntime;
use crate::models::ServerInstance;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::process::Child;

impl AppRuntime {
    pub fn lock_processes(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Child>>, String> {
        self.processes
            .lock()
            .map_err(|_| "运行进程状态锁已损坏".to_string())
    }

    pub fn begin_update(&self, instance_id: &str) -> Result<Arc<AtomicBool>, String> {
        let mut update_cancels = self.lock_update_cancels()?;
        if update_cancels.contains_key(instance_id) {
            return Err("该实例已有安装/更新任务正在运行".to_string());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        update_cancels.insert(instance_id.to_string(), Arc::clone(&cancel));
        Ok(cancel)
    }

    pub fn finish_update(&self, instance_id: &str) {
        if let Ok(mut update_cancels) = self.lock_update_cancels() {
            update_cancels.remove(instance_id);
        }
    }

    pub fn cancel_update(&self, instance_id: &str) -> Result<bool, String> {
        let update_cancels = self.lock_update_cancels()?;
        let Some(cancel) = update_cancels.get(instance_id) else {
            return Ok(false);
        };
        cancel.store(true, Ordering::SeqCst);
        Ok(true)
    }

    pub fn is_update_running(&self, instance_id: &str) -> Result<bool, String> {
        Ok(self.lock_update_cancels()?.contains_key(instance_id))
    }

    pub fn clear_startup_auto_update_skip_flags(
        &self,
        instance_ids: &[String],
    ) -> Result<Vec<ServerInstance>, String> {
        let targets: HashSet<&str> = instance_ids.iter().map(String::as_str).collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated = Vec::new();
        {
            let mut data = self.lock()?;
            for instance in &mut data.instances {
                if !targets.contains(instance.id.as_str())
                    || !instance.skip_auto_update_on_start_once
                {
                    continue;
                }
                instance.skip_auto_update_on_start_once = false;
                updated.push(instance.clone());
            }
        }

        if !updated.is_empty() {
            self.persist()?;
        }

        Ok(updated)
    }

    fn lock_update_cancels(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Arc<AtomicBool>>>, String> {
        self.update_cancels
            .lock()
            .map_err(|_| "更新任务状态锁已损坏".to_string())
    }
}
