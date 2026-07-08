use super::*;
use crate::models::{ServerInstance, ServerStatus};
use std::{fs, path::Path};

fn instance(root: &Path) -> ServerInstance {
    ServerInstance {
        id: "asa-test".to_string(),
        name: "测试/实例".to_string(),
        map: "The Island".to_string(),
        map_code: "TheIsland_WP".to_string(),
        mode: "PvE".to_string(),
        status: ServerStatus::Stopped,
        game_port: 7777,
        query_port: 27015,
        players: 0,
        max_players: 30,
        install_path: root.to_string_lossy().into_owned(),
        rcon_port: 32330,
        cluster_id: "Cluster".to_string(),
        description: String::new(),
        pid: None,
        last_started_at: None,
        last_stopped_at: None,
        server_version: String::new(),
        version_state: "已安装".to_string(),
        last_error: None,
        skip_auto_update_on_start_once: false,
    }
}

#[test]
fn 创建并恢复备份() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let install_root = temp.path().join("server");
    let saved = install_root.join("ShooterGame").join("Saved");
    fs::create_dir_all(&saved).expect("创建存档目录");
    fs::write(saved.join("world.ark"), "data").expect("写入存档");
    let backup_root = temp.path().join("backups");
    let instance = instance(&install_root);

    let backup = create_instance_backup(&backup_root, &instance).expect("创建备份");
    fs::remove_file(saved.join("world.ark")).expect("删除源文件");
    restore_instance_backup(&instance, Path::new(&backup.path)).expect("恢复备份");

    assert_eq!(fs::read_to_string(saved.join("world.ark")).unwrap(), "data");
}

#[test]
fn 按全局保留数量清理旧备份() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let backup_root = temp.path().join("backups");
    let instance = instance(&temp.path().join("server"));
    let instance_backup_dir = backup_root.join(naming::sanitize_filename(&instance.name));
    fs::create_dir_all(&instance_backup_dir).expect("创建备份目录");

    for index in 0..4 {
        fs::write(
            instance_backup_dir.join(format!("backup-{index}.zip")),
            format!("backup-{index}"),
        )
        .expect("写入备份占位文件");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let removed = prune_instance_backups(&backup_root, &instance, 2).expect("清理旧备份");
    let remaining = list_instance_backups(&backup_root, &instance).expect("读取备份列表");

    assert_eq!(removed, 2);
    assert_eq!(remaining.len(), 2);
}
