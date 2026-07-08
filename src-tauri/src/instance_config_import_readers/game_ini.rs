use crate::{
    instance_config_import_ini::IniDocument,
    instance_config_import_mapping::{map_bool, map_f64, map_u32, parse_item_stack_override},
};
use serde_json::{Map, Value};

use super::GAME_MODE_SETTINGS;
pub(crate) fn read_game_ini(document: &IniDocument, config: &mut Map<String, Value>) {
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
