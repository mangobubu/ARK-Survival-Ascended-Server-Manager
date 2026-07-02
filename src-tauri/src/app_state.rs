use crate::models::{
    AddInstancePayload, GlobalSettings, LogLine, ModItem, ServerInstance, ServerStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
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
            if let Some(current) = data.instances.iter_mut().find(|item| item.id == instance.id) {
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
            data.configs.insert(
                id.clone(),
                default_config_from_payload(&payload, &instance),
            );
            data.mods.insert(id.clone(), Vec::new());
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
        {
            let mut data = self.lock()?;
            data.configs.insert(instance_id.to_string(), config);
            data.mods.insert(instance_id.to_string(), mods);
        }
        self.persist()
    }

    pub fn add_log(&self, instance: &str, level: &str, message: &str) -> Result<LogLine, String> {
        let line = LogLine {
            id: now_millis(),
            time: current_time_text(),
            instance: instance.to_string(),
            level: level.to_string(),
            message: message.to_string(),
        };
        {
            let mut data = self.lock()?;
            data.logs.push(line.clone());
            if data.logs.len() > MAX_LOG_LINES {
                let overflow = data.logs.len() - MAX_LOG_LINES;
                data.logs.drain(0..overflow);
            }
        }
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

    pub fn lock_processes(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, Child>>, String> {
        self.processes
            .lock()
            .map_err(|_| "运行进程状态锁已损坏".to_string())
    }

    pub fn snapshot(&self) -> Result<ManagerData, String> {
        Ok(self.lock()?.clone())
    }

    pub fn replace_from_import(&self, bundles: Vec<crate::models::InstanceConfigBundle>) -> Result<(usize, usize), String> {
        let mut imported = 0;
        let mut skipped = 0;
        {
            let mut data = self.lock()?;
            for bundle in bundles {
                if data.instances.iter().any(|item| item.id == bundle.instance.id) {
                    skipped += 1;
                    continue;
                }
                data.configs.insert(bundle.instance.id.clone(), bundle.config);
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
    Ok(())
}

fn ensure_port_available(
    instances: &[ServerInstance],
    port: u16,
    label: &str,
) -> Result<(), String> {
    if instances.iter().any(|item| {
        item.game_port == port || item.query_port == port || item.rcon_port == port
    }) {
        return Err(format!("{label} {port} 已被其他实例占用"));
    }
    Ok(())
}

fn normalize_path_text(path: &str) -> PathBuf {
    PathBuf::from(path.trim().trim_matches('"'))
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
    map.insert("clusterDirOverride".to_string(), json!("ShooterGame/Saved/clusters"));
    map.insert("customLaunchArgs".to_string(), json!("-culture=zh"));
    Value::Object(map)
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
