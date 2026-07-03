use crate::models::{
    AddInstancePayload, GlobalSettings, LogLine, LogSource, ModItem, PortCheckResult,
    ServerInstance, ServerLogKind, ServerStatus,
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

        {
            let mut data = self.lock()?;
            data.configs
                .insert(id.clone(), config_from_payload(&payload, &instance));
            data.mods
                .insert(id.clone(), sanitize_imported_mods(&payload));
            data.instances.push(instance.clone());
        }
        self.add_log(&instance.name, "success", "已创建服务器实例")?;
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
    ) -> Result<(), String> {
        self.get_instance(instance_id)?;
        let config = normalize_required_rcon_config(config)?;
        {
            let mut data = self.lock()?;
            data.configs.insert(instance_id.to_string(), config);
            data.mods.insert(instance_id.to_string(), mods);
        }
        self.persist()
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
    map.insert("autoRestart".to_string(), json!(true));
    map.insert("restartTime".to_string(), json!("04:00"));
    map.insert("saveInterval".to_string(), json!(15));
    map.insert("backupRetention".to_string(), json!(7));
    map.insert("autoUpdateServer".to_string(), json!(payload.auto_install));
    map.insert("autoUpdateMods".to_string(), json!(true));
    map.insert("restartOnCrash".to_string(), json!(true));
    map.insert("saveOnStop".to_string(), json!(true));
    map.insert("useAllCores".to_string(), json!(true));
    map.insert("noBattlEye".to_string(), json!(true));
    map.insert("serverPlatform".to_string(), json!("ALL"));
    map.insert(
        "clusterDirOverride".to_string(),
        json!("ShooterGame/Saved/clusters"),
    );
    map.insert("customLaunchArgs".to_string(), json!("-culture=zh"));
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
