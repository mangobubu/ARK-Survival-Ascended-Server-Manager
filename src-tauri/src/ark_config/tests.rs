use super::*;
use crate::{
    ark_config_game_user_settings::render_game_user_settings,
    ark_config_ini::{render_engine_ini, render_game_ini},
    models::{ServerInstance, ServerStatus},
};
use serde_json::json;
use std::path::Path;

fn test_instance(path: &Path) -> ServerInstance {
    ServerInstance {
        id: "asa-test".to_string(),
        name: "测试服".to_string(),
        map: "The Island".to_string(),
        map_code: "TheIsland_WP".to_string(),
        mode: "PvE".to_string(),
        status: ServerStatus::Stopped,
        game_port: 7777,
        query_port: 27015,
        players: 0,
        max_players: 30,
        install_path: path.to_string_lossy().into_owned(),
        rcon_port: 32330,
        cluster_id: "Cluster".to_string(),
        description: String::new(),
        pid: None,
        last_started_at: None,
        last_stopped_at: None,
        server_version: String::new(),
        version_state: "未安装".to_string(),
        last_error: None,
        skip_auto_update_on_start_once: false,
    }
}

#[test]
fn server_admin_password_is_last_map_url_option() {
    let instance = test_instance(Path::new("D:\\ASA"));
    let config = json!({
        "adminPassword": "admin.pass",
        "pve": true
    });

    let args = build_launch_arguments(&instance, &config, &[]);

    assert!(args[0].contains("?ServerPVE=True?ServerAdminPassword=admin.pass"));
    assert!(args[0].ends_with("?ServerAdminPassword=admin.pass"));
}

#[test]
fn harvest_multipliers_are_added_to_launch_url_before_admin_password() {
    let instance = test_instance(Path::new("D:\\ASA"));
    let config = json!({
        "adminPassword": "admin.pass",
        "pve": true,
        "harvestAmount": 5.0,
        "harvestHealthMultiplier": 1.0
    });

    let args = build_launch_arguments(&instance, &config, &[]);

    assert!(args[0].contains("?HarvestAmountMultiplier=5"));
    assert!(args[0].contains("?HarvestHealthMultiplier=1"));
    assert!(args[0].ends_with("?ServerAdminPassword=admin.pass"));
    assert!(
        args[0].find("HarvestAmountMultiplier").unwrap()
            < args[0].find("ServerAdminPassword").unwrap()
    );
}

#[test]
fn private_visibility_requires_password_or_exclusive_join() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let instance = test_instance(temp.path());
    let config = json!({
        "adminPassword": "admin",
        "visibility": "private",
        "serverPassword": "",
        "exclusiveJoin": false,
        "whitelist": false
    });

    let error =
        apply_instance_config(&instance, &config, &[]).expect_err("私有模式缺少准入条件应失败");

    assert!(error.contains("私有"));
    assert!(!config_dir(&instance).exists());
}

#[test]
fn private_visibility_allows_exclusive_join_without_password() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let instance = test_instance(temp.path());
    let config = json!({
        "adminPassword": "admin",
        "visibility": "private",
        "serverPassword": "",
        "exclusiveJoin": true
    });

    let applied =
        apply_instance_config(&instance, &config, &[]).expect("Exclusive Join 可作为私有准入条件");
    let game_user_settings =
        fs::read_to_string(applied.game_user_settings_path).expect("read GameUserSettings.ini");

    assert!(game_user_settings.contains("ServerPassword="));
    assert!(
        applied
            .launch_arguments
            .iter()
            .any(|arg| arg == "-exclusivejoin")
    );
    assert!(!applied.launch_arguments[0].contains("?ServerPassword="));
}

#[test]
fn 生成启动参数包含地图端口与模组() {
    let instance = test_instance(Path::new("D:\\ASA"));
    let config = json!({
        "sessionName": "中文 测试",
        "gamePort": 7788,
        "queryPort": 27016,
        "rconPort": 32331,
        "maxPlayers": 20,
        "useAllCores": true,
        "clusterId": "Cluster-Z",
        "whitelist": true,
        "customLaunchArgs": "-culture=zh"
    });
    let mods = vec![ModItem {
        id: "928708".to_string(),
        name: "测试 MOD".to_string(),
        version: "1.0".to_string(),
        size: "1 MB".to_string(),
        enabled: true,
        update_available: None,
    }];

    let args = build_launch_arguments(&instance, &config, &mods);
    assert!(args[0].contains("TheIsland_WP?listen"));
    assert!(!args[0].contains("Port=7788"));
    assert!(!args[0].contains("MaxPlayers=20"));
    assert!(args.iter().any(|arg| arg == "-port=7788"));
    assert!(args.iter().any(|arg| arg == "-WinLiveMaxPlayers=20"));
    assert!(args.iter().any(|arg| arg == "-USEALLAVAILABLECORES"));
    assert!(args.iter().any(|arg| arg == "-clusterid=Cluster-Z"));
    assert!(args.iter().any(|arg| arg == "-exclusivejoin"));
    assert!(args.iter().any(|arg| arg == "-mods=928708"));
}

#[test]
fn 写入真实配置文件() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let instance = test_instance(temp.path());
    let config = json!({ "sessionName": "测试服", "adminPassword": "admin" });

    let applied = apply_instance_config(&instance, &config, &[]).expect("写入配置成功");
    assert!(applied.game_user_settings_path.is_file());
    assert!(applied.game_ini_path.is_file());
    assert!(applied.engine_ini_path.is_file());

    let game_user_settings =
        fs::read_to_string(applied.game_user_settings_path).expect("read GameUserSettings.ini");
    let server_settings_section = game_user_settings
        .split("[SessionSettings]")
        .next()
        .unwrap_or_default();
    assert!(!server_settings_section.contains("MaxPlayers="));
    assert!(!game_user_settings.contains("[/Script/Engine.GameSession]"));
    assert!(!game_user_settings.contains("MaxPlayers=30"));
    assert!(
        applied
            .launch_arguments
            .iter()
            .any(|arg| arg == "-WinLiveMaxPlayers=30")
    );
}

#[test]
fn 写入扩展配置到正确文件() {
    let instance = test_instance(Path::new("D:\\ASA"));
    let mut config = json!({
        "clusterId": "Cluster-Exact",
        "crossTransfer": false,
        "preventDownloadItems": false,
        "preventDownloadDinos": false,
        "preventDownloadSurvivors": false,
        "preventUploadItems": false,
        "preventUploadDinos": false,
        "preventUploadSurvivors": false,
        "noTributeDownloads": false,
        "rconBufferSize": 9000,
        "kickIdlePlayersPeriod": 120,
        "allowFlyingStaminaRecovery": true,
        "allowCaveBuildingPvE": true,
        "allowCaveBuildingPvP": false,
        "pveAllowStructuresAtSupplyDrops": true,
        "enableExtraStructurePreventionVolumes": true,
        "enableIdlePlayerKick": true,
        "tributeCharacterExpirationSeconds": 3600,
        "tributeDinoExpirationSeconds": 7200,
        "tributeItemExpirationSeconds": 1800,
        "crossArkAllowForeignDinoDownloads": true,
        "resourceNoReplenishRadiusPlayers": 2.5,
        "resourceNoReplenishRadiusStructures": 3.5,
        "maxTamedDinos": 1234,
        "harvestAmount": 5.0,
        "harvestHealthMultiplier": 1.5,
        "cropGrowthSpeedMultiplier": 4.5,
        "cropDecaySpeedMultiplier": 5.5,
        "supplyCrateLootQualityMultiplier": 2.25,
        "fishingLootQualityMultiplier": 1.75,
        "fuelConsumptionIntervalMultiplier": 6.5,
        "itemStackSizeMultiplier": 7.5,
        "itemStackOverrides": [
            {"itemClassString": "PrimalItemResource_Stone_C", "maxItemQuantity": 1000, "ignoreMultiplier": true},
            {"itemClassString": "PrimalItemResource_Wood_C", "maxItemQuantity": 500, "ignoreMultiplier": false},
            {"itemClassString": "   ", "maxItemQuantity": 50, "ignoreMultiplier": true}
        ],
        "globalSpoilingTimeMultiplier": 8.5,
        "globalItemDecompositionTimeMultiplier": 9.5,
        "globalCorpseDecompositionTimeMultiplier": 10.5,
        "raidDinoFoodDrainMultiplier": 0.75,
        "minimumDinoReuploadInterval": 1800
    });
    let extra_config = json!({
        "adminLogging": false,
        "chatLogging": false,
        "structureDamageRepairCooldown": 77,
        "structurePickupTimeAfterPlacement": 88,
        "structurePickupHoldDuration": 1.5,
        "autoDestroyOldStructuresMultiplier": 2.25,
        "limitGeneratorsNum": 4,
        "limitGeneratorsRange": 500,
        "tribeNameChangeCooldown": 3,
        "maxAlliancesPerTribe": 2,
        "maxTribesPerAlliance": 6,
        "disableFriendlyFire": true,
        "fastDecayUnsnappedCoreStructures": true,
        "allowCryoFridgeOnSaddle": true,
        "disableCryopodEnemyCheck": true,
        "disableCryopodFridgeRequirement": true,
        "disableCryopodCooldown": true,
        "allowSpeedLeveling": true,
        "enableDiseases": false,
        "nonPermanentDiseases": true,
        "networkTickRate": 45,
        "maxClientRate": 150000
    });
    config
        .as_object_mut()
        .expect("测试配置是对象")
        .extend(extra_config.as_object().expect("扩展配置是对象").clone());

    let game_user_settings = render_game_user_settings(&instance, &config, &[]);
    let game_ini = render_game_ini(&config);
    let engine_ini = render_engine_ini(&config);
    let server_settings_section = game_user_settings
        .split("[SessionSettings]")
        .next()
        .unwrap_or_default();

    for expected in [
        "RCONServerGameLogBuffer=9000",
        "AllowFlyingStaminaRecovery=True",
        "AllowCaveBuildingPvE=True",
        "AllowCaveBuildingPvP=False",
        "PvEAllowStructuresAtSupplyDrops=True",
        "EnableExtraStructurePreventionVolumes=True",
        "KickIdlePlayersPeriod=120",
        "PreventDownloadItems=True",
        "PreventDownloadDinos=True",
        "PreventDownloadSurvivors=True",
        "PreventUploadItems=True",
        "PreventUploadDinos=True",
        "PreventUploadSurvivors=True",
        "NoTributeDownloads=True",
        "OverrideOfficialDifficulty=5",
        "TributeCharacterExpirationSeconds=3600",
        "TributeDinoExpirationSeconds=7200",
        "TributeItemExpirationSeconds=1800",
        "MaxTamedDinos=1234",
        "HarvestAmountMultiplier=5",
        "HarvestHealthMultiplier=1.5",
        "ItemStackSizeMultiplier=7.5",
        "RaidDinoCharacterFoodDrainMultiplier=0.75",
        "MinimumDinoReuploadInterval=1800",
        "StructurePickupHoldDuration=1.5",
        "StructurePickupTimeAfterPlacement=88",
        "AutoDestroyOldStructuresMultiplier=2.25",
        "FastDecayUnsnappedCoreStructures=True",
        "AllowCryoFridgeOnSaddle=True",
        "DisableCryopodEnemyCheck=True",
        "DisableCryopodFridgeRequirement=True",
        "DisableCryopodCooldown=True",
        "PreventDiseases=True",
        "NonPermanentDiseases=True",
        "TribeNameChangeCooldown=3",
        "AdminLogging=False",
        "ChatLogging=False",
    ] {
        assert!(
            server_settings_section.contains(expected),
            "ServerSettings 缺少 {expected}"
        );
    }

    for expected in [
        "ResourceNoReplenishRadiusPlayers=2.5",
        "ResourceNoReplenishRadiusStructures=3.5",
        "CropGrowthSpeedMultiplier=4.5",
        "CropDecaySpeedMultiplier=5.5",
        "SupplyCrateLootQualityMultiplier=2.25",
        "FishingLootQualityMultiplier=1.75",
        "FuelConsumptionIntervalMultiplier=6.5",
        "GlobalSpoilingTimeMultiplier=8.5",
        "GlobalItemDecompositionTimeMultiplier=9.5",
        "GlobalCorpseDecompositionTimeMultiplier=10.5",
        "StructureDamageRepairCooldown=77",
        "LimitGeneratorsNum=4",
        "LimitGeneratorsRange=500",
        "bAllowSpeedLeveling=True",
        "MaxAlliancesPerTribe=2",
        "MaxTribesPerAlliance=6",
        "bPvEDisableFriendlyFire=True",
        "ConfigOverrideItemMaxQuantity=(ItemClassString=\"PrimalItemResource_Stone_C\",Quantity=(MaxItemQuantity=1000,bIgnoreMultiplier=True))",
        "ConfigOverrideItemMaxQuantity=(ItemClassString=\"PrimalItemResource_Wood_C\",Quantity=(MaxItemQuantity=500,bIgnoreMultiplier=False))",
    ] {
        assert!(game_ini.contains(expected), "Game.ini 缺少 {expected}");
    }

    assert!(engine_ini.contains("[/Script/OnlineSubsystemUtils.IpNetDriver]"));
    assert!(engine_ini.contains("NetServerMaxTickRate=45"));
    assert!(engine_ini.contains("MaxClientRate=150000"));
    assert!(!game_ini.contains("PreventDownloadItems="));
    assert!(!game_ini.contains("AllowCryoFridgeOnSaddle="));
    assert!(!game_ini.contains("EnableDiseases="));
    assert!(!game_user_settings.contains("NetServerMaxTickRate="));
    assert!(!server_settings_section.contains("CrossARKAllowForeignDinoDownloads="));
    assert!(!server_settings_section.contains("LimitBunkersPerTribe="));
    assert!(!server_settings_section.contains("AllowBunkerModulesAboveGround="));

    let mut aberration = instance.clone();
    aberration.map = "Aberration".to_string();
    aberration.map_code = "Aberration_WP".to_string();
    let aberration_settings = render_game_user_settings(&aberration, &config, &[]);
    let aberration_server_settings = aberration_settings
        .split("[SessionSettings]")
        .next()
        .unwrap_or_default();
    assert!(aberration_server_settings.contains("CrossARKAllowForeignDinoDownloads=True"));
    assert!(!aberration_server_settings.contains("LimitBunkersPerTribe="));

    let mut lost_colony = instance;
    lost_colony.map = "Lost Colony".to_string();
    lost_colony.map_code = "LostColony_WP".to_string();
    let lost_colony_settings = render_game_user_settings(&lost_colony, &config, &[]);
    let lost_colony_server_settings = lost_colony_settings
        .split("[SessionSettings]")
        .next()
        .unwrap_or_default();
    for expected in [
        "LimitBunkersPerTribe=True",
        "AllowBunkersInPreventionZones=False",
        "AllowRidingDinosInsideBunkers=True",
        "AllowBunkerModulesAboveGround=False",
        "AllowDinoAIInsideBunkers=True",
        "AllowBunkerModulesInPreventionZones=False",
        "LimitBunkersPerTribeNum=3",
        "MinDistanceBetweenBunkers=3000",
        "EnemyAccessBunkerHPThreshold=0.25",
        "BunkerUnderHPThresholdDmgMultiplier=0.05",
    ] {
        assert!(
            lost_colony_server_settings.contains(expected),
            "Lost Colony ServerSettings 缺少 {expected}"
        );
    }
    assert!(!lost_colony_server_settings.contains("CrossARKAllowForeignDinoDownloads="));
}
