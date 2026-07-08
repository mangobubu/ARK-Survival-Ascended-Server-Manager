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
