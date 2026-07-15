use crate::{
    ark_config_values::{bool_value, ini_bool, number_f64, number_u16, number_u32, text},
    models::ServerInstance,
};
use serde_json::Value;

struct TransferRules {
    prevent_download_items: bool,
    prevent_download_dinos: bool,
    prevent_download_survivors: bool,
    prevent_upload_items: bool,
    prevent_upload_dinos: bool,
    prevent_upload_survivors: bool,
    no_tribute_downloads: bool,
}

impl TransferRules {
    fn from_config(config: &Value) -> Self {
        let cross_transfer = bool_value(config, "crossTransfer", true);
        Self {
            prevent_download_items: !cross_transfer
                || bool_value(config, "preventDownloadItems", false),
            prevent_download_dinos: !cross_transfer
                || bool_value(config, "preventDownloadDinos", false),
            prevent_download_survivors: !cross_transfer
                || bool_value(config, "preventDownloadSurvivors", false),
            prevent_upload_items: !cross_transfer
                || bool_value(config, "preventUploadItems", false),
            prevent_upload_dinos: !cross_transfer
                || bool_value(config, "preventUploadDinos", false),
            prevent_upload_survivors: !cross_transfer
                || bool_value(config, "preventUploadSurvivors", false),
            no_tribute_downloads: !cross_transfer
                || bool_value(config, "noTributeDownloads", false),
        }
    }
}

pub(super) fn append_basic_server_settings(
    lines: &mut Vec<String>,
    instance: &ServerInstance,
    config: &Value,
) {
    let transfer = TransferRules::from_config(config);
    lines.extend([
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
            "AutoSavePeriodMinutes={}",
            number_u32(config, "saveInterval", 15)
        ),
        format!(
            "DisableStructureDecayPvE={}",
            ini_bool(!bool_value(config, "pveStructureDecay", false))
        ),
        format!(
            "PreventTribeAlliances={}",
            ini_bool(!bool_value(config, "tribeAlliances", true))
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
            "AllowHideDamageSourceFromLogs={}",
            ini_bool(bool_value(config, "allowHideDamageSourceFromLogs", true))
        ),
        format!(
            "AllowMultipleAttachedC4={}",
            ini_bool(bool_value(config, "allowMultipleAttachedC4", false))
        ),
        format!(
            "AllowRaidDinoFeeding={}",
            ini_bool(bool_value(config, "allowRaidDinoFeeding", false))
        ),
        format!(
            "ClampItemSpoilingTimes={}",
            ini_bool(bool_value(config, "clampItemSpoilingTimes", false))
        ),
        format!(
            "ClampResourceHarvestDamage={}",
            ini_bool(bool_value(config, "clampResourceHarvestDamage", false))
        ),
        format!(
            "DestroyTamesOverTheSoftTameLimit={}",
            ini_bool(bool_value(config, "destroyTamesOverSoftTameLimit", false))
        ),
        format!(
            "DisableImprintDinoBuff={}",
            ini_bool(bool_value(config, "disableImprintDinoBuff", false))
        ),
        format!(
            "ForceAllStructureLocking={}",
            ini_bool(bool_value(config, "forceAllStructureLocking", false))
        ),
        format!(
            "GlobalVoiceChat={}",
            ini_bool(bool_value(config, "globalVoiceChat", false))
        ),
        format!(
            "PreventMateBoost={}",
            ini_bool(bool_value(config, "preventMateBoost", false))
        ),
        format!(
            "PreventOfflinePvP={}",
            ini_bool(bool_value(config, "preventOfflinePvP", false))
        ),
        format!(
            "PreventSpawnAnimations={}",
            ini_bool(bool_value(config, "preventSpawnAnimations", false))
        ),
        format!(
            "ProximityChat={}",
            ini_bool(bool_value(config, "proximityChat", false))
        ),
        format!(
            "DisableDinoDecayPvE={}",
            ini_bool(!bool_value(config, "pveDinoDecay", true))
        ),
        format!(
            "PvPDinoDecay={}",
            ini_bool(bool_value(config, "pvpDinoDecay", false))
        ),
        format!(
            "RandomSupplyCratePoints={}",
            ini_bool(bool_value(config, "randomSupplyCratePoints", false))
        ),
        format!(
            "ServerForceNoHUD={}",
            ini_bool(bool_value(config, "serverForceNoHud", false))
        ),
        format!(
            "ShowFloatingDamageText={}",
            ini_bool(bool_value(config, "showFloatingDamageText", false))
        ),
        format!(
            "DontAlwaysNotifyPlayerJoined={}",
            ini_bool(!bool_value(config, "showPlayerJoinNotifications", true))
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
            "PvEAllowStructuresAtSupplyDrops={}",
            ini_bool(bool_value(config, "pveAllowStructuresAtSupplyDrops", false))
        ),
        format!(
            "EnableExtraStructurePreventionVolumes={}",
            ini_bool(bool_value(
                config,
                "enableExtraStructurePreventionVolumes",
                false
            ))
        ),
        format!(
            "KickIdlePlayersPeriod={}",
            number_u32(config, "kickIdlePlayersPeriod", 3600)
        ),
        format!(
            "PreventDownloadItems={}",
            ini_bool(transfer.prevent_download_items)
        ),
        format!(
            "PreventDownloadDinos={}",
            ini_bool(transfer.prevent_download_dinos)
        ),
        format!(
            "PreventDownloadSurvivors={}",
            ini_bool(transfer.prevent_download_survivors)
        ),
        format!(
            "PreventUploadItems={}",
            ini_bool(transfer.prevent_upload_items)
        ),
        format!(
            "PreventUploadDinos={}",
            ini_bool(transfer.prevent_upload_dinos)
        ),
        format!(
            "PreventUploadSurvivors={}",
            ini_bool(transfer.prevent_upload_survivors)
        ),
        format!(
            "NoTributeDownloads={}",
            ini_bool(transfer.no_tribute_downloads)
        ),
        "DifficultyOffset=1".to_string(),
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
    ]);
}
