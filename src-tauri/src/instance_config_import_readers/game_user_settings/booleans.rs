use crate::{
    instance_config_import_ini::IniDocument,
    instance_config_import_mapping::{map_bool, map_bool_inverted},
};
use serde_json::{Map, Value};

use super::super::SERVER_SETTINGS;

pub(super) fn read_boolean_settings(document: &IniDocument, config: &mut Map<String, Value>) {
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
        (
            "PvEAllowStructuresAtSupplyDrops",
            "pveAllowStructuresAtSupplyDrops",
        ),
        (
            "EnableExtraStructurePreventionVolumes",
            "enableExtraStructurePreventionVolumes",
        ),
        ("AlwaysAllowStructurePickup", "alwaysAllowStructurePickup"),
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
}
