use crate::{
    ark_config,
    models::{
        AddInstancePayload, GlobalSettings, LogLine, LogSource, ModItem, PortCheckResult,
        ServerInstance, ServerLogKind, ServerStatus,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    net::{TcpListener, UdpSocket},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};
use tokio::process::Child;

const STATE_FILE_NAME: &str = "manager-state.json";
const MAX_LOG_LINES: usize = 1_500;

#[derive(Clone)]
pub struct AppRuntime {
    data_dir: Arc<PathBuf>,
    data: Arc<Mutex<ManagerData>>,
    processes: Arc<Mutex<HashMap<String, Child>>>,
    update_cancels: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerData {
    pub settings: GlobalSettings,
    pub instances: Vec<ServerInstance>,
    pub configs: HashMap<String, Value>,
    pub mods: HashMap<String, Vec<ModItem>>,
    pub logs: Vec<LogLine>,
}

impl Default for ManagerData {
    fn default() -> Self {
        Self {
            settings: GlobalSettings::default(),
            instances: Vec::new(),
            configs: HashMap::new(),
            mods: HashMap::new(),
            logs: Vec::new(),
        }
    }
}

impl AppRuntime {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("无法定位应用数据目录：{error}"))?;
        fs::create_dir_all(&data_dir)
            .map_err(|error| format!("无法创建应用数据目录 {}：{error}", data_dir.display()))?;

        let data = read_data(&data_dir)?;
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

    pub fn save_settings(&self, settings: GlobalSettings) -> Result<GlobalSettings, String> {
        {
            let mut data = self.lock()?;
            data.settings = settings.clone();
        }
        self.persist()?;
        Ok(settings)
    }

    pub fn list_instances(&self) -> Result<Vec<ServerInstance>, String> {
        Ok(self.lock()?.instances.clone())
    }

    pub fn check_instance_port(
        &self,
        port: u16,
        port_kind: &str,
    ) -> Result<PortCheckResult, String> {
        validate_port_kind(port_kind)?;
        if port < 1024 {
            return Ok(PortCheckResult {
                port,
                available: false,
                exists: false,
                suggested_port: None,
                reason: Some("端口必须在 1024-65535 范围内".to_string()),
            });
        }

        let data = self.lock()?;
        let exists = instance_uses_port(&data.instances, port);
        let suggested_port = if exists {
            suggest_next_instance_port(&data.instances, port_kind)?
        } else {
            None
        };

        if exists {
            return Ok(PortCheckResult {
                port,
                available: false,
                exists,
                suggested_port,
                reason: Some(format!("端口 {port} 已被其他实例占用")),
            });
        }

        let reason = system_port_unavailable_reason(port);
        Ok(PortCheckResult {
            port,
            available: reason.is_none(),
            exists,
            suggested_port,
            reason,
        })
    }

    pub fn get_instance(&self, instance_id: &str) -> Result<ServerInstance, String> {
        self.lock()?
            .instances
            .iter()
            .find(|instance| instance.id == instance_id)
            .cloned()
            .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))
    }

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
            instance.status = status;
            instance.last_error = last_error;
            instance.clone()
        };
        self.persist()?;
        Ok(updated)
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
        self.add_log(
            &removed.name,
            "warn",
            &format!("已删除实例记录，实例文件仍保留在：{}", removed.install_path),
        )?;
        Ok(removed)
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
            }
            instance.clone()
        };
        self.persist()?;
        Ok(updated)
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
            version_state: "未安装".to_string(),
            last_error: None,
        };

        let config = normalize_required_rcon_config(config_from_payload(&payload, &instance))?;
        let mods = sanitize_imported_mods(&payload);
        let applied_config = ark_config::apply_instance_config(&instance, &config, &mods)?;

        {
            let mut data = self.lock()?;
            data.configs.insert(id.clone(), config);
            data.mods.insert(id.clone(), mods);
            data.instances.push(instance.clone());
        }
        self.add_log(
            &instance.name,
            "success",
            &format!(
                "已创建服务器实例，初始 ARK 配置已写入：{}、{}、{}",
                applied_config.game_user_settings_path.to_string_lossy(),
                applied_config.game_ini_path.to_string_lossy(),
                applied_config.engine_ini_path.to_string_lossy()
            ),
        )?;
        self.persist()?;
        Ok(instance)
    }

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
            let instance_index = data
                .instances
                .iter()
                .position(|item| item.id == instance_id)
                .ok_or_else(|| format!("未找到服务器实例：{instance_id}"))?;
            let current = data.instances[instance_index].clone();
            let updated = instance_with_config_metadata(&data.instances, &current, &config)?;
            data.instances[instance_index] = updated.clone();
            data.configs.insert(instance_id.to_string(), config);
            data.mods.insert(instance_id.to_string(), mods);
            updated
        };
        self.persist()?;
        Ok(updated)
    }

    pub fn add_log(&self, instance: &str, level: &str, message: &str) -> Result<LogLine, String> {
        self.add_log_with_source(LogSource::Application, None, instance, level, message)
    }

    pub fn add_server_log_with_kind(
        &self,
        instance: &str,
        level: &str,
        message: &str,
        server_log_kind: ServerLogKind,
    ) -> Result<LogLine, String> {
        self.add_log_with_source(
            LogSource::Server,
            Some(server_log_kind),
            instance,
            level,
            message,
        )
    }

    fn add_log_with_source(
        &self,
        source: LogSource,
        server_log_kind: Option<ServerLogKind>,
        instance: &str,
        level: &str,
        message: &str,
    ) -> Result<LogLine, String> {
        let line = {
            let mut data = self.lock()?;
            let timestamp_id = now_millis();
            let id = data
                .logs
                .last()
                .map(|line| line.id.saturating_add(1))
                .filter(|next_id| *next_id > timestamp_id)
                .unwrap_or(timestamp_id);
            let line = LogLine {
                id,
                time: current_time_text(),
                source,
                server_log_kind,
                instance: instance.to_string(),
                level: level.to_string(),
                message: message.to_string(),
            };
            data.logs.push(line.clone());
            if data.logs.len() > MAX_LOG_LINES {
                let overflow = data.logs.len() - MAX_LOG_LINES;
                data.logs.drain(0..overflow);
            }
            line
        };
        self.persist()?;
        Ok(line)
    }

    pub fn query_logs(&self, limit: Option<usize>) -> Result<Vec<LogLine>, String> {
        let data = self.lock()?;
        let limit = limit.unwrap_or(MAX_LOG_LINES);
        let start = data.logs.len().saturating_sub(limit);
        Ok(data.logs[start..].to_vec())
    }

    pub fn clear_logs(&self) -> Result<(), String> {
        {
            let mut data = self.lock()?;
            data.logs.clear();
        }
        self.persist()
    }

    pub fn clear_logs_by_scope(
        &self,
        source: LogSource,
        instance: Option<&str>,
        server_log_kind: Option<ServerLogKind>,
    ) -> Result<(), String> {
        {
            let mut data = self.lock()?;
            data.logs.retain(|line| {
                let matches_instance = instance.map_or(true, |target| line.instance == target);
                let matches_kind = server_log_kind.as_ref().map_or(true, |target| {
                    line.server_log_kind
                        .as_ref()
                        .unwrap_or(&ServerLogKind::Console)
                        == target
                });
                !(line.source == source && matches_instance && matches_kind)
            });
        }
        self.persist()
    }

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

    fn lock_update_cancels(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Arc<AtomicBool>>>, String> {
        self.update_cancels
            .lock()
            .map_err(|_| "更新任务状态锁已损坏".to_string())
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

fn read_data(data_dir: &Path) -> Result<ManagerData, String> {
    let path = data_dir.join(STATE_FILE_NAME);
    if !path.exists() {
        return Ok(ManagerData::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("无法读取管理器状态文件 {}：{error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("管理器状态文件格式无效 {}：{error}", path.display()))
}

fn write_data(data_dir: &Path, data: &ManagerData) -> Result<(), String> {
    fs::create_dir_all(data_dir)
        .map_err(|error| format!("无法创建应用数据目录 {}：{error}", data_dir.display()))?;
    let path = data_dir.join(STATE_FILE_NAME);
    let temp_path = data_dir.join(format!("{STATE_FILE_NAME}.tmp"));
    let content = serde_json::to_string_pretty(data)
        .map_err(|error| format!("无法序列化管理器状态：{error}"))?;
    fs::write(&temp_path, content)
        .map_err(|error| format!("无法写入临时状态文件 {}：{error}", temp_path.display()))?;
    fs::rename(&temp_path, &path)
        .map_err(|error| format!("无法替换状态文件 {}：{error}", path.display()))
}

fn validate_instance_payload(payload: &AddInstancePayload) -> Result<(), String> {
    if payload.name.trim().is_empty() {
        return Err("实例名称不能为空".to_string());
    }
    if payload.map_code.trim().is_empty() {
        return Err("地图代码不能为空".to_string());
    }
    if payload.install_path.trim().is_empty() {
        return Err("实例目录不能为空".to_string());
    }
    if payload.admin_password.trim().is_empty() {
        return Err("管理员密码不能为空".to_string());
    }
    if [payload.game_port, payload.query_port, payload.rcon_port]
        .iter()
        .any(|port| *port < 1024)
    {
        return Err("端口必须在 1024-65535 范围内".to_string());
    }
    if payload.game_port == payload.query_port
        || payload.game_port == payload.rcon_port
        || payload.query_port == payload.rcon_port
    {
        return Err("游戏端口、查询端口和 RCON 端口不能重复".to_string());
    }
    Ok(())
}

pub fn normalize_required_rcon_config(mut config: Value) -> Result<Value, String> {
    let Some(config_map) = config.as_object_mut() else {
        return Err("实例配置格式无效".to_string());
    };
    let admin_password = config_map
        .get("adminPassword")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if admin_password.trim().is_empty() {
        return Err("管理员密码不能为空，RCON 必须启用并设置密码".to_string());
    }
    config_map.insert("rconEnabled".to_string(), json!(true));
    Ok(config)
}

fn instance_with_config_metadata(
    instances: &[ServerInstance],
    current: &ServerInstance,
    config: &Value,
) -> Result<ServerInstance, String> {
    let mut updated = current.clone();

    if let Some(session_name) = trimmed_config_string(config, "sessionName") {
        if session_name.is_empty() {
            return Err("服务器名称不能为空".to_string());
        }
        updated.name = session_name;
    }
    if let Some(game_port) = config_u16(config, "gamePort") {
        updated.game_port = game_port;
    }
    if let Some(query_port) = config_u16(config, "queryPort") {
        updated.query_port = query_port;
    }
    if let Some(rcon_port) = config_u16(config, "rconPort") {
        updated.rcon_port = rcon_port;
    }
    if let Some(max_players) = config_u32(config, "maxPlayers") {
        updated.max_players = max_players;
    }
    if let Some(cluster_id) = trimmed_config_string(config, "clusterId") {
        updated.cluster_id = cluster_id;
    }
    if let Some(pve) = config.get("pve").and_then(Value::as_bool) {
        updated.mode = if pve { "PvE" } else { "PvP" }.to_string();
    }

    validate_updated_instance_metadata(instances, current, &updated)?;
    Ok(updated)
}

fn validate_updated_instance_metadata(
    instances: &[ServerInstance],
    current: &ServerInstance,
    updated: &ServerInstance,
) -> Result<(), String> {
    if updated.name.trim().is_empty() {
        return Err("服务器名称不能为空".to_string());
    }
    if instances
        .iter()
        .any(|item| item.id != current.id && item.name.eq_ignore_ascii_case(updated.name.trim()))
    {
        return Err(format!("实例名称已存在：{}", updated.name.trim()));
    }
    if [updated.game_port, updated.query_port, updated.rcon_port]
        .iter()
        .any(|port| *port < 1024)
    {
        return Err("端口必须在 1024-65535 范围内".to_string());
    }
    if updated.game_port == updated.query_port
        || updated.game_port == updated.rcon_port
        || updated.query_port == updated.rcon_port
    {
        return Err("游戏端口、查询端口和 RCON 端口不能重复".to_string());
    }

    let other_instances: Vec<ServerInstance> = instances
        .iter()
        .filter(|item| item.id != current.id)
        .cloned()
        .collect();
    for (port, label) in [
        (updated.game_port, "游戏端口"),
        (updated.query_port, "查询端口"),
        (updated.rcon_port, "RCON 端口"),
    ] {
        if instance_uses_port(&other_instances, port) {
            return Err(format!("{label} {port} 已被其他实例占用"));
        }
    }
    for (next_port, previous_port, label) in [
        (updated.game_port, current.game_port, "游戏端口"),
        (updated.query_port, current.query_port, "查询端口"),
        (updated.rcon_port, current.rcon_port, "RCON 端口"),
    ] {
        if next_port != previous_port {
            if let Some(reason) = system_port_unavailable_reason(next_port) {
                return Err(format!("{label} {next_port} 不可用：{reason}"));
            }
        }
    }
    Ok(())
}

fn trimmed_config_string(config: &Value, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .map(ToString::to_string)
}

fn config_u16(config: &Value, key: &str) -> Option<u16> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
}

fn config_u32(config: &Value, key: &str) -> Option<u32> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn ensure_port_available(
    instances: &[ServerInstance],
    port: u16,
    label: &str,
) -> Result<(), String> {
    if instance_uses_port(instances, port) {
        return Err(format!("{label} {port} 已被其他实例占用"));
    }
    if let Some(reason) = system_port_unavailable_reason(port) {
        return Err(format!("{label} {port} 不可用：{reason}"));
    }
    Ok(())
}

fn instance_uses_port(instances: &[ServerInstance], port: u16) -> bool {
    instances
        .iter()
        .any(|item| item.game_port == port || item.query_port == port || item.rcon_port == port)
}

fn validate_port_kind(port_kind: &str) -> Result<(), String> {
    match port_kind {
        "gamePort" | "queryPort" | "rconPort" => Ok(()),
        _ => Err(format!("未知端口类型：{port_kind}")),
    }
}

fn port_for_kind(instance: &ServerInstance, port_kind: &str) -> Result<u16, String> {
    match port_kind {
        "gamePort" => Ok(instance.game_port),
        "queryPort" => Ok(instance.query_port),
        "rconPort" => Ok(instance.rcon_port),
        _ => Err(format!("未知端口类型：{port_kind}")),
    }
}

fn suggest_next_instance_port(
    instances: &[ServerInstance],
    port_kind: &str,
) -> Result<Option<u16>, String> {
    let Some(last_instance) = instances.last() else {
        return Ok(None);
    };
    let mut candidate = u32::from(port_for_kind(last_instance, port_kind)?) + 10;
    while candidate <= u32::from(u16::MAX) {
        let port = candidate as u16;
        if !instance_uses_port(instances, port) {
            return Ok(Some(port));
        }
        candidate += 10;
    }
    Ok(None)
}

fn system_port_unavailable_reason(port: u16) -> Option<String> {
    let tcp_listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => listener,
        Err(error) => return Some(format!("TCP 绑定失败：{error}")),
    };
    drop(tcp_listener);

    let udp_socket = match UdpSocket::bind(("0.0.0.0", port)) {
        Ok(socket) => socket,
        Err(error) => return Some(format!("UDP 绑定失败：{error}")),
    };
    drop(udp_socket);

    None
}

fn normalize_path_text(path: &str) -> PathBuf {
    PathBuf::from(clean_windows_path_text(path.trim().trim_matches('"')))
}

fn clean_windows_path_text(value: &str) -> String {
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        return format!("\\\\{rest}");
    }
    if let Some(rest) = value.strip_prefix("\\\\?\\") {
        return rest.to_string();
    }
    value.to_string()
}

fn default_config_from_payload(payload: &AddInstancePayload, instance: &ServerInstance) -> Value {
    let mut map = Map::new();
    map.insert("sessionName".to_string(), json!(instance.name));
    map.insert("serverPassword".to_string(), json!(payload.server_password));
    map.insert("spectatorPassword".to_string(), json!(""));
    map.insert("adminPassword".to_string(), json!(payload.admin_password));
    map.insert("gamePort".to_string(), json!(payload.game_port));
    map.insert("queryPort".to_string(), json!(payload.query_port));
    map.insert("rconEnabled".to_string(), json!(true));
    map.insert("rconPort".to_string(), json!(payload.rcon_port));
    map.insert("visibility".to_string(), json!("public"));
    map.insert("clusterId".to_string(), json!(payload.cluster_id));
    map.insert("crossTransfer".to_string(), json!(true));
    map.insert("maxPlayers".to_string(), json!(payload.max_players));
    map.insert("pve".to_string(), json!(payload.mode == "PvE"));
    map.insert("hardcore".to_string(), json!(false));
    map.insert("disableFriendlyFire".to_string(), json!(false));
    map.insert("enablePvPGamma".to_string(), json!(true));
    map.insert("allowHitMarkers".to_string(), json!(true));
    map.insert("difficulty".to_string(), json!(5));
    map.insert("xpMultiplier".to_string(), json!(1.5));
    map.insert("tamingSpeed".to_string(), json!(3));
    map.insert("harvestAmount".to_string(), json!(2));
    map.insert("harvestHealthMultiplier".to_string(), json!(1));
    map.insert("playerDamageMultiplier".to_string(), json!(1));
    map.insert("playerResistanceMultiplier".to_string(), json!(1));
    map.insert("dinoDamageMultiplier".to_string(), json!(1));
    map.insert("dinoResistanceMultiplier".to_string(), json!(1));
    map.insert("tamedDinoDamageMultiplier".to_string(), json!(1));
    map.insert("tamedDinoResistanceMultiplier".to_string(), json!(1));
    map.insert("playerFoodDrainMultiplier".to_string(), json!(1));
    map.insert("playerWaterDrainMultiplier".to_string(), json!(1));
    map.insert("playerStaminaDrainMultiplier".to_string(), json!(1));
    map.insert("dinoFoodDrainMultiplier".to_string(), json!(1));
    map.insert("dinoStaminaDrainMultiplier".to_string(), json!(1));
    map.insert("thirdPerson".to_string(), json!(true));
    map.insert("crosshair".to_string(), json!(true));
    map.insert("showMapPlayer".to_string(), json!(true));
    map.insert("flyerCarry".to_string(), json!(true));
    map.insert("autoRestart".to_string(), json!(true));
    map.insert("restartTime".to_string(), json!("04:00"));
    map.insert("saveInterval".to_string(), json!(15));
    map.insert("backupRetention".to_string(), json!(7));
    map.insert("autoUpdateServer".to_string(), json!(payload.auto_install));
    map.insert("autoUpdateMods".to_string(), json!(true));
    map.insert("restartOnCrash".to_string(), json!(true));
    map.insert("saveOnStop".to_string(), json!(true));
    map.insert("dayCycleSpeed".to_string(), json!(1));
    map.insert("dayTimeSpeed".to_string(), json!(1));
    map.insert("nightTimeSpeed".to_string(), json!(1.5));
    map.insert("resourceRespawn".to_string(), json!(0.7));
    map.insert("resourceNoReplenishRadiusPlayers".to_string(), json!(1));
    map.insert("resourceNoReplenishRadiusStructures".to_string(), json!(1));
    map.insert("dinoCount".to_string(), json!(1));
    map.insert("maxTamedDinos".to_string(), json!(5000));
    map.insert("destroyWildDinos".to_string(), json!(false));
    map.insert("cropGrowthSpeedMultiplier".to_string(), json!(1));
    map.insert("cropDecaySpeedMultiplier".to_string(), json!(1));
    map.insert("supplyCrateLootQualityMultiplier".to_string(), json!(1));
    map.insert("fishingLootQualityMultiplier".to_string(), json!(1));
    map.insert("fuelConsumptionIntervalMultiplier".to_string(), json!(1));
    map.insert("itemStackSizeMultiplier".to_string(), json!(1));
    map.insert("globalSpoilingTimeMultiplier".to_string(), json!(1));
    map.insert(
        "globalItemDecompositionTimeMultiplier".to_string(),
        json!(1),
    );
    map.insert(
        "globalCorpseDecompositionTimeMultiplier".to_string(),
        json!(1),
    );
    map.insert("matingInterval".to_string(), json!(0.25));
    map.insert("matingSpeedMultiplier".to_string(), json!(1));
    map.insert("eggHatchSpeed".to_string(), json!(10));
    map.insert("babyMatureSpeed".to_string(), json!(20));
    map.insert("cuddleInterval".to_string(), json!(0.1));
    map.insert("babyFoodConsumption".to_string(), json!(0.5));
    map.insert("layEggIntervalMultiplier".to_string(), json!(1));
    map.insert("babyCuddleGracePeriodMultiplier".to_string(), json!(1));
    map.insert(
        "babyCuddleLoseImprintQualitySpeedMultiplier".to_string(),
        json!(1),
    );
    map.insert("babyImprintingStatScaleMultiplier".to_string(), json!(1));
    map.insert("babyImprintAmountMultiplier".to_string(), json!(1));
    map.insert("allowAnyoneBabyImprintCuddle".to_string(), json!(false));
    map.insert("structureLimit".to_string(), json!(10500));
    map.insert("platformStructureMultiplier".to_string(), json!(1.5));
    map.insert("disablePlacementCollision".to_string(), json!(true));
    map.insert("maxTribeSize".to_string(), json!(8));
    map.insert("tribeAlliances".to_string(), json!(true));
    map.insert("pveStructureDecay".to_string(), json!(false));
    map.insert("allowCaveBuildingPvE".to_string(), json!(false));
    map.insert("allowCaveBuildingPvP".to_string(), json!(true));
    map.insert("structureDamageRepairCooldown".to_string(), json!(180));
    map.insert("structurePickupTimeAfterPlacement".to_string(), json!(30));
    map.insert("structurePickupHoldDuration".to_string(), json!(0.5));
    map.insert("autoDestroyOldStructuresMultiplier".to_string(), json!(1));
    map.insert("fastDecayUnsnappedCoreStructures".to_string(), json!(false));
    map.insert("limitGeneratorsNum".to_string(), json!(3));
    map.insert("limitGeneratorsRange".to_string(), json!(15000));
    map.insert("allowCryoFridgeOnSaddle".to_string(), json!(false));
    map.insert("disableCryopodEnemyCheck".to_string(), json!(false));
    map.insert("disableCryopodFridgeRequirement".to_string(), json!(false));
    map.insert("disableCryopodCooldown".to_string(), json!(false));
    map.insert("allowFlyerSpeedLeveling".to_string(), json!(false));
    map.insert("forceAllowCaveFlyers".to_string(), json!(false));
    map.insert("allowFlyingStaminaRecovery".to_string(), json!(false));
    map.insert("raidDinoFoodDrainMultiplier".to_string(), json!(1));
    map.insert("whitelist".to_string(), json!(false));
    map.insert("exclusiveJoin".to_string(), json!(false));
    map.insert("preventDownloadItems".to_string(), json!(false));
    map.insert("preventDownloadDinos".to_string(), json!(false));
    map.insert("preventDownloadSurvivors".to_string(), json!(false));
    map.insert("preventUploadItems".to_string(), json!(false));
    map.insert("preventUploadDinos".to_string(), json!(false));
    map.insert("preventUploadSurvivors".to_string(), json!(false));
    map.insert("noTributeDownloads".to_string(), json!(false));
    map.insert("minimumDinoReuploadInterval".to_string(), json!(0));
    map.insert("tributeCharacterExpirationSeconds".to_string(), json!(0));
    map.insert("tributeDinoExpirationSeconds".to_string(), json!(0));
    map.insert("tributeItemExpirationSeconds".to_string(), json!(0));
    map.insert("useAllCores".to_string(), json!(true));
    map.insert("noBattlEye".to_string(), json!(true));
    map.insert("noTransferFromFiltering".to_string(), json!(true));
    map.insert("enableIdlePlayerKick".to_string(), json!(false));
    map.insert("kickIdlePlayersPeriod".to_string(), json!(3600));
    map.insert("enableDiseases".to_string(), json!(true));
    map.insert("nonPermanentDiseases".to_string(), json!(false));
    map.insert("tribeNameChangeCooldown".to_string(), json!(15));
    map.insert("maxAlliancesPerTribe".to_string(), json!(0));
    map.insert("maxTribesPerAlliance".to_string(), json!(0));
    map.insert("processPriority".to_string(), json!("aboveNormal"));
    map.insert("cpuAffinity".to_string(), json!("自动"));
    map.insert("memoryWarningGb".to_string(), json!(24));
    map.insert("networkTickRate".to_string(), json!(30));
    map.insert("maxClientRate".to_string(), json!(100000));
    map.insert("rconBufferSize".to_string(), json!(6000));
    map.insert("compressBackups".to_string(), json!(true));
    map.insert("snapshotBeforeRestart".to_string(), json!(true));
    map.insert("preventHibernation".to_string(), json!(false));
    map.insert("stasisKeepControllers".to_string(), json!(false));
    map.insert("useStructureStasisGrid".to_string(), json!(true));
    map.insert(
        "alwaysTickDedicatedSkeletalMeshes".to_string(),
        json!(false),
    );
    map.insert("gbUsageToForceRestart".to_string(), json!(35));
    map.insert("serverPlatform".to_string(), json!("ALL"));
    map.insert("activeEvent".to_string(), json!(""));
    map.insert("useDynamicConfig".to_string(), json!(false));
    map.insert("customDynamicConfigUrl".to_string(), json!(""));
    map.insert(
        "clusterDirOverride".to_string(),
        json!("ShooterGame/Saved/clusters"),
    );
    map.insert("customLaunchArgs".to_string(), json!("-culture=zh"));
    map.insert("serverGameLog".to_string(), json!(true));
    map.insert("serverGameLogIncludeTribe".to_string(), json!(true));
    map.insert("adminLogging".to_string(), json!(true));
    map.insert("chatLogging".to_string(), json!(true));
    map.insert("logTimestamp".to_string(), json!(true));
    map.insert("logLevel".to_string(), json!("normal"));
    map.insert("rotateSizeMb".to_string(), json!(100));
    map.insert("logRetentionDays".to_string(), json!(14));
    map.insert("logPath".to_string(), json!("ShooterGame/Saved/Logs"));
    map.insert(
        "crossArkAllowForeignDinoDownloads".to_string(),
        json!(false),
    );
    map.insert("limitBunkersPerTribe".to_string(), json!(true));
    map.insert("limitBunkersPerTribeNum".to_string(), json!(3));
    map.insert("allowBunkersInPreventionZones".to_string(), json!(false));
    map.insert("allowRidingDinosInsideBunkers".to_string(), json!(true));
    map.insert("allowBunkerModulesAboveGround".to_string(), json!(false));
    map.insert("allowDinoAIInsideBunkers".to_string(), json!(true));
    map.insert(
        "allowBunkerModulesInPreventionZones".to_string(),
        json!(false),
    );
    map.insert("minDistanceBetweenBunkers".to_string(), json!(3000));
    map.insert("enemyAccessBunkerHPThreshold".to_string(), json!(0.25));
    map.insert(
        "bunkerUnderHPThresholdDmgMultiplier".to_string(),
        json!(0.05),
    );
    Value::Object(map)
}

fn config_from_payload(payload: &AddInstancePayload, instance: &ServerInstance) -> Value {
    let mut config = match default_config_from_payload(payload, instance) {
        Value::Object(map) => map,
        _ => Map::new(),
    };

    if let Some(Value::Object(imported_config)) = &payload.imported_config {
        for (key, value) in imported_config {
            config.insert(key.clone(), value.clone());
        }
    }

    apply_payload_config_overrides(&mut config, payload, instance);
    Value::Object(config)
}

fn apply_payload_config_overrides(
    config: &mut Map<String, Value>,
    payload: &AddInstancePayload,
    instance: &ServerInstance,
) {
    config.insert("sessionName".to_string(), json!(instance.name));
    config.insert("serverPassword".to_string(), json!(payload.server_password));
    config.insert("adminPassword".to_string(), json!(payload.admin_password));
    config.insert("gamePort".to_string(), json!(payload.game_port));
    config.insert("queryPort".to_string(), json!(payload.query_port));
    config.insert("rconEnabled".to_string(), json!(true));
    config.insert("rconPort".to_string(), json!(payload.rcon_port));
    config.insert("clusterId".to_string(), json!(payload.cluster_id));
    config.insert("maxPlayers".to_string(), json!(payload.max_players));
    config.insert("pve".to_string(), json!(payload.mode == "PvE"));
    config.insert("autoUpdateServer".to_string(), json!(payload.auto_install));
    config.insert(
        "visibility".to_string(),
        json!(if payload.server_password.trim().is_empty() {
            "public"
        } else {
            "private"
        }),
    );
}

fn sanitize_imported_mods(payload: &AddInstancePayload) -> Vec<ModItem> {
    let mut seen = HashSet::new();
    payload
        .imported_mods
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|item| {
            let id = item.id.trim();
            if id.is_empty() || !id.chars().all(|ch| ch.is_ascii_digit()) {
                return None;
            }
            if !seen.insert(id.to_string()) {
                return None;
            }
            Some(ModItem {
                id: id.to_string(),
                name: if item.name.trim().is_empty() {
                    format!("MOD {id}")
                } else {
                    item.name.trim().to_string()
                },
                version: if item.version.trim().is_empty() {
                    "配置导入".to_string()
                } else {
                    item.version.trim().to_string()
                },
                size: if item.size.trim().is_empty() {
                    "未知大小".to_string()
                } else {
                    item.size.trim().to_string()
                },
                enabled: item.enabled,
                update_available: item.update_available.or(Some(false)),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_runtime(data_dir: &Path) -> AppRuntime {
        AppRuntime {
            data_dir: Arc::new(data_dir.to_path_buf()),
            data: Arc::new(Mutex::new(ManagerData::default())),
            processes: Arc::new(Mutex::new(HashMap::new())),
            update_cancels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn available_test_ports() -> (u16, u16, u16) {
        let mut ports = Vec::new();
        while ports.len() < 3 {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("获取可用测试端口");
            let port = listener.local_addr().expect("读取测试端口").port();
            drop(listener);

            if !ports.contains(&port) && system_port_unavailable_reason(port).is_none() {
                ports.push(port);
            }
        }
        (ports[0], ports[1], ports[2])
    }

    fn test_payload(install_path: &Path) -> AddInstancePayload {
        let (game_port, query_port, rcon_port) = available_test_ports();
        AddInstancePayload {
            id: None,
            name: "新增实例".to_string(),
            map: "The Island".to_string(),
            map_code: "TheIsland_WP".to_string(),
            mode: "PvE".to_string(),
            status: None,
            game_port,
            query_port,
            players: None,
            max_players: 42,
            install_path: install_path.to_string_lossy().into_owned(),
            rcon_port,
            cluster_id: "Cluster-A".to_string(),
            server_password: "join-pass".to_string(),
            admin_password: "admin-pass".to_string(),
            auto_install: false,
            description: "用于验证新增即写入配置".to_string(),
            imported_config: None,
            imported_mods: None,
        }
    }

    #[test]
    fn 新增实例会立即写入默认_ark_ini() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let install_path = temp.path().join("未安装实例");
        let runtime = test_runtime(temp.path());
        let payload = test_payload(&install_path);
        let game_port = payload.game_port;
        let query_port = payload.query_port;
        let rcon_port = payload.rcon_port;

        let instance = runtime.create_instance(payload).expect("创建实例");

        let config_dir = install_path
            .join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsServer");
        let game_user_settings_path = config_dir.join("GameUserSettings.ini");
        let game_ini_path = config_dir.join("Game.ini");
        let engine_ini_path = config_dir.join("Engine.ini");
        assert!(game_user_settings_path.is_file());
        assert!(game_ini_path.is_file());
        assert!(engine_ini_path.is_file());

        let game_user_settings =
            fs::read_to_string(game_user_settings_path).expect("读取 GameUserSettings.ini");
        assert!(game_user_settings.contains("SessionName=新增实例"));
        assert!(game_user_settings.contains("ServerPassword=join-pass"));
        assert!(game_user_settings.contains("ServerAdminPassword=admin-pass"));
        assert!(game_user_settings.contains(&format!("Port={game_port}")));
        assert!(game_user_settings.contains(&format!("QueryPort={query_port}")));
        assert!(game_user_settings.contains(&format!("RCONPort={rcon_port}")));
        let server_settings_section = game_user_settings
            .split("[SessionSettings]")
            .next()
            .unwrap_or_default();
        assert!(!server_settings_section.contains("MaxPlayers="));
        assert!(!game_user_settings.contains("[/Script/Engine.GameSession]"));
        assert!(!game_user_settings.contains("MaxPlayers=42"));
        assert!(game_user_settings.contains("XPMultiplier=1.5"));
        assert!(game_user_settings.contains("TamingSpeedMultiplier=3"));
        assert!(game_user_settings.contains("NightTimeSpeedScale=1.5"));
        assert!(game_user_settings.contains("TheMaxStructuresInRange=10500"));
        assert!(!game_user_settings.contains("LimitBunkersPerTribe="));

        let game_ini = fs::read_to_string(game_ini_path).expect("读取 Game.ini");
        assert!(game_ini.contains("MatingIntervalMultiplier=0.25"));
        assert!(game_ini.contains("BabyMatureSpeedMultiplier=20"));
        assert!(game_ini.contains("bDisableStructurePlacementCollision=True"));
        assert!(!game_ini.contains("TheMaxStructuresInRange="));
        assert!(!game_ini.contains("LimitBunkersPerTribe="));

        let engine_ini = fs::read_to_string(engine_ini_path).expect("读取 Engine.ini");
        assert!(engine_ini.contains("[/Script/OnlineSubsystemUtils.IpNetDriver]"));
        assert!(engine_ini.contains("NetServerMaxTickRate=30"));
        assert!(engine_ini.contains("MaxClientRate=100000"));

        let stored_config = runtime.get_config(&instance.id).expect("读取实例配置");
        assert_eq!(
            stored_config.get("xpMultiplier").and_then(Value::as_f64),
            Some(1.5)
        );
        assert_eq!(
            stored_config
                .get("disablePlacementCollision")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn 按来源和实例清除日志不会影响其他面板() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let runtime = test_runtime(temp.path());

        runtime
            .add_log("管理器", "info", "应用日志")
            .expect("写入应用日志");
        runtime
            .add_server_log_with_kind("孤岛", "info", "孤岛日志", ServerLogKind::Console)
            .expect("写入孤岛日志");
        runtime
            .add_server_log_with_kind("孤岛", "warn", "孤岛文件日志", ServerLogKind::File)
            .expect("写入孤岛文件日志");
        runtime
            .add_server_log_with_kind("仙境", "warn", "仙境窗口日志", ServerLogKind::Console)
            .expect("写入仙境窗口日志");

        runtime
            .clear_logs_by_scope(
                LogSource::Server,
                Some("孤岛"),
                Some(ServerLogKind::Console),
            )
            .expect("清除孤岛窗口日志");

        let logs = runtime.query_logs(None).expect("查询日志");
        assert_eq!(logs.len(), 3);
        assert!(
            logs.iter()
                .any(|line| line.source == LogSource::Application && line.instance == "管理器")
        );
        assert!(logs.iter().any(|line| line.source == LogSource::Server
            && line.instance == "孤岛"
            && line.server_log_kind.as_ref() == Some(&ServerLogKind::File)));
        assert!(logs.iter().any(|line| line.source == LogSource::Server
            && line.instance == "仙境"
            && line.server_log_kind.as_ref() == Some(&ServerLogKind::Console)));
        assert!(!logs.iter().any(|line| {
            line.source == LogSource::Server
                && line.instance == "孤岛"
                && line
                    .server_log_kind
                    .as_ref()
                    .unwrap_or(&ServerLogKind::Console)
                    == &ServerLogKind::Console
        }));
    }

    #[test]
    fn 实例配置会强制启用_rcon() {
        let config = normalize_required_rcon_config(json!({
            "adminPassword": "ark-admin",
            "rconEnabled": false
        }))
        .expect("规范化配置");

        assert_eq!(
            config.get("rconEnabled").and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn 实例配置拒绝空管理员密码() {
        let error = normalize_required_rcon_config(json!({
            "adminPassword": "   ",
            "rconEnabled": true
        }))
        .expect_err("应拒绝空管理员密码");

        assert!(error.contains("管理员密码不能为空"));
    }

    #[test]
    fn 保存配置会同步实例列表元数据() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let install_path = temp.path().join("metadata-sync-instance");
        let runtime = test_runtime(temp.path());
        let instance = runtime
            .create_instance(test_payload(&install_path))
            .expect("创建实例");
        let mut config = runtime.get_config(&instance.id).expect("读取实例配置");
        let config_map = config.as_object_mut().expect("配置是对象");
        config_map.insert("sessionName".to_string(), json!("MG-TEST"));
        config_map.insert("maxPlayers".to_string(), json!(64));
        config_map.insert("pve".to_string(), json!(false));
        config_map.insert("clusterId".to_string(), json!("Cluster-B"));

        let updated = runtime
            .save_config_and_mods(&instance.id, config, Vec::new())
            .expect("保存配置");
        let stored = runtime.get_instance(&instance.id).expect("读取实例");

        assert_eq!(updated.name, "MG-TEST");
        assert_eq!(stored.name, "MG-TEST");
        assert_eq!(stored.max_players, 64);
        assert_eq!(stored.mode, "PvP");
        assert_eq!(stored.cluster_id, "Cluster-B");
    }
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn current_time_text() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        % 86_400;
    let hour = seconds / 3_600;
    let minute = seconds % 3_600 / 60;
    let second = seconds % 60;
    format!("{hour:02}:{minute:02}:{second:02}")
}

pub fn current_timestamp_text() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{seconds}")
}
