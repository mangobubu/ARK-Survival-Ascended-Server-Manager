use crate::{
    ark_config_mods::active_mod_ids,
    ark_config_values::{
        bool_value, number_u16, number_u32, split_custom_args, text, url_component,
    },
    models::{ModItem, ServerInstance},
};
use serde_json::Value;

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
    // 保留 GameUserSettings.ini 写入作为主配置，同时把采集倍率追加到启动 URL，
    // 让本次启动明确使用同一个倍率值，避免运行时继续沿用旧倍率。
    push_number_url_option(
        &mut map_url,
        config,
        "harvestAmount",
        "HarvestAmountMultiplier",
    );
    push_number_url_option(
        &mut map_url,
        config,
        "harvestHealthMultiplier",
        "HarvestHealthMultiplier",
    );
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
        "-servergamelogincludetribelogs",
    );
    push_flag(
        &mut args,
        config,
        "serverGameLogIncludeTribe",
        "-ServerRCONOutputTribeLogs",
    );
    push_flag(&mut args, config, "destroyWildDinos", "-ForceRespawnDinos");
    push_flag(&mut args, config, "noDinos", "-NoDinos");
    push_flag(&mut args, config, "noWildBabies", "-NoWildBabies");
    push_flag(
        &mut args,
        config,
        "disableCustomCosmetics",
        "-DisableCustomCosmetics",
    );
    push_flag(
        &mut args,
        config,
        "unstasisDinoObstructionCheck",
        "-UnstasisDinoObstructionCheck",
    );
    push_flag(
        &mut args,
        config,
        "useServerNetSpeedCheck",
        "-UseServerNetSpeedCheck",
    );
    push_flag(&mut args, config, "noSound", "-nosound");
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

fn push_flag(args: &mut Vec<String>, config: &Value, key: &str, flag: &str) {
    if bool_value(config, key, false) {
        args.push(flag.to_string());
    }
}

fn push_number_url_option(map_url: &mut String, config: &Value, key: &str, option: &str) {
    if let Some(value) = config.get(key).and_then(Value::as_f64) {
        map_url.push_str(&format!("?{option}={value}"));
    }
}
