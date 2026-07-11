use super::*;
use serde_json::json;
use std::fs;

#[test]
fn reads_server_config_and_mods_from_install_root() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let config_dir = temp
        .path()
        .join("ShooterGame")
        .join("Saved")
        .join("Config")
        .join("WindowsServer");
    fs::create_dir_all(&config_dir).expect("创建配置目录");
    fs::write(
        config_dir.join("GameUserSettings.ini"),
        r#"
[ServerSettings]
ServerPassword=
ServerAdminPassword=admin-123
RCONEnabled=True
RCONPort=32340
RCONServerGameLogBuffer=9000
ServerPVE=False
OverrideOfficialDifficulty=7.0
XPMultiplier=3.5
AllowFlyingStaminaRecovery=True
CrossARKAllowForeignDinoDownloads=True
PvEAllowStructuresAtSupplyDrops=True
EnableExtraStructurePreventionVolumes=True
AlwaysAllowStructurePickup=True
PerPlatformMaxStructuresMultiplier=1.5
TheMaxStructuresInRange=10500
LimitBunkersPerTribe=True
LimitBunkersPerTribeNum=3
ActiveMods=12345, 67890, 12345, abc

[SessionSettings]
SessionName=导入服务器
Port=7787
QueryPort=27025

[/Script/Engine.GameSession]
MaxPlayers=42
"#,
    )
    .expect("写入 GameUserSettings.ini");
    fs::write(
        config_dir.join("Game.ini"),
        r#"
[/Script/ShooterGame.ShooterGameMode]
ResourceNoReplenishRadiusPlayers=2.5
MatingIntervalMultiplier=0.2
bDisableStructurePlacementCollision=True
bAllowSpeedLeveling=True
ConfigOverrideItemMaxQuantity=(ItemClassString="PrimalItemResource_Stone_C",Quantity=(MaxItemQuantity=1000,bIgnoreMultiplier=True))
ConfigOverrideItemMaxQuantity=(ItemClassString="PrimalItemResource_Wood_C",Quantity=(MaxItemQuantity=500,bIgnoreMultiplier=False))
"#,
    )
    .expect("写入 Game.ini");
    fs::write(
        config_dir.join("Engine.ini"),
        r#"
[/Script/OnlineSubsystemUtils.IpNetDriver]
NetServerMaxTickRate=45
MaxClientRate=150000
"#,
    )
    .expect("写入 Engine.ini");
    let save_dir = temp
        .path()
        .join("ShooterGame")
        .join("Saved")
        .join("SavedArks");
    fs::create_dir_all(&save_dir).expect("创建存档目录");
    fs::write(save_dir.join("TheCenter_WP.ark"), b"").expect("写入地图存档");

    let preview = read_server_directory_config(temp.path()).expect("读取服务端配置");

    assert_eq!(preview.name.as_deref(), Some("导入服务器"));
    assert_eq!(preview.mode.as_deref(), Some("PvP"));
    assert_eq!(preview.game_port, Some(7787));
    assert_eq!(preview.query_port, Some(27025));
    assert_eq!(preview.rcon_port, Some(32340));
    assert_eq!(preview.max_players, Some(42));
    assert_eq!(preview.map_code.as_deref(), Some("TheCenter_WP"));
    assert_eq!(preview.mods.len(), 2);
    assert_eq!(preview.config["xpMultiplier"], json!(3.5));
    assert_eq!(preview.config["rconBufferSize"], json!(9000));
    assert_eq!(preview.config["allowFlyingStaminaRecovery"], json!(true));
    assert_eq!(
        preview.config["pveAllowStructuresAtSupplyDrops"],
        json!(true)
    );
    assert_eq!(
        preview.config["enableExtraStructurePreventionVolumes"],
        json!(true)
    );
    assert_eq!(preview.config["alwaysAllowStructurePickup"], json!(true));
    assert_eq!(
        preview.config["crossArkAllowForeignDinoDownloads"],
        json!(true)
    );
    assert_eq!(
        preview.config["resourceNoReplenishRadiusPlayers"],
        json!(2.5)
    );
    assert_eq!(preview.config["platformStructureMultiplier"], json!(1.5));
    assert_eq!(preview.config["networkTickRate"], json!(45));
    assert_eq!(preview.config["maxClientRate"], json!(150000));
    assert_eq!(preview.config["structureLimit"], json!(10500));
    assert_eq!(preview.config["limitBunkersPerTribe"], json!(true));
    assert_eq!(preview.config["limitBunkersPerTribeNum"], json!(3));
    assert_eq!(preview.config["difficulty"], json!(7.0));
    assert_eq!(preview.config["disablePlacementCollision"], json!(true));
    assert_eq!(preview.config["allowSpeedLeveling"], json!(true));
    assert_eq!(
        preview.config["itemStackOverrides"],
        json!([
            {
                "itemClassString": "PrimalItemResource_Stone_C",
                "maxItemQuantity": 1000,
                "ignoreMultiplier": true
            },
            {
                "itemClassString": "PrimalItemResource_Wood_C",
                "maxItemQuantity": 500,
                "ignoreMultiplier": false
            }
        ])
    );
}
