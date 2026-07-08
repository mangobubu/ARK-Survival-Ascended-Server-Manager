use crate::{
    ark_config_values::{bool_value, ini_bool, number_f64, number_u32},
    models::ServerInstance,
};
use serde_json::Value;

pub(super) fn append_map_specific_settings(
    lines: &mut Vec<String>,
    instance: &ServerInstance,
    config: &Value,
) {
    if is_aberration(instance) {
        append_aberration_settings(lines, config);
    }
    if is_lost_colony(instance) {
        append_lost_colony_settings(lines, config);
    }
}

fn append_aberration_settings(lines: &mut Vec<String>, config: &Value) {
    lines.push(format!(
        "CrossARKAllowForeignDinoDownloads={}",
        ini_bool(bool_value(
            config,
            "crossArkAllowForeignDinoDownloads",
            false
        ))
    ));
}

fn append_lost_colony_settings(lines: &mut Vec<String>, config: &Value) {
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

fn is_aberration(instance: &ServerInstance) -> bool {
    instance.map_code == "Aberration_WP"
}

fn is_lost_colony(instance: &ServerInstance) -> bool {
    instance.map_code == "LostColony_WP"
}
