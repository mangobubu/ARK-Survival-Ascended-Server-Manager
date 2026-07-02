use crate::models::{ModItem, ServerInstance};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct AppliedConfig {
    pub config_dir: PathBuf,
    pub game_user_settings_path: PathBuf,
    pub game_ini_path: PathBuf,
    pub launch_arguments: Vec<String>,
}

pub fn apply_instance_config(
    instance: &ServerInstance,
    config: &Value,
    mods: &[ModItem],
) -> Result<AppliedConfig, String> {
    let config_dir = config_dir(instance);
    fs::create_dir_all(&config_dir)
        .map_err(|error| format!("无法创建 ARK 配置目录 {}：{error}", config_dir.display()))?;

    let game_user_settings_path = config_dir.join("GameUserSettings.ini");
    let game_ini_path = config_dir.join("Game.ini");
    fs::write(
        &game_user_settings_path,
        render_game_user_settings(instance, config, mods),
    )
    .map_err(|error| {
        format!(
            "无法写入 GameUserSettings.ini {}：{error}",
            game_user_settings_path.display()
        )
    })?;
    fs::write(&game_ini_path, render_game_ini(config))
        .map_err(|error| format!("无法写入 Game.ini {}：{error}", game_ini_path.display()))?;

    Ok(AppliedConfig {
        config_dir,
        game_user_settings_path,
        game_ini_path,
        launch_arguments: build_launch_arguments(instance, config, mods),
    })
}

pub fn config_dir(instance: &ServerInstance) -> PathBuf {
    Path::new(&instance.install_path)
        .join("ShooterGame")
        .join("Saved")
        .join("Config")
        .join("WindowsServer")
}

pub fn saved_dir(instance: &ServerInstance) -> PathBuf {
    Path::new(&instance.install_path)
        .join("ShooterGame")
        .join("Saved")
}

pub fn server_executable(instance: &ServerInstance) -> Option<PathBuf> {
    let root = Path::new(&instance.install_path);
    [
        root.join("ShooterGame")
            .join("Binaries")
            .join("Win64")
            .join("ArkAscendedServer.exe"),
        root.join("ShooterGame")
            .join("Binaries")
            .join("Win64")
            .join("ShooterGameServer.exe"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

pub fn build_launch_arguments(
    instance: &ServerInstance,
    config: &Value,
    mods: &[ModItem],
) -> Vec<String> {
    let session_name = text(config, "sessionName", &instance.name);
    let server_password = text(config, "serverPassword", "");
    let admin_password = text(config, "adminPassword", "");
    let spectator_password = text(config, "spectatorPassword", "");
    let game_port = number_u16(config, "gamePort", instance.game_port);
    let query_port = number_u16(config, "queryPort", instance.query_port);
    let rcon_port = number_u16(config, "rconPort", instance.rcon_port);
    let max_players = number_u32(config, "maxPlayers", instance.max_players);

    let mut map_url = format!(
        "{}?listen?SessionName={}?Port={game_port}?QueryPort={query_port}?RCONPort={rcon_port}?MaxPlayers={max_players}",
        instance.map_code,
        url_component(&session_name)
    );

    if !server_password.is_empty() {
        map_url.push_str(&format!(
            "?ServerPassword={}",
            url_component(&server_password)
        ));
    }
    if !admin_password.is_empty() {
        map_url.push_str(&format!(
            "?ServerAdminPassword={}",
            url_component(&admin_password)
        ));
    }
    if !spectator_password.is_empty() {
        map_url.push_str(&format!(
            "?SpectatorPassword={}",
            url_component(&spectator_password)
        ));
    }
    if bool_value(config, "pve", instance.mode == "PvE") {
        map_url.push_str("?ServerPVE=True");
    }

    let mut args = vec![map_url];
    push_flag(&mut args, config, "useAllCores", "-USEALLAVAILABLECORES");
    push_flag(&mut args, config, "noBattlEye", "-NoBattlEye");
    push_flag(
        &mut args,
        config,
        "allowFlyerSpeedLeveling",
        "-AllowFlyerSpeedLeveling",
    );
    push_flag(
        &mut args,
        config,
        "forceAllowCaveFlyers",
        "-ForceAllowCaveFlyers",
    );
    push_flag(
        &mut args,
        config,
        "enableIdlePlayerKick",
        "-EnableIdlePlayerKick",
    );
    push_flag(
        &mut args,
        config,
        "preventHibernation",
        "-preventhibernation",
    );
    push_flag(
        &mut args,
        config,
        "stasisKeepControllers",
        "-StasisKeepControllers",
    );
    push_flag(
        &mut args,
        config,
        "useStructureStasisGrid",
        "-UseStructureStasisGrid",
    );
    push_flag(
        &mut args,
        config,
        "alwaysTickDedicatedSkeletalMeshes",
        "-AlwaysTickDedicatedSkeletalMeshes",
    );
    push_flag(
        &mut args,
        config,
        "noTransferFromFiltering",
        "-NoTransferFromFiltering",
    );
    push_flag(&mut args, config, "serverGameLog", "-servergamelog");
    push_flag(
        &mut args,
        config,
        "serverGameLogIncludeTribe",
        "-ServerRCONOutputTribeLogs",
    );
    push_flag(&mut args, config, "destroyWildDinos", "-ForceRespawnDinos");

    let gb_restart = number_u32(config, "gbUsageToForceRestart", 0);
    if gb_restart > 0 {
        args.push(format!("-GBUsageToForceRestart={gb_restart}"));
    }

    let server_platform = text(config, "serverPlatform", "ALL");
    if !server_platform.is_empty() {
        args.push(format!("-ServerPlatform={server_platform}"));
    }

    let active_event = text(config, "activeEvent", "");
    if !active_event.is_empty() {
        args.push(format!("-ActiveEvent={active_event}"));
    }

    let cluster_dir = text(config, "clusterDirOverride", "ShooterGame/Saved/clusters");
    if !cluster_dir.is_empty() {
        args.push(format!("-ClusterDirOverride=\"{cluster_dir}\""));
    }

    if bool_value(config, "useDynamicConfig", false) {
        args.push("-UseDynamicConfig".to_string());
        let dynamic_url = text(config, "customDynamicConfigUrl", "");
        if !dynamic_url.is_empty() {
            args.push(format!("-CustomDynamicConfigUrl=\"{dynamic_url}\""));
        }
    }

    let enabled_mods = active_mod_ids(mods);
    if !enabled_mods.is_empty() {
        args.push(format!("-mods={}", enabled_mods.join(",")));
    }

    let custom = text(config, "customLaunchArgs", "");
    if !custom.trim().is_empty() {
        args.extend(split_custom_args(&custom));
    }

    args
}

fn render_game_user_settings(
    instance: &ServerInstance,
    config: &Value,
    mods: &[ModItem],
) -> String {
    let active_mods = active_mod_ids(mods).join(",");
    let mut lines = vec![
        "[ServerSettings]".to_string(),
        format!(
            "SessionName={}",
            text(config, "sessionName", &instance.name)
        ),
        format!("ServerPassword={}", text(config, "serverPassword", "")),
        format!("ServerAdminPassword={}", text(config, "adminPassword", "")),
        format!(
            "SpectatorPassword={}",
            text(config, "spectatorPassword", "")
        ),
        format!(
            "RCONEnabled={}",
            ini_bool(bool_value(config, "rconEnabled", true))
        ),
        format!(
            "RCONPort={}",
            number_u16(config, "rconPort", instance.rcon_port)
        ),
        format!(
            "Port={}",
            number_u16(config, "gamePort", instance.game_port)
        ),
        format!(
            "QueryPort={}",
            number_u16(config, "queryPort", instance.query_port)
        ),
        format!(
            "MaxPlayers={}",
            number_u32(config, "maxPlayers", instance.max_players)
        ),
        format!(
            "ServerPVE={}",
            ini_bool(bool_value(config, "pve", instance.mode == "PvE"))
        ),
        format!(
            "ServerHardcore={}",
            ini_bool(bool_value(config, "hardcore", false))
        ),
        format!(
            "DisableFriendlyFire={}",
            ini_bool(bool_value(config, "disableFriendlyFire", false))
        ),
        format!(
            "EnablePVPGamma={}",
            ini_bool(bool_value(config, "enablePvPGamma", true))
        ),
        format!(
            "AllowHitMarkers={}",
            ini_bool(bool_value(config, "allowHitMarkers", true))
        ),
        format!(
            "AllowThirdPersonPlayer={}",
            ini_bool(bool_value(config, "thirdPerson", true))
        ),
        format!(
            "ServerCrosshair={}",
            ini_bool(bool_value(config, "crosshair", true))
        ),
        format!(
            "ShowMapPlayerLocation={}",
            ini_bool(bool_value(config, "showMapPlayer", true))
        ),
        format!(
            "AllowFlyerCarryPvE={}",
            ini_bool(bool_value(config, "flyerCarry", true))
        ),
        format!("XPMultiplier={}", number_f64(config, "xpMultiplier", 1.0)),
        format!(
            "TamingSpeedMultiplier={}",
            number_f64(config, "tamingSpeed", 1.0)
        ),
        format!(
            "HarvestAmountMultiplier={}",
            number_f64(config, "harvestAmount", 1.0)
        ),
        format!(
            "HarvestHealthMultiplier={}",
            number_f64(config, "harvestHealthMultiplier", 1.0)
        ),
        format!(
            "PlayerDamageMultiplier={}",
            number_f64(config, "playerDamageMultiplier", 1.0)
        ),
        format!(
            "PlayerResistanceMultiplier={}",
            number_f64(config, "playerResistanceMultiplier", 1.0)
        ),
        format!(
            "DinoDamageMultiplier={}",
            number_f64(config, "dinoDamageMultiplier", 1.0)
        ),
        format!(
            "DinoResistanceMultiplier={}",
            number_f64(config, "dinoResistanceMultiplier", 1.0)
        ),
        format!(
            "TamedDinoDamageMultiplier={}",
            number_f64(config, "tamedDinoDamageMultiplier", 1.0)
        ),
        format!(
            "TamedDinoResistanceMultiplier={}",
            number_f64(config, "tamedDinoResistanceMultiplier", 1.0)
        ),
        format!(
            "PlayerCharacterFoodDrainMultiplier={}",
            number_f64(config, "playerFoodDrainMultiplier", 1.0)
        ),
        format!(
            "PlayerCharacterWaterDrainMultiplier={}",
            number_f64(config, "playerWaterDrainMultiplier", 1.0)
        ),
        format!(
            "PlayerCharacterStaminaDrainMultiplier={}",
            number_f64(config, "playerStaminaDrainMultiplier", 1.0)
        ),
        format!(
            "DinoCharacterFoodDrainMultiplier={}",
            number_f64(config, "dinoFoodDrainMultiplier", 1.0)
        ),
        format!(
            "DinoCharacterStaminaDrainMultiplier={}",
            number_f64(config, "dinoStaminaDrainMultiplier", 1.0)
        ),
        format!(
            "DayCycleSpeedScale={}",
            number_f64(config, "dayCycleSpeed", 1.0)
        ),
        format!(
            "DayTimeSpeedScale={}",
            number_f64(config, "dayTimeSpeed", 1.0)
        ),
        format!(
            "NightTimeSpeedScale={}",
            number_f64(config, "nightTimeSpeed", 1.0)
        ),
        format!(
            "ResourcesRespawnPeriodMultiplier={}",
            number_f64(config, "resourceRespawn", 1.0)
        ),
        format!(
            "DinoCountMultiplier={}",
            number_f64(config, "dinoCount", 1.0)
        ),
        format!("ActiveMods={active_mods}"),
        String::new(),
        "[SessionSettings]".to_string(),
        format!(
            "SessionName={}",
            text(config, "sessionName", &instance.name)
        ),
    ];

    if bool_value(config, "adminLogging", true) {
        lines.push("AdminLogging=True".to_string());
    }
    if bool_value(config, "chatLogging", true) {
        lines.push("ChatLogging=True".to_string());
    }
    lines.push(String::new());
    lines.join("\r\n")
}

fn render_game_ini(config: &Value) -> String {
    let mut lines = vec![
        "[/Script/ShooterGame.ShooterGameMode]".to_string(),
        format!(
            "OverrideOfficialDifficulty={}",
            number_f64(config, "difficulty", 5.0)
        ),
        format!(
            "MatingIntervalMultiplier={}",
            number_f64(config, "matingInterval", 1.0)
        ),
        format!(
            "MatingSpeedMultiplier={}",
            number_f64(config, "matingSpeedMultiplier", 1.0)
        ),
        format!(
            "EggHatchSpeedMultiplier={}",
            number_f64(config, "eggHatchSpeed", 1.0)
        ),
        format!(
            "BabyMatureSpeedMultiplier={}",
            number_f64(config, "babyMatureSpeed", 1.0)
        ),
        format!(
            "BabyCuddleIntervalMultiplier={}",
            number_f64(config, "cuddleInterval", 1.0)
        ),
        format!(
            "BabyFoodConsumptionSpeedMultiplier={}",
            number_f64(config, "babyFoodConsumption", 1.0)
        ),
        format!(
            "LayEggIntervalMultiplier={}",
            number_f64(config, "layEggIntervalMultiplier", 1.0)
        ),
        format!(
            "BabyCuddleGracePeriodMultiplier={}",
            number_f64(config, "babyCuddleGracePeriodMultiplier", 1.0)
        ),
        format!(
            "BabyCuddleLoseImprintQualitySpeedMultiplier={}",
            number_f64(config, "babyCuddleLoseImprintQualitySpeedMultiplier", 1.0)
        ),
        format!(
            "BabyImprintingStatScaleMultiplier={}",
            number_f64(config, "babyImprintingStatScaleMultiplier", 1.0)
        ),
        format!(
            "BabyImprintAmountMultiplier={}",
            number_f64(config, "babyImprintAmountMultiplier", 1.0)
        ),
        format!(
            "bAllowAnyoneBabyImprintCuddle={}",
            ini_bool(bool_value(config, "allowAnyoneBabyImprintCuddle", false))
        ),
        format!(
            "PerPlatformMaxStructuresMultiplier={}",
            number_f64(config, "platformStructureMultiplier", 1.0)
        ),
        format!(
            "TheMaxStructuresInRange={}",
            number_u32(config, "structureLimit", 10_500)
        ),
        format!(
            "bDisableStructurePlacementCollision={}",
            ini_bool(bool_value(config, "disablePlacementCollision", false))
        ),
        format!(
            "MaxNumberOfPlayersInTribe={}",
            number_u32(config, "maxTribeSize", 0)
        ),
        format!(
            "bPvEAllowTribeWar={}",
            ini_bool(bool_value(config, "tribeAlliances", true))
        ),
        format!("bDisableDinoBreeding={}", ini_bool(false)),
    ];

    let lost_colony_settings = [
        ("LimitBunkersPerTribe", "limitBunkersPerTribe"),
        (
            "AllowBunkersInPreventionZones",
            "allowBunkersInPreventionZones",
        ),
        (
            "AllowRidingDinosInsideBunkers",
            "allowRidingDinosInsideBunkers",
        ),
        (
            "AllowBunkerModulesAboveGround",
            "allowBunkerModulesAboveGround",
        ),
        ("AllowDinoAIInsideBunkers", "allowDinoAIInsideBunkers"),
        (
            "AllowBunkerModulesInPreventionZones",
            "allowBunkerModulesInPreventionZones",
        ),
    ];
    for (ini_key, config_key) in lost_colony_settings {
        lines.push(format!(
            "{ini_key}={}",
            ini_bool(bool_value(config, config_key, false))
        ));
    }
    lines.push(format!(
        "LimitBunkersPerTribeNum={}",
        number_u32(config, "limitBunkersPerTribeNum", 0)
    ));
    lines.push(format!(
        "MinDistanceBetweenBunkers={}",
        number_u32(config, "minDistanceBetweenBunkers", 0)
    ));
    lines.push(format!(
        "EnemyAccessBunkerHPThreshold={}",
        number_f64(config, "enemyAccessBunkerHPThreshold", 0.0)
    ));
    lines.push(format!(
        "BunkerUnderHPThresholdDmgMultiplier={}",
        number_f64(config, "bunkerUnderHPThresholdDmgMultiplier", 1.0)
    ));
    lines.push(String::new());
    lines.join("\r\n")
}

fn active_mod_ids(mods: &[ModItem]) -> Vec<String> {
    mods.iter()
        .filter(|item| item.enabled)
        .map(|item| item.id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

fn push_flag(args: &mut Vec<String>, config: &Value, key: &str, flag: &str) {
    if bool_value(config, key, false) {
        args.push(flag.to_string());
    }
}

fn text(config: &Value, key: &str, fallback: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn bool_value(config: &Value, key: &str, fallback: bool) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}

fn number_u16(config: &Value, key: &str, fallback: u16) -> u16 {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(fallback)
}

fn number_u32(config: &Value, key: &str, fallback: u32) -> u32 {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(fallback)
}

fn number_f64(config: &Value, key: &str, fallback: f64) -> f64 {
    config.get(key).and_then(Value::as_f64).unwrap_or(fallback)
}

fn ini_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

fn url_component(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('?', "%3F")
        .replace('&', "%26")
        .replace(' ', "%20")
        .replace('"', "%22")
}

fn split_custom_args(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ServerInstance, ServerStatus};
    use serde_json::json;

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
            version_state: "未安装".to_string(),
            last_error: None,
        }
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
        assert!(args[0].contains("Port=7788"));
        assert!(args.iter().any(|arg| arg == "-USEALLAVAILABLECORES"));
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
    }
}
