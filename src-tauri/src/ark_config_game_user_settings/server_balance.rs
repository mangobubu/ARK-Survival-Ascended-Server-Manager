use crate::ark_config_values::{bool_value, ini_bool, number_f64, number_u32};
use serde_json::Value;

pub(super) fn append_balance_server_settings(lines: &mut Vec<String>, config: &Value) {
    lines.extend([
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
            "DinoCharacterHealthRecoveryMultiplier={}",
            number_f64(config, "dinoHealthRecoveryMultiplier", 1.0)
        ),
        format!(
            "PlayerCharacterHealthRecoveryMultiplier={}",
            number_f64(config, "playerHealthRecoveryMultiplier", 1.0)
        ),
        format!(
            "OxygenSwimSpeedStatMultiplier={}",
            number_f64(config, "oxygenSwimSpeedStatMultiplier", 1.0)
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
            "MaxPersonalTamedDinos={}",
            number_u32(config, "maxPersonalTamedDinos", 0)
        ),
        format!(
            "MaxTamedDinos_SoftTameLimit={}",
            number_u32(config, "maxTamedDinosSoftTameLimit", 5000)
        ),
        format!(
            "MaxTamedDinos_SoftTameLimit_CountdownForDeletionDuration={}",
            number_u32(config, "maxTamedDinosSoftTameLimitCountdown", 604800)
        ),
        format!(
            "MaxTributeDinos={}",
            number_u32(config, "maxTributeDinos", 20).clamp(20, 273)
        ),
        format!(
            "MaxTributeItems={}",
            number_u32(config, "maxTributeItems", 50).clamp(50, 154)
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
            "PlatformSaddleBuildAreaBoundsMultiplier={}",
            number_f64(config, "platformSaddleBuildAreaBoundsMultiplier", 1.0)
        ),
        format!(
            "PreventOfflinePvPInterval={}",
            number_f64(config, "preventOfflinePvPInterval", 0.0)
        ),
        format!(
            "PvEDinoDecayPeriodMultiplier={}",
            number_f64(config, "pveDinoDecayPeriodMultiplier", 1.0)
        ),
        format!(
            "StructureResistanceMultiplier={}",
            number_f64(config, "structureResistanceMultiplier", 1.0)
        ),
        format!(
            "TheMaxStructuresInRange={}",
            number_u32(config, "structureLimit", 10_500)
        ),
        format!(
            "AlwaysAllowStructurePickup={}",
            ini_bool(bool_value(config, "alwaysAllowStructurePickup", false))
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
    ]);
}
