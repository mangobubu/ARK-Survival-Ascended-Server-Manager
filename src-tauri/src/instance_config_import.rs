use crate::models::{ImportedServerConfigPreview, ModItem};
use serde_json::{Map, Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

const SERVER_SETTINGS: &[&str] = &["ServerSettings"];
const SESSION_SETTINGS: &[&str] = &["SessionSettings", "ServerSettings"];
const GAME_SESSION_SETTINGS: &[&str] = &["/Script/Engine.GameSession", "ServerSettings"];
const GAME_MODE_SETTINGS: &[&str] = &["/Script/ShooterGame.ShooterGameMode", ""];
const ENGINE_IP_NET_DRIVER_SETTINGS: &[&str] = &["/Script/OnlineSubsystemUtils.IpNetDriver"];

const MAPS: &[(&str, &str)] = &[
    ("TheIsland_WP", "The Island"),
    ("ScorchedEarth_WP", "Scorched Earth"),
    ("TheCenter_WP", "The Center"),
    ("Aberration_WP", "Aberration"),
    ("Extinction_WP", "Extinction"),
    ("Astraeos_WP", "Astraeos"),
    ("Ragnarok_WP", "Ragnarok"),
    ("Valguero_WP", "Valguero"),
    ("LostColony_WP", "Lost Colony"),
];

#[derive(Default)]
struct IniDocument {
    sections: HashMap<String, HashMap<String, String>>,
    repeated_values: HashMap<String, HashMap<String, Vec<String>>>,
}

impl IniDocument {
    fn get(&self, sections: &[&str], key: &str) -> Option<&str> {
        let key = normalize_ini_name(key);
        sections.iter().find_map(|section| {
            self.sections
                .get(&normalize_ini_name(section))
                .and_then(|values| values.get(&key))
                .map(String::as_str)
        })
    }

    fn get_all(&self, sections: &[&str], key: &str) -> Vec<&str> {
        let key = normalize_ini_name(key);
        sections
            .iter()
            .filter_map(|section| self.repeated_values.get(&normalize_ini_name(section)))
            .filter_map(|values| values.get(&key))
            .flat_map(|values| values.iter().map(String::as_str))
            .collect()
    }
}

pub fn read_server_directory_config(path: &Path) -> Result<ImportedServerConfigPreview, String> {
    if !path.exists() {
        return Err(format!("服务端目录不存在：{}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!(
            "请选择服务端文件夹，而不是文件：{}",
            path.display()
        ));
    }

    let selected_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let config_dir = locate_config_dir(&selected_path);
    let install_path = infer_install_path(&selected_path, &config_dir);
    let game_user_settings_path = config_dir.join("GameUserSettings.ini");
    let game_ini_path = config_dir.join("Game.ini");
    let engine_ini_path = config_dir.join("Engine.ini");

    let mut config = Map::new();
    let mut found_files = Vec::new();
    let mut warnings = Vec::new();
    let mut mods = Vec::new();

    if game_user_settings_path.is_file() {
        let document = parse_ini_file(&game_user_settings_path)?;
        found_files.push(path_text(&game_user_settings_path));
        read_game_user_settings(&document, &mut config, &mut mods);
    } else {
        warnings.push(format!(
            "未找到 GameUserSettings.ini：{}",
            path_text(&game_user_settings_path)
        ));
    }

    if game_ini_path.is_file() {
        let document = parse_ini_file(&game_ini_path)?;
        found_files.push(path_text(&game_ini_path));
        read_game_ini(&document, &mut config);
    } else {
        warnings.push(format!("未找到 Game.ini：{}", path_text(&game_ini_path)));
    }

    if engine_ini_path.is_file() {
        let document = parse_ini_file(&engine_ini_path)?;
        found_files.push(path_text(&engine_ini_path));
        read_engine_ini(&document, &mut config);
    }

    if found_files.is_empty() {
        warnings.push("未在所选目录下找到 ASA 服务端配置文件".to_string());
    }

    let mut preview = ImportedServerConfigPreview {
        install_path: path_text(&install_path),
        name: text_from_config(&config, "sessionName"),
        map: None,
        map_code: None,
        mode: bool_from_config(&config, "pve").map(|pve| {
            if pve {
                "PvE".to_string()
            } else {
                "PvP".to_string()
            }
        }),
        game_port: u16_from_config(&config, "gamePort"),
        query_port: u16_from_config(&config, "queryPort"),
        rcon_port: u16_from_config(&config, "rconPort"),
        max_players: u32_from_config(&config, "maxPlayers"),
        cluster_id: text_from_config(&config, "clusterId"),
        server_password: text_from_config(&config, "serverPassword"),
        admin_password: text_from_config(&config, "adminPassword"),
        config: Value::Object(config),
        mods,
        found_files,
        warnings,
    };

    if preview
        .name
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && !preview.found_files.is_empty()
    {
        preview.name = install_path
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToString::to_string);
    }

    if let Some((map_code, map_name)) = infer_map_from_saved_arks(&install_path) {
        preview.map_code = Some(map_code);
        preview.map = Some(map_name);
    }

    Ok(preview)
}

fn read_game_user_settings(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    mods: &mut Vec<ModItem>,
) {
    if let Some(value) = map_text(
        document,
        config,
        SESSION_SETTINGS,
        "SessionName",
        "sessionName",
    ) {
        config.insert("sessionName".to_string(), json!(value));
    }

    for (ini_key, config_key) in [
        ("ServerPassword", "serverPassword"),
        ("ServerAdminPassword", "adminPassword"),
        ("SpectatorPassword", "spectatorPassword"),
        ("ClusterID", "clusterId"),
        ("ClusterId", "clusterId"),
    ] {
        map_text(document, config, SERVER_SETTINGS, ini_key, config_key);
    }

    for (ini_key, config_key) in [("Port", "gamePort"), ("QueryPort", "queryPort")] {
        map_u16(document, config, SESSION_SETTINGS, ini_key, config_key);
    }

    for (ini_key, config_key) in [("RCONPort", "rconPort")] {
        map_u16(document, config, SERVER_SETTINGS, ini_key, config_key);
    }

    map_u32(
        document,
        config,
        GAME_SESSION_SETTINGS,
        "MaxPlayers",
        "maxPlayers",
    );

    for (ini_key, config_key) in [
        ("MaxTamedDinos", "maxTamedDinos"),
        ("TheMaxStructuresInRange", "structureLimit"),
        ("RCONServerGameLogBuffer", "rconBufferSize"),
        ("KickIdlePlayersPeriod", "kickIdlePlayersPeriod"),
        ("TribeNameChangeCooldown", "tribeNameChangeCooldown"),
        ("LimitBunkersPerTribeNum", "limitBunkersPerTribeNum"),
        (
            "TributeCharacterExpirationSeconds",
            "tributeCharacterExpirationSeconds",
        ),
        (
            "TributeDinoExpirationSeconds",
            "tributeDinoExpirationSeconds",
        ),
        (
            "TributeItemExpirationSeconds",
            "tributeItemExpirationSeconds",
        ),
    ] {
        map_u32(document, config, SERVER_SETTINGS, ini_key, config_key);
    }

    for (ini_key, config_key) in [
        ("RCONEnabled", "rconEnabled"),
        ("ServerPVE", "pve"),
        ("ServerHardcore", "hardcore"),
        ("DisableFriendlyFire", "disableFriendlyFire"),
        ("EnablePVPGamma", "enablePvPGamma"),
        ("AllowHitMarkers", "allowHitMarkers"),
        ("AllowThirdPersonPlayer", "thirdPerson"),
        ("ServerCrosshair", "crosshair"),
        ("ShowMapPlayerLocation", "showMapPlayer"),
        ("AllowFlyerCarryPvE", "flyerCarry"),
        ("AllowFlyingStaminaRecovery", "allowFlyingStaminaRecovery"),
        ("PreventDownloadItems", "preventDownloadItems"),
        ("PreventDownloadDinos", "preventDownloadDinos"),
        ("PreventDownloadSurvivors", "preventDownloadSurvivors"),
        ("PreventUploadItems", "preventUploadItems"),
        ("PreventUploadDinos", "preventUploadDinos"),
        ("PreventUploadSurvivors", "preventUploadSurvivors"),
        ("NoTributeDownloads", "noTributeDownloads"),
        ("AllowCaveBuildingPvE", "allowCaveBuildingPvE"),
        ("AllowCaveBuildingPvP", "allowCaveBuildingPvP"),
        ("EnableIdlePlayerKick", "enableIdlePlayerKick"),
        (
            "AllowAnyoneBabyImprintCuddle",
            "allowAnyoneBabyImprintCuddle",
        ),
        (
            "FastDecayUnsnappedCoreStructures",
            "fastDecayUnsnappedCoreStructures",
        ),
        ("AllowCryoFridgeOnSaddle", "allowCryoFridgeOnSaddle"),
        ("DisableCryopodEnemyCheck", "disableCryopodEnemyCheck"),
        (
            "DisableCryopodFridgeRequirement",
            "disableCryopodFridgeRequirement",
        ),
        ("DisableCryopodCooldown", "disableCryopodCooldown"),
        ("NonPermanentDiseases", "nonPermanentDiseases"),
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
        (
            "CrossARKAllowForeignDinoDownloads",
            "crossArkAllowForeignDinoDownloads",
        ),
        ("ServerAdminLogs", "adminLogging"),
        ("AdminLogging", "adminLogging"),
        ("ChatLogging", "chatLogging"),
    ] {
        map_bool(document, config, SERVER_SETTINGS, ini_key, config_key);
    }
    map_bool_inverted(
        document,
        config,
        SERVER_SETTINGS,
        "PreventDiseases",
        "enableDiseases",
    );

    for (ini_key, config_key) in [
        ("OverrideOfficialDifficulty", "difficulty"),
        ("XPMultiplier", "xpMultiplier"),
        ("TamingSpeedMultiplier", "tamingSpeed"),
        ("HarvestAmountMultiplier", "harvestAmount"),
        ("HarvestHealthMultiplier", "harvestHealthMultiplier"),
        ("PlayerDamageMultiplier", "playerDamageMultiplier"),
        ("PlayerResistanceMultiplier", "playerResistanceMultiplier"),
        ("DinoDamageMultiplier", "dinoDamageMultiplier"),
        ("DinoResistanceMultiplier", "dinoResistanceMultiplier"),
        ("TamedDinoDamageMultiplier", "tamedDinoDamageMultiplier"),
        (
            "TamedDinoResistanceMultiplier",
            "tamedDinoResistanceMultiplier",
        ),
        (
            "PlayerCharacterFoodDrainMultiplier",
            "playerFoodDrainMultiplier",
        ),
        (
            "PlayerCharacterWaterDrainMultiplier",
            "playerWaterDrainMultiplier",
        ),
        (
            "PlayerCharacterStaminaDrainMultiplier",
            "playerStaminaDrainMultiplier",
        ),
        (
            "DinoCharacterFoodDrainMultiplier",
            "dinoFoodDrainMultiplier",
        ),
        (
            "DinoCharacterStaminaDrainMultiplier",
            "dinoStaminaDrainMultiplier",
        ),
        ("DayCycleSpeedScale", "dayCycleSpeed"),
        ("DayTimeSpeedScale", "dayTimeSpeed"),
        ("NightTimeSpeedScale", "nightTimeSpeed"),
        ("ResourcesRespawnPeriodMultiplier", "resourceRespawn"),
        (
            "ResourceNoReplenishRadiusPlayers",
            "resourceNoReplenishRadiusPlayers",
        ),
        (
            "ResourceNoReplenishRadiusStructures",
            "resourceNoReplenishRadiusStructures",
        ),
        ("DinoCountMultiplier", "dinoCount"),
        ("CropGrowthSpeedMultiplier", "cropGrowthSpeedMultiplier"),
        ("CropDecaySpeedMultiplier", "cropDecaySpeedMultiplier"),
        (
            "SupplyCrateLootQualityMultiplier",
            "supplyCrateLootQualityMultiplier",
        ),
        (
            "FishingLootQualityMultiplier",
            "fishingLootQualityMultiplier",
        ),
        (
            "FuelConsumptionIntervalMultiplier",
            "fuelConsumptionIntervalMultiplier",
        ),
        ("ItemStackSizeMultiplier", "itemStackSizeMultiplier"),
        (
            "GlobalSpoilingTimeMultiplier",
            "globalSpoilingTimeMultiplier",
        ),
        (
            "GlobalItemDecompositionTimeMultiplier",
            "globalItemDecompositionTimeMultiplier",
        ),
        (
            "GlobalCorpseDecompositionTimeMultiplier",
            "globalCorpseDecompositionTimeMultiplier",
        ),
        (
            "RaidDinoCharacterFoodDrainMultiplier",
            "raidDinoFoodDrainMultiplier",
        ),
        ("MinimumDinoReuploadInterval", "minimumDinoReuploadInterval"),
        (
            "PerPlatformMaxStructuresMultiplier",
            "platformStructureMultiplier",
        ),
        (
            "EnemyAccessBunkerHPThreshold",
            "enemyAccessBunkerHPThreshold",
        ),
        (
            "BunkerUnderHPThresholdDmgMultiplier",
            "bunkerUnderHPThresholdDmgMultiplier",
        ),
        ("MinDistanceBetweenBunkers", "minDistanceBetweenBunkers"),
        ("StructurePickupHoldDuration", "structurePickupHoldDuration"),
        (
            "StructurePickupTimeAfterPlacement",
            "structurePickupTimeAfterPlacement",
        ),
        (
            "AutoDestroyOldStructuresMultiplier",
            "autoDestroyOldStructuresMultiplier",
        ),
    ] {
        map_f64(document, config, SERVER_SETTINGS, ini_key, config_key);
    }

    if let Some(server_password) = text_from_config(config, "serverPassword") {
        config.insert(
            "visibility".to_string(),
            json!(if server_password.is_empty() {
                "public"
            } else {
                "private"
            }),
        );
    }

    if let Some(active_mods) = document.get(SERVER_SETTINGS, "ActiveMods") {
        *mods = parse_active_mods(active_mods);
    }
}

fn read_game_ini(document: &IniDocument, config: &mut Map<String, Value>) {
    for (ini_key, config_key) in [
        ("MatingIntervalMultiplier", "matingInterval"),
        ("MatingSpeedMultiplier", "matingSpeedMultiplier"),
        ("EggHatchSpeedMultiplier", "eggHatchSpeed"),
        ("BabyMatureSpeedMultiplier", "babyMatureSpeed"),
        ("BabyCuddleIntervalMultiplier", "cuddleInterval"),
        ("BabyFoodConsumptionSpeedMultiplier", "babyFoodConsumption"),
        ("LayEggIntervalMultiplier", "layEggIntervalMultiplier"),
        (
            "BabyCuddleGracePeriodMultiplier",
            "babyCuddleGracePeriodMultiplier",
        ),
        (
            "BabyCuddleLoseImprintQualitySpeedMultiplier",
            "babyCuddleLoseImprintQualitySpeedMultiplier",
        ),
        (
            "BabyImprintingStatScaleMultiplier",
            "babyImprintingStatScaleMultiplier",
        ),
        ("BabyImprintAmountMultiplier", "babyImprintAmountMultiplier"),
        (
            "ResourceNoReplenishRadiusPlayers",
            "resourceNoReplenishRadiusPlayers",
        ),
        (
            "ResourceNoReplenishRadiusStructures",
            "resourceNoReplenishRadiusStructures",
        ),
        ("CropGrowthSpeedMultiplier", "cropGrowthSpeedMultiplier"),
        ("CropDecaySpeedMultiplier", "cropDecaySpeedMultiplier"),
        (
            "SupplyCrateLootQualityMultiplier",
            "supplyCrateLootQualityMultiplier",
        ),
        (
            "FishingLootQualityMultiplier",
            "fishingLootQualityMultiplier",
        ),
        (
            "FuelConsumptionIntervalMultiplier",
            "fuelConsumptionIntervalMultiplier",
        ),
        (
            "GlobalSpoilingTimeMultiplier",
            "globalSpoilingTimeMultiplier",
        ),
        (
            "GlobalItemDecompositionTimeMultiplier",
            "globalItemDecompositionTimeMultiplier",
        ),
        (
            "GlobalCorpseDecompositionTimeMultiplier",
            "globalCorpseDecompositionTimeMultiplier",
        ),
    ] {
        map_f64(document, config, GAME_MODE_SETTINGS, ini_key, config_key);
    }

    for (ini_key, config_key) in [
        ("MaxNumberOfPlayersInTribe", "maxTribeSize"),
        (
            "StructureDamageRepairCooldown",
            "structureDamageRepairCooldown",
        ),
        ("LimitGeneratorsNum", "limitGeneratorsNum"),
        ("LimitGeneratorsRange", "limitGeneratorsRange"),
        ("MaxAlliancesPerTribe", "maxAlliancesPerTribe"),
        ("MaxTribesPerAlliance", "maxTribesPerAlliance"),
    ] {
        map_u32(document, config, GAME_MODE_SETTINGS, ini_key, config_key);
    }

    for (ini_key, config_key) in [
        (
            "bDisableStructurePlacementCollision",
            "disablePlacementCollision",
        ),
        ("bAllowSpeedLeveling", "allowSpeedLeveling"),
        ("bPvEAllowTribeWar", "tribeAlliances"),
        ("bPvEDisableFriendlyFire", "disableFriendlyFire"),
    ] {
        map_bool(document, config, GAME_MODE_SETTINGS, ini_key, config_key);
    }

    let item_stack_overrides = document
        .get_all(GAME_MODE_SETTINGS, "ConfigOverrideItemMaxQuantity")
        .into_iter()
        .filter_map(parse_item_stack_override)
        .collect::<Vec<_>>();
    if !item_stack_overrides.is_empty() {
        config.insert(
            "itemStackOverrides".to_string(),
            Value::Array(item_stack_overrides),
        );
    }
}

fn parse_item_stack_override(value: &str) -> Option<Value> {
    let item_class_string = extract_assignment(value, "ItemClassString")?;
    let max_item_quantity = extract_u32_assignment(value, "MaxItemQuantity").unwrap_or(1);
    let ignore_multiplier = extract_bool_assignment(value, "bIgnoreMultiplier").unwrap_or(true);

    Some(json!({
        "itemClassString": item_class_string,
        "maxItemQuantity": max_item_quantity,
        "ignoreMultiplier": ignore_multiplier,
    }))
}

fn extract_assignment(value: &str, key: &str) -> Option<String> {
    let start = value.find(key)? + key.len();
    let rest = value[start..].trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();

    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        let text = rest[..end].trim();
        return (!text.is_empty()).then(|| text.to_string());
    }

    let end = rest
        .find(|character| matches!(character, ',' | ')' | '('))
        .unwrap_or(rest.len());
    let text = rest[..end].trim().trim_matches('"');
    (!text.is_empty()).then(|| text.to_string())
}

fn extract_u32_assignment(value: &str, key: &str) -> Option<u32> {
    extract_assignment(value, key)?
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
}

fn extract_bool_assignment(value: &str, key: &str) -> Option<bool> {
    match extract_assignment(value, key)?
        .to_ascii_lowercase()
        .as_str()
    {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn read_engine_ini(document: &IniDocument, config: &mut Map<String, Value>) {
    for (ini_key, config_key) in [
        ("NetServerMaxTickRate", "networkTickRate"),
        ("MaxClientRate", "maxClientRate"),
    ] {
        map_u32(
            document,
            config,
            ENGINE_IP_NET_DRIVER_SETTINGS,
            ini_key,
            config_key,
        );
    }
}

fn parse_ini_file(path: &Path) -> Result<IniDocument, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("无法读取配置文件 {}：{error}", path.display()))?;
    Ok(parse_ini(&decode_config_bytes(&bytes)))
}

fn parse_ini(content: &str) -> IniDocument {
    let mut document = IniDocument::default();
    let mut current_section = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = normalize_ini_name(&line[1..line.len() - 1]);
            document
                .sections
                .entry(current_section.clone())
                .or_default();
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = normalize_ini_name(key);
        let value = clean_ini_value(value);
        document
            .sections
            .entry(current_section.clone())
            .or_default()
            .insert(key.clone(), value.clone());
        document
            .repeated_values
            .entry(current_section.clone())
            .or_default()
            .entry(key)
            .or_default()
            .push(value);
    }

    document
}

fn decode_config_bytes(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let values = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16_lossy(&values);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let values = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16_lossy(&values);
    }
    String::from_utf8_lossy(bytes).into_owned()
}

fn locate_config_dir(root: &Path) -> PathBuf {
    candidate_config_dirs_with_ancestors(root)
        .into_iter()
        .find(|path| path.join("GameUserSettings.ini").is_file() || path.join("Game.ini").is_file())
        .or_else(|| search_config_dir(root, 7))
        .unwrap_or_else(|| {
            root.join("ShooterGame")
                .join("Saved")
                .join("Config")
                .join("WindowsServer")
        })
}

fn candidate_config_dirs_with_ancestors(root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut current = Some(root);

    for _ in 0..=5 {
        let Some(path) = current else {
            break;
        };
        candidates.extend(candidate_config_dirs(path));
        current = path.parent();
    }

    dedupe_paths(candidates)
}

fn candidate_config_dirs(root: &Path) -> Vec<PathBuf> {
    vec![
        root.to_path_buf(),
        root.join("WindowsServer"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsServer"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("Win64"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("Windows"),
        root.join("ShooterGame")
            .join("Saved")
            .join("Config")
            .join("WindowsNoEditor"),
        root.join("Saved").join("Config").join("WindowsServer"),
        root.join("Saved").join("Config").join("Win64"),
        root.join("Saved").join("Config").join("Windows"),
        root.join("Saved").join("Config").join("WindowsNoEditor"),
        root.join("Config").join("WindowsServer"),
        root.join("Config").join("Win64"),
        root.join("Config").join("Windows"),
        root.join("Config").join("WindowsNoEditor"),
    ]
}

fn infer_install_path(selected_path: &Path, config_dir: &Path) -> PathBuf {
    if path_tail_matches_config_platform(config_dir, &["shootergame", "saved", "config"]) {
        return ancestor(config_dir, 4).unwrap_or_else(|| selected_path.to_path_buf());
    }

    if path_tail_matches_config_platform(config_dir, &["saved", "config"]) {
        return ancestor(config_dir, 3).unwrap_or_else(|| selected_path.to_path_buf());
    }

    selected_path.to_path_buf()
}

fn search_config_dir(root: &Path, max_depth: usize) -> Option<PathBuf> {
    let mut pending = vec![(root.to_path_buf(), 0_usize)];

    while let Some((directory, depth)) = pending.pop() {
        if directory.join("GameUserSettings.ini").is_file() || directory.join("Game.ini").is_file()
        {
            return Some(directory);
        }
        if depth >= max_depth {
            continue;
        }

        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        let mut children = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        children.sort_by_key(|path| config_search_priority(path));
        for child in children.into_iter().rev() {
            pending.push((child, depth + 1));
        }
    }

    None
}

fn config_search_priority(path: &Path) -> u8 {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match name.as_str() {
        "shootergame" => 0,
        "saved" => 1,
        "config" => 2,
        "windowsserver" => 3,
        "win64" | "windows" | "windowsnoeditor" => 4,
        _ => 9,
    }
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(normalize_path_for_dedupe(path)))
        .collect()
}

fn normalize_path_for_dedupe(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn infer_map_from_saved_arks(root: &Path) -> Option<(String, String)> {
    let save_dirs = [
        root.join("ShooterGame").join("Saved").join("SavedArks"),
        root.join("ShooterGame")
            .join("Saved")
            .join("SavedArksLocal"),
        root.join("Saved").join("SavedArks"),
        root.join("Saved").join("SavedArksLocal"),
    ];

    let mut best_match: Option<(String, String, SystemTime)> = None;
    for directory in save_dirs {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            let lower_name = file_name.to_ascii_lowercase();
            for (code, name) in MAPS {
                if !lower_name.starts_with(&code.to_ascii_lowercase())
                    || !lower_name.ends_with(".ark")
                {
                    continue;
                }
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let should_replace = best_match
                    .as_ref()
                    .map(|(_, _, current)| modified > *current)
                    .unwrap_or(true);
                if should_replace {
                    best_match = Some(((*code).to_string(), (*name).to_string(), modified));
                }
            }
        }
    }

    best_match.map(|(code, name, _)| (code, name))
}

fn map_text(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) -> Option<String> {
    let value = document.get(sections, ini_key)?.to_string();
    config.insert(config_key.to_string(), json!(value));
    Some(value)
}

fn map_bool(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_bool) {
        config.insert(config_key.to_string(), json!(value));
    }
}

fn map_bool_inverted(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_bool) {
        config.insert(config_key.to_string(), json!(!value));
    }
}

fn map_u16(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_u16) {
        config.insert(config_key.to_string(), json!(value));
    }
}

fn map_u32(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_u32) {
        config.insert(config_key.to_string(), json!(value));
    }
}

fn map_f64(
    document: &IniDocument,
    config: &mut Map<String, Value>,
    sections: &[&str],
    ini_key: &str,
    config_key: &str,
) {
    if let Some(value) = document.get(sections, ini_key).and_then(parse_f64) {
        config.insert(config_key.to_string(), json!(value));
    }
}

fn parse_active_mods(value: &str) -> Vec<ModItem> {
    let mut seen = HashSet::new();
    value
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty() && id.chars().all(|ch| ch.is_ascii_digit()))
        .filter(|id| seen.insert((*id).to_string()))
        .map(|id| ModItem {
            id: id.to_string(),
            name: format!("MOD {id}"),
            version: "配置导入".to_string(),
            size: "未知大小".to_string(),
            enabled: true,
            update_available: Some(false),
        })
        .collect()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_u16(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok()
}

fn parse_u32(value: &str) -> Option<u32> {
    value.trim().parse::<u32>().ok()
}

fn parse_f64(value: &str) -> Option<f64> {
    value.trim().parse::<f64>().ok()
}

fn text_from_config(config: &Map<String, Value>, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn bool_from_config(config: &Map<String, Value>, key: &str) -> Option<bool> {
    config.get(key).and_then(Value::as_bool)
}

fn u16_from_config(config: &Map<String, Value>, key: &str) -> Option<u16> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
}

fn u32_from_config(config: &Map<String, Value>, key: &str) -> Option<u32> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn normalize_ini_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn clean_ini_value(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].trim().to_string();
        }
    }
    value.to_string()
}

fn path_tail_matches_config_platform(path: &Path, expected_prefix: &[&str]) -> bool {
    let names = path_tail_names(path, expected_prefix.len() + 1);
    if names.len() != expected_prefix.len() + 1 {
        return false;
    }
    let prefix_matches = names[..expected_prefix.len()]
        .iter()
        .map(String::as_str)
        .eq(expected_prefix.iter().copied());
    prefix_matches && is_config_platform_dir(&names[expected_prefix.len()])
}

fn is_config_platform_dir(name: &str) -> bool {
    matches!(
        name,
        "windowsserver" | "win64" | "windows" | "windowsnoeditor"
    )
}

fn path_tail_names(path: &Path, count: usize) -> Vec<String> {
    let mut names = path
        .iter()
        .rev()
        .take(count)
        .map(|part| part.to_string_lossy().to_ascii_lowercase())
        .collect::<Vec<_>>();
    names.reverse();
    names
}

fn ancestor(path: &Path, levels: usize) -> Option<PathBuf> {
    let mut current = path;
    for _ in 0..levels {
        current = current.parent()?;
    }
    Some(current.to_path_buf())
}

fn path_text(path: &Path) -> String {
    clean_windows_path_text(&path.to_string_lossy())
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

#[cfg(test)]
mod tests {
    use super::*;
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
    fn parses_utf16_le_ini() {
        let text = "\u{feff}[ServerSettings]\r\nSessionName=中文名称\r\n";
        let mut bytes = vec![0xFF, 0xFE];
        for unit in text.encode_utf16().skip(1) {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }

        let document = parse_ini(&decode_config_bytes(&bytes));

        assert_eq!(
            document.get(SESSION_SETTINGS, "SessionName"),
            Some("中文名称")
        );
    }

    #[test]
    fn hides_windows_verbatim_prefix_from_preview_paths() {
        assert_eq!(
            clean_windows_path_text("\\\\?\\D:\\Game\\ASA-SERVER\\ASA-01"),
            "D:\\Game\\ASA-SERVER\\ASA-01"
        );
        assert_eq!(
            clean_windows_path_text("\\\\?\\UNC\\server\\share\\ASA-01"),
            "\\\\server\\share\\ASA-01"
        );
    }
}
