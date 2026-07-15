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

#[test]
fn imports_expanded_server_and_game_ini_settings() {
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
AutoSavePeriodMinutes=21.0
DisableStructureDecayPvE=False
PreventTribeAlliances=True
AllowHideDamageSourceFromLogs=False
AllowMultipleAttachedC4=True
AllowRaidDinoFeeding=True
ClampItemSpoilingTimes=True
ClampResourceHarvestDamage=True
DestroyTamesOverTheSoftTameLimit=True
DisableImprintDinoBuff=True
ForceAllStructureLocking=True
GlobalVoiceChat=True
PreventMateBoost=True
PreventOfflinePvP=True
PreventSpawnAnimations=True
ProximityChat=True
DisableDinoDecayPvE=True
PvPDinoDecay=True
RandomSupplyCratePoints=True
ServerForceNoHUD=True
ShowFloatingDamageText=True
DontAlwaysNotifyPlayerJoined=True
DinoCharacterHealthRecoveryMultiplier=1.1
MaxPersonalTamedDinos=12
MaxTamedDinos_SoftTameLimit=3456
MaxTamedDinos_SoftTameLimit_CountdownForDeletionDuration=789
MaxTributeDinos=22
MaxTributeItems=55
OxygenSwimSpeedStatMultiplier=1.2
PlatformSaddleBuildAreaBoundsMultiplier=1.3
PlayerCharacterHealthRecoveryMultiplier=1.4
PreventOfflinePvPInterval=60.5
PvEDinoDecayPeriodMultiplier=2.5
StructureResistanceMultiplier=0.75
"#,
    )
    .expect("写入 GUS");
    fs::write(
        config_dir.join("Game.ini"),
        r#"
[/Script/ShooterGame.ShooterGameMode]
bAllowUnlimitedRespecs=True
bDisablePhotoMode=True
bShowCreativeMode=True
bUseDinoLevelUpAnimations=False
bDisableWirelessCrafting=True
bDisableFriendlyFire=True
CraftingSkillBonusMultiplier=1.5
CraftXPMultiplier=1.6
CustomRecipeEffectivenessMultiplier=1.7
CustomRecipeSkillMultiplier=1.8
GenericXPMultiplier=1.9
HarvestXPMultiplier=2.1
KillXPMultiplier=2.2
SpecialXPMultiplier=2.3
HairGrowthSpeedMultiplier=0.25
MaxFallSpeedMultiplier=2.4
PoopIntervalMultiplier=2.5
WildDinoCharacterFoodDrainMultiplier=2.6
PhotoModeRangeLimit=3500.5
WirelessCraftingRangeOverride=4500.5
"#,
    )
    .expect("写入 Game.ini");

    let preview = read_server_directory_config(temp.path()).expect("读取扩展配置");
    for (key, expected) in [
        ("saveInterval", json!(21)),
        ("pveStructureDecay", json!(true)),
        ("tribeAlliances", json!(false)),
        ("allowHideDamageSourceFromLogs", json!(false)),
        ("allowMultipleAttachedC4", json!(true)),
        ("allowRaidDinoFeeding", json!(true)),
        ("clampItemSpoilingTimes", json!(true)),
        ("clampResourceHarvestDamage", json!(true)),
        ("destroyTamesOverSoftTameLimit", json!(true)),
        ("disableImprintDinoBuff", json!(true)),
        ("forceAllStructureLocking", json!(true)),
        ("globalVoiceChat", json!(true)),
        ("preventMateBoost", json!(true)),
        ("preventOfflinePvP", json!(true)),
        ("preventSpawnAnimations", json!(true)),
        ("proximityChat", json!(true)),
        ("pveDinoDecay", json!(false)),
        ("pvpDinoDecay", json!(true)),
        ("randomSupplyCratePoints", json!(true)),
        ("serverForceNoHud", json!(true)),
        ("showFloatingDamageText", json!(true)),
        ("showPlayerJoinNotifications", json!(false)),
        ("dinoHealthRecoveryMultiplier", json!(1.1)),
        ("maxPersonalTamedDinos", json!(12)),
        ("maxTamedDinosSoftTameLimit", json!(3456)),
        ("maxTamedDinosSoftTameLimitCountdown", json!(789)),
        ("maxTributeDinos", json!(22)),
        ("maxTributeItems", json!(55)),
        ("oxygenSwimSpeedStatMultiplier", json!(1.2)),
        ("platformSaddleBuildAreaBoundsMultiplier", json!(1.3)),
        ("playerHealthRecoveryMultiplier", json!(1.4)),
        ("preventOfflinePvPInterval", json!(60.5)),
        ("pveDinoDecayPeriodMultiplier", json!(2.5)),
        ("structureResistanceMultiplier", json!(0.75)),
        ("allowUnlimitedRespecs", json!(true)),
        ("disablePhotoMode", json!(true)),
        ("showCreativeMode", json!(true)),
        ("useDinoLevelUpAnimations", json!(false)),
        ("disableWirelessCrafting", json!(true)),
        ("disableFriendlyFirePvP", json!(true)),
        ("craftingSkillBonusMultiplier", json!(1.5)),
        ("craftXpMultiplier", json!(1.6)),
        ("customRecipeEffectivenessMultiplier", json!(1.7)),
        ("customRecipeSkillMultiplier", json!(1.8)),
        ("genericXpMultiplier", json!(1.9)),
        ("harvestXpMultiplier", json!(2.1)),
        ("killXpMultiplier", json!(2.2)),
        ("specialXpMultiplier", json!(2.3)),
        ("hairGrowthSpeedMultiplier", json!(0.25)),
        ("maxFallSpeedMultiplier", json!(2.4)),
        ("poopIntervalMultiplier", json!(2.5)),
        ("wildDinoFoodDrainMultiplier", json!(2.6)),
        ("photoModeRangeLimit", json!(3500.5)),
        ("wirelessCraftingRangeOverride", json!(4500.5)),
    ] {
        assert_eq!(preview.config[key], expected, "导入字段 {key} 不匹配");
    }
}
