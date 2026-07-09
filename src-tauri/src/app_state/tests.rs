use super::*;
use crate::{
    instance_runtime_config::system_port_unavailable_reason,
    models::{AddInstancePayload, LogSource, ServerLogKind, ServerStatus},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    net::TcpListener,
    path::Path,
    sync::{Arc, Mutex},
};

fn test_runtime(data_dir: &Path) -> AppRuntime {
    AppRuntime {
        data_dir: Arc::new(data_dir.to_path_buf()),
        data: Arc::new(Mutex::new(ManagerData::default())),
        processes: Arc::new(Mutex::new(HashMap::new())),
        update_cancels: Arc::new(Mutex::new(HashMap::new())),
        online_players: Arc::new(Mutex::new(HashMap::new())),
    }
}

fn test_instance(id: &str, status: ServerStatus) -> ServerInstance {
    ServerInstance {
        id: id.to_string(),
        name: format!("实例-{id}"),
        map: "The Island".to_string(),
        map_code: "TheIsland_WP".to_string(),
        mode: "PvE".to_string(),
        status,
        game_port: 7777,
        query_port: 27015,
        players: 3,
        max_players: 30,
        install_path: format!("D:\\ASA-Server\\{id}"),
        rcon_port: 32330,
        cluster_id: "cluster".to_string(),
        description: String::new(),
        pid: Some(12_345),
        last_started_at: None,
        last_stopped_at: None,
        server_version: String::new(),
        version_state: "已安装/已更新".to_string(),
        last_error: Some("旧错误".to_string()),
        skip_auto_update_on_start_once: false,
    }
}

#[test]
fn 启动时恢复中断的安装更新状态() {
    let mut data = ManagerData::default();
    data.instances
        .push(test_instance("asa-stale-update", ServerStatus::Updating));

    assert!(recover_stale_update_statuses(&mut data));

    let instance = data.instances.first().expect("恢复后的实例存在");
    assert_eq!(instance.status, ServerStatus::Stopped);
    assert_eq!(instance.pid, None);
    assert_eq!(instance.players, 0);
    assert_eq!(instance.last_error, None);
    assert!(instance.skip_auto_update_on_start_once);
    assert!(instance.last_stopped_at.is_some());
    assert_eq!(data.logs.len(), 1);
    assert_eq!(data.logs[0].instance, "实例-asa-stale-update");
    assert_eq!(data.logs[0].level, "warn");
    assert!(data.logs[0].message.contains("已自动恢复为已停止"));
}

#[test]
fn 启动时不会误改正常停止状态() {
    let mut data = ManagerData::default();
    data.instances
        .push(test_instance("asa-stopped", ServerStatus::Stopped));

    assert!(!recover_stale_update_statuses(&mut data));
    assert_eq!(data.instances[0].status, ServerStatus::Stopped);
    assert!(!data.instances[0].skip_auto_update_on_start_once);
    assert!(data.logs.is_empty());
}

#[test]
fn 可清除启动自动更新一次性跳过标记() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let runtime = test_runtime(temp.path());
    let mut instance = test_instance("asa-recovered", ServerStatus::Stopped);
    instance.skip_auto_update_on_start_once = true;
    runtime.upsert_instance(instance).expect("写入实例");

    let updated = runtime
        .clear_startup_auto_update_skip_flags(&["asa-recovered".to_string()])
        .expect("清除跳过标记");

    assert_eq!(updated.len(), 1);
    assert!(!updated[0].skip_auto_update_on_start_once);
    assert!(
        !runtime
            .get_instance("asa-recovered")
            .expect("读取实例")
            .skip_auto_update_on_start_once
    );
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
fn 实例配置拒绝无准入条件的私有可见性() {
    let error = normalize_required_rcon_config(json!({
        "adminPassword": "ark-admin",
        "visibility": "private",
        "serverPassword": "",
        "exclusiveJoin": false,
        "whitelist": false
    }))
    .expect_err("私有可见性必须具备加入密码或 Exclusive Join");

    assert!(error.contains("私有"));
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
