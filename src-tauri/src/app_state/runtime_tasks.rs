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

pub(crate) struct LifecycleOperationGuard {
    instance_id: String,
    operations: Arc<std::sync::Mutex<HashSet<String>>>,
}

pub(crate) struct ConfigurationOperationGuard {
    operation: Arc<std::sync::Mutex<bool>>,
}

impl Drop for ConfigurationOperationGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = self.operation.lock() {
            *active = false;
        }
    }
}

pub(crate) struct UpdateOperationGuard {
    instance_id: String,
    update_cancels: Arc<std::sync::Mutex<HashMap<String, Arc<AtomicBool>>>>,
    cancel: Arc<AtomicBool>,
}

impl UpdateOperationGuard {
    pub(crate) fn cancel_token(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }
}

impl Drop for UpdateOperationGuard {
    fn drop(&mut self) {
        if let Ok(mut update_cancels) = self.update_cancels.lock() {
            update_cancels.remove(&self.instance_id);
        }
    }
}

impl Drop for LifecycleOperationGuard {
    fn drop(&mut self) {
        if let Ok(mut operations) = self.operations.lock() {
            operations.remove(&self.instance_id);
        }
    }
}

impl AppRuntime {
    pub fn lock_processes(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Child>>, String> {
        self.processes
            .lock()
            .map_err(|_| "运行进程状态锁已损坏".to_string())
    }

    pub(crate) fn begin_lifecycle_operation(
        &self,
        instance_id: &str,
    ) -> Result<LifecycleOperationGuard, String> {
        self.begin_lifecycle_operation_inner(instance_id, false)
    }

    pub(crate) fn begin_configuration_operation(
        &self,
    ) -> Result<ConfigurationOperationGuard, String> {
        let mut active = self
            .configuration_operation
            .lock()
            .map_err(|_| "配置操作状态锁已损坏".to_string())?;
        if *active {
            return Err("已有配置保存或应用操作正在进行，请等待当前操作完成".to_string());
        }
        *active = true;
        Ok(ConfigurationOperationGuard {
            operation: Arc::clone(&self.configuration_operation),
        })
    }

    pub(crate) fn begin_stop_operation(
        &self,
        instance_id: &str,
    ) -> Result<LifecycleOperationGuard, String> {
        self.begin_lifecycle_operation_inner(instance_id, true)
    }

    fn begin_lifecycle_operation_inner(
        &self,
        instance_id: &str,
        allow_running_update: bool,
    ) -> Result<LifecycleOperationGuard, String> {
        let mut operations = self
            .lifecycle_operations
            .lock()
            .map_err(|_| "实例生命周期状态锁已损坏".to_string())?;
        if operations.contains(instance_id) {
            return Err("该实例正在执行启动、停止或重启操作，请等待当前操作完成".to_string());
        }
        if !allow_running_update && self.lock_update_cancels()?.contains_key(instance_id) {
            return Err("该实例正在安装/更新，请先等待任务完成或执行停止以取消更新".to_string());
        }
        operations.insert(instance_id.to_string());

        Ok(LifecycleOperationGuard {
            instance_id: instance_id.to_string(),
            operations: Arc::clone(&self.lifecycle_operations),
        })
    }

    pub(crate) fn begin_update(&self, instance_id: &str) -> Result<UpdateOperationGuard, String> {
        let operations = self
            .lifecycle_operations
            .lock()
            .map_err(|_| "实例生命周期状态锁已损坏".to_string())?;
        if operations.contains(instance_id) {
            return Err("该实例正在执行启动、停止或重启操作，当前不能安装/更新".to_string());
        }

        let mut update_cancels = self.lock_update_cancels()?;
        if update_cancels.contains_key(instance_id) {
            return Err("该实例已有安装/更新任务正在运行".to_string());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        update_cancels.insert(instance_id.to_string(), Arc::clone(&cancel));
        Ok(UpdateOperationGuard {
            instance_id: instance_id.to_string(),
            update_cancels: Arc::clone(&self.update_cancels),
            cancel,
        })
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
