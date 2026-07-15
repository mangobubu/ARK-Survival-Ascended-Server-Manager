use crate::{
    instance_config_import_ini::IniDocument,
    instance_config_import_mapping::{map_f64, map_u32},
};
use serde_json::{Map, Value};

use super::super::SERVER_SETTINGS;

pub(super) fn read_numeric_settings(document: &IniDocument, config: &mut Map<String, Value>) {
    read_u32_settings(document, config);
    read_f64_settings(document, config);
}

fn read_u32_settings(document: &IniDocument, config: &mut Map<String, Value>) {
    for (ini_key, config_key) in [
        ("MaxTamedDinos", "maxTamedDinos"),
        ("MaxPersonalTamedDinos", "maxPersonalTamedDinos"),
        ("MaxTamedDinos_SoftTameLimit", "maxTamedDinosSoftTameLimit"),
        (
            "MaxTamedDinos_SoftTameLimit_CountdownForDeletionDuration",
            "maxTamedDinosSoftTameLimitCountdown",
        ),
        ("MaxTributeDinos", "maxTributeDinos"),
        ("MaxTributeItems", "maxTributeItems"),
        ("AutoSavePeriodMinutes", "saveInterval"),
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
}

fn read_f64_settings(document: &IniDocument, config: &mut Map<String, Value>) {
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
        (
            "DinoCharacterHealthRecoveryMultiplier",
            "dinoHealthRecoveryMultiplier",
        ),
        (
            "PlayerCharacterHealthRecoveryMultiplier",
            "playerHealthRecoveryMultiplier",
        ),
        (
            "OxygenSwimSpeedStatMultiplier",
            "oxygenSwimSpeedStatMultiplier",
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
            "PlatformSaddleBuildAreaBoundsMultiplier",
            "platformSaddleBuildAreaBoundsMultiplier",
        ),
        ("PreventOfflinePvPInterval", "preventOfflinePvPInterval"),
        (
            "PvEDinoDecayPeriodMultiplier",
            "pveDinoDecayPeriodMultiplier",
        ),
        (
            "StructureResistanceMultiplier",
            "structureResistanceMultiplier",
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
}
