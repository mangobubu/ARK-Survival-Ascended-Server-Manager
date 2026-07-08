use crate::ark_config_values::{bool_value, ini_bool, number_f64, number_u32};
use serde_json::Value;

pub(crate) fn render_game_ini(config: &Value) -> String {
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

    lines.extend(render_item_stack_override_lines(config));
    lines.push(String::new());
    lines.join("\r\n")
}

fn render_item_stack_override_lines(config: &Value) -> Vec<String> {
    config
        .get("itemStackOverrides")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let item_class_string = item
                .get("itemClassString")
                .and_then(Value::as_str)
                .map(clean_item_class_string)
                .unwrap_or_default();
            if item_class_string.is_empty() {
                return None;
            }

            let max_item_quantity = positive_u32(item.get("maxItemQuantity"), 1);
            let ignore_multiplier = item
                .get("ignoreMultiplier")
                .and_then(Value::as_bool)
                .unwrap_or(true);

            Some(format!(
                "ConfigOverrideItemMaxQuantity=(ItemClassString=\"{}\",Quantity=(MaxItemQuantity={},bIgnoreMultiplier={}))",
                item_class_string,
                max_item_quantity,
                ini_bool(ignore_multiplier)
            ))
        })
        .collect()
}

fn clean_item_class_string(value: &str) -> String {
    value.trim().trim_matches('"').replace('"', "")
}

fn positive_u32(value: Option<&Value>, fallback: u32) -> u32 {
    value
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_f64().map(|number| number.trunc().max(0.0) as u64))
        })
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .unwrap_or(fallback)
}

pub(crate) fn render_engine_ini(config: &Value) -> String {
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
