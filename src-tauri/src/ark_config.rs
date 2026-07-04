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
    pub engine_ini_path: PathBuf,
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
    let engine_ini_path = config_dir.join("Engine.ini");
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
    fs::write(&engine_ini_path, render_engine_ini(config))
        .map_err(|error| format!("无法写入 Engine.ini {}：{error}", engine_ini_path.display()))?;

    Ok(AppliedConfig {
        config_dir,
        game_user_settings_path,
        game_ini_path,
        engine_ini_path,
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
        "{}?listen?SessionName={}?QueryPort={query_port}?RCONPort={rcon_port}",
        instance.map_code,
        url_component(&session_name)
    );

    if !server_password.is_empty() {
        map_url.push_str(&format!(
            "?ServerPassword={}",
            url_component(&server_password)
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
    if !admin_password.is_empty() {
        // ASA persists trailing URL options into ServerAdminPassword, so keep it last.
        map_url.push_str(&format!(
            "?ServerAdminPassword={}",
            url_component(&admin_password)
        ));
    }

    let mut args = vec![map_url];
    args.push(format!("-port={game_port}"));
    args.push(format!("-WinLiveMaxPlayers={max_players}"));
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
    if bool_value(config, "whitelist", false) || bool_value(config, "exclusiveJoin", false) {
        args.push("-exclusivejoin".to_string());
    }

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

    let cluster_id = text(config, "clusterId", &instance.cluster_id);
    if !cluster_id.is_empty() {
        args.push(format!("-clusterid={cluster_id}"));
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
    let cross_transfer = bool_value(config, "crossTransfer", true);
    let prevent_download_items =
        !cross_transfer || bool_value(config, "preventDownloadItems", false);
    let prevent_download_dinos =
        !cross_transfer || bool_value(config, "preventDownloadDinos", false);
    let prevent_download_survivors =
        !cross_transfer || bool_value(config, "preventDownloadSurvivors", false);
    let prevent_upload_items = !cross_transfer || bool_value(config, "preventUploadItems", false);
    let prevent_upload_dinos = !cross_transfer || bool_value(config, "preventUploadDinos", false);
    let prevent_upload_survivors =
        !cross_transfer || bool_value(config, "preventUploadSurvivors", false);
    let no_tribute_downloads = !cross_transfer || bool_value(config, "noTributeDownloads", false);

    let mut lines = vec![
        "[ServerSettings]".to_string(),
        format!("ServerPassword={}", text(config, "serverPassword", "")),
        format!("ServerAdminPassword={}", text(config, "adminPassword", "")),
        format!(
            "SpectatorPassword={}",
            text(config, "spectatorPassword", "")
        ),
        "RCONEnabled=True".to_string(),
        format!(
            "RCONPort={}",
            number_u16(config, "rconPort", instance.rcon_port)
        ),
        format!(
            "RCONServerGameLogBuffer={}",
            number_u32(config, "rconBufferSize", 6000)
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
            "EnablePvPGamma={}",
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
        format!(
            "AllowFlyingStaminaRecovery={}",
            ini_bool(bool_value(config, "allowFlyingStaminaRecovery", false))
        ),
        format!(
            "AllowCaveBuildingPvE={}",
            ini_bool(bool_value(config, "allowCaveBuildingPvE", false))
        ),
        format!(
            "AllowCaveBuildingPvP={}",
            ini_bool(bool_value(config, "allowCaveBuildingPvP", true))
        ),
        format!(
            "KickIdlePlayersPeriod={}",
            number_u32(config, "kickIdlePlayersPeriod", 3600)
        ),
        format!("PreventDownloadItems={}", ini_bool(prevent_download_items)),
        format!("PreventDownloadDinos={}", ini_bool(prevent_download_dinos)),
        format!(
            "PreventDownloadSurvivors={}",
            ini_bool(prevent_download_survivors)
        ),
        format!("PreventUploadItems={}", ini_bool(prevent_upload_items)),
        format!("PreventUploadDinos={}", ini_bool(prevent_upload_dinos)),
        format!(
            "PreventUploadSurvivors={}",
            ini_bool(prevent_upload_survivors)
        ),
        format!("NoTributeDownloads={}", ini_bool(no_tribute_downloads)),
        format!(
            "OverrideOfficialDifficulty={}",
            number_f64(config, "difficulty", 5.0)
        ),
        format!(
            "TributeCharacterExpirationSeconds={}",
            number_u32(config, "tributeCharacterExpirationSeconds", 0)
        ),
        format!(
            "TributeDinoExpirationSeconds={}",
            number_u32(config, "tributeDinoExpirationSeconds", 0)
        ),
        format!(
            "TributeItemExpirationSeconds={}",
            number_u32(config, "tributeItemExpirationSeconds", 0)
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
        format!(
            "MaxTamedDinos={}",
            number_u32(config, "maxTamedDinos", 5000)
        ),
        format!(
            "ItemStackSizeMultiplier={}",
            number_f64(config, "itemStackSizeMultiplier", 1.0)
        ),
        format!(
            "RaidDinoCharacterFoodDrainMultiplier={}",
            number_f64(config, "raidDinoFoodDrainMultiplier", 1.0)
        ),
        format!(
            "MinimumDinoReuploadInterval={}",
            number_f64(config, "minimumDinoReuploadInterval", 0.0)
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
            "StructurePickupHoldDuration={}",
            number_f64(config, "structurePickupHoldDuration", 0.5)
        ),
        format!(
            "StructurePickupTimeAfterPlacement={}",
            number_f64(config, "structurePickupTimeAfterPlacement", 30.0)
        ),
        format!(
            "AutoDestroyOldStructuresMultiplier={}",
            number_f64(config, "autoDestroyOldStructuresMultiplier", 1.0)
        ),
        format!(
            "AllowAnyoneBabyImprintCuddle={}",
            ini_bool(bool_value(config, "allowAnyoneBabyImprintCuddle", false))
        ),
        format!(
            "FastDecayUnsnappedCoreStructures={}",
            ini_bool(bool_value(
                config,
                "fastDecayUnsnappedCoreStructures",
                false
            ))
        ),
        format!(
            "AllowCryoFridgeOnSaddle={}",
            ini_bool(bool_value(config, "allowCryoFridgeOnSaddle", false))
        ),
        format!(
            "DisableCryopodEnemyCheck={}",
            ini_bool(bool_value(config, "disableCryopodEnemyCheck", false))
        ),
        format!(
            "DisableCryopodFridgeRequirement={}",
            ini_bool(bool_value(config, "disableCryopodFridgeRequirement", false))
        ),
        format!(
            "DisableCryopodCooldown={}",
            ini_bool(bool_value(config, "disableCryopodCooldown", false))
        ),
        format!(
            "PreventDiseases={}",
            ini_bool(!bool_value(config, "enableDiseases", true))
        ),
        format!(
            "NonPermanentDiseases={}",
            ini_bool(bool_value(config, "nonPermanentDiseases", false))
        ),
        format!(
            "TribeNameChangeCooldown={}",
            number_u32(config, "tribeNameChangeCooldown", 15)
        ),
        format!(
            "AdminLogging={}",
            ini_bool(bool_value(config, "adminLogging", true))
        ),
        format!(
            "ChatLogging={}",
            ini_bool(bool_value(config, "chatLogging", true))
        ),
    ];

    if is_aberration(instance) {
        lines.push(format!(
            "CrossARKAllowForeignDinoDownloads={}",
            ini_bool(bool_value(
                config,
                "crossArkAllowForeignDinoDownloads",
                false
            ))
        ));
    }
    if is_lost_colony(instance) {
        lines.extend([
            format!(
                "LimitBunkersPerTribe={}",
                ini_bool(bool_value(config, "limitBunkersPerTribe", true))
            ),
            format!(
                "AllowBunkersInPreventionZones={}",
                ini_bool(bool_value(config, "allowBunkersInPreventionZones", false))
            ),
            format!(
                "AllowRidingDinosInsideBunkers={}",
                ini_bool(bool_value(config, "allowRidingDinosInsideBunkers", true))
            ),
            format!(
                "AllowBunkerModulesAboveGround={}",
                ini_bool(bool_value(config, "allowBunkerModulesAboveGround", false))
            ),
            format!(
                "AllowDinoAIInsideBunkers={}",
                ini_bool(bool_value(config, "allowDinoAIInsideBunkers", true))
            ),
            format!(
                "AllowBunkerModulesInPreventionZones={}",
                ini_bool(bool_value(
                    config,
                    "allowBunkerModulesInPreventionZones",
                    false
                ))
            ),
            format!(
                "LimitBunkersPerTribeNum={}",
                number_u32(config, "limitBunkersPerTribeNum", 3)
            ),
            format!(
                "MinDistanceBetweenBunkers={}",
                number_f64(config, "minDistanceBetweenBunkers", 3000.0)
            ),
            format!(
                "EnemyAccessBunkerHPThreshold={}",
                number_f64(config, "enemyAccessBunkerHPThreshold", 0.25)
            ),
            format!(
                "BunkerUnderHPThresholdDmgMultiplier={}",
                number_f64(config, "bunkerUnderHPThresholdDmgMultiplier", 0.05)
            ),
        ]);
    }
    lines.extend([
        format!("ActiveMods={active_mods}"),
        String::new(),
        "[SessionSettings]".to_string(),
        format!(
            "SessionName={}",
            text(config, "sessionName", &instance.name)
        ),
        format!(
            "Port={}",
            number_u16(config, "gamePort", instance.game_port)
        ),
        format!(
            "QueryPort={}",
            number_u16(config, "queryPort", instance.query_port)
        ),
    ]);
    lines.push(String::new());
    lines.join("\r\n")
}

fn render_game_ini(config: &Value) -> String {
    let mut lines = vec![
        "[/Script/ShooterGame.ShooterGameMode]".to_string(),
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
            "ResourceNoReplenishRadiusPlayers={}",
            number_f64(config, "resourceNoReplenishRadiusPlayers", 1.0)
        ),
        format!(
            "ResourceNoReplenishRadiusStructures={}",
            number_f64(config, "resourceNoReplenishRadiusStructures", 1.0)
        ),
        format!(
            "CropGrowthSpeedMultiplier={}",
            number_f64(config, "cropGrowthSpeedMultiplier", 1.0)
        ),
        format!(
            "CropDecaySpeedMultiplier={}",
            number_f64(config, "cropDecaySpeedMultiplier", 1.0)
        ),
        format!(
            "SupplyCrateLootQualityMultiplier={}",
            number_f64(config, "supplyCrateLootQualityMultiplier", 1.0)
        ),
        format!(
            "FishingLootQualityMultiplier={}",
            number_f64(config, "fishingLootQualityMultiplier", 1.0)
        ),
        format!(
            "FuelConsumptionIntervalMultiplier={}",
            number_f64(config, "fuelConsumptionIntervalMultiplier", 1.0)
        ),
        format!(
            "GlobalSpoilingTimeMultiplier={}",
            number_f64(config, "globalSpoilingTimeMultiplier", 1.0)
        ),
        format!(
            "GlobalItemDecompositionTimeMultiplier={}",
            number_f64(config, "globalItemDecompositionTimeMultiplier", 1.0)
        ),
        format!(
            "GlobalCorpseDecompositionTimeMultiplier={}",
            number_f64(config, "globalCorpseDecompositionTimeMultiplier", 1.0)
        ),
        format!(
            "bDisableStructurePlacementCollision={}",
            ini_bool(bool_value(config, "disablePlacementCollision", false))
        ),
        format!(
            "bAllowSpeedLeveling={}",
            ini_bool(bool_value(config, "allowSpeedLeveling", false))
        ),
        format!(
            "MaxNumberOfPlayersInTribe={}",
            number_u32(config, "maxTribeSize", 0)
        ),
        format!(
            "StructureDamageRepairCooldown={}",
            number_u32(config, "structureDamageRepairCooldown", 180)
        ),
        format!(
            "LimitGeneratorsNum={}",
            number_u32(config, "limitGeneratorsNum", 3)
        ),
        format!(
            "LimitGeneratorsRange={}",
            number_u32(config, "limitGeneratorsRange", 15000)
        ),
        format!(
            "MaxAlliancesPerTribe={}",
            number_u32(config, "maxAlliancesPerTribe", 0)
        ),
        format!(
            "MaxTribesPerAlliance={}",
            number_u32(config, "maxTribesPerAlliance", 0)
        ),
        format!(
            "bPvEAllowTribeWar={}",
            ini_bool(bool_value(config, "tribeAlliances", true))
        ),
        format!(
            "bPvEDisableFriendlyFire={}",
            ini_bool(bool_value(config, "disableFriendlyFire", false))
        ),
    ];

    lines.push(String::new());
    lines.join("\r\n")
}

fn render_engine_ini(config: &Value) -> String {
    let lines = [
        "[/Script/OnlineSubsystemUtils.IpNetDriver]".to_string(),
        format!(
            "NetServerMaxTickRate={}",
            number_u32(config, "networkTickRate", 30)
        ),
        format!(
            "MaxClientRate={}",
            number_u32(config, "maxClientRate", 100000)
        ),
        String::new(),
    ];
    lines.join("\r\n")
}

fn is_aberration(instance: &ServerInstance) -> bool {
    instance.map_code == "Aberration_WP"
}

fn is_lost_colony(instance: &ServerInstance) -> bool {
    instance.map_code == "LostColony_WP"
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
            "enableIdlePlayerKick": true,
            "tributeCharacterExpirationSeconds": 3600,
            "tributeDinoExpirationSeconds": 7200,
            "tributeItemExpirationSeconds": 1800,
            "crossArkAllowForeignDinoDownloads": true,
            "resourceNoReplenishRadiusPlayers": 2.5,
            "resourceNoReplenishRadiusStructures": 3.5,
            "maxTamedDinos": 1234,
            "cropGrowthSpeedMultiplier": 4.5,
            "cropDecaySpeedMultiplier": 5.5,
            "supplyCrateLootQualityMultiplier": 2.25,
            "fishingLootQualityMultiplier": 1.75,
            "fuelConsumptionIntervalMultiplier": 6.5,
            "itemStackSizeMultiplier": 7.5,
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
}
