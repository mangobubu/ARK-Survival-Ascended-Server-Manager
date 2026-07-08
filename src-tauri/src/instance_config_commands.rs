use crate::{
    app_state::{AppRuntime, normalize_required_rcon_config},
    ark_config,
    command_events::emit_instance_log,
    models::{ModItem, ServerInstance},
    server_version::with_current_server_version,
    sync_events::{INSTANCE_CONFIG_CHANGED_EVENT, INSTANCES_CHANGED_EVENT},
    window_controls, windows_firewall,
};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::{AppHandle, State};

pub(crate) fn get_instance_config_for_runtime(
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<Value, String> {
    runtime.get_config(instance_id)
}

pub(crate) fn get_instance_mods_for_runtime(
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<Vec<ModItem>, String> {
    runtime.get_mods(instance_id)
}

pub(crate) fn save_config_for_runtime(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
    config: Value,
    mods: Vec<ModItem>,
) -> Result<ServerInstance, String> {
    let config = normalize_required_rcon_config(config)?;
    let instance = runtime.save_config_and_mods(instance_id, config.clone(), mods.clone())?;
    let applied = ark_config::apply_instance_config(&instance, &config, &mods)?;
    let firewall_rules = windows_firewall::ensure_instance_firewall_rules(&instance)?;
    runtime.add_log(
        &instance.name,
        "success",
        &format!(
            "配置已保存：{}、{}、{}，配置目录：{}，启动参数 {} 项",
            applied.game_user_settings_path.to_string_lossy(),
            applied.game_ini_path.to_string_lossy(),
            applied.engine_ini_path.to_string_lossy(),
            applied.config_dir.to_string_lossy(),
            applied.launch_arguments.len()
        ),
    )?;
    if !firewall_rules.is_empty() {
        let _ = emit_instance_log(
            app,
            runtime,
            &instance.name,
            "success",
            &format!(
                "Windows 防火墙规则已确认：{}",
                windows_firewall::format_rule_summaries(&firewall_rules)
            ),
        );
    }
    publish_instance_config_changed(app, runtime, instance_id)?;
    publish_instances_changed(app);
    Ok(instance)
}

pub(crate) fn update_instance_mods_for_runtime(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
    mods: Vec<ModItem>,
) -> Result<Vec<ModItem>, String> {
    validate_mods(&mods)?;
    let config = runtime.get_config(instance_id)?;
    runtime.save_config_and_mods(instance_id, config, mods.clone())?;
    publish_instance_config_changed(app, runtime, instance_id)?;
    publish_instances_changed(app);
    Ok(mods)
}

pub(crate) fn check_mod_updates_for_runtime(mods: Vec<ModItem>) -> Result<Vec<ModItem>, String> {
    validate_mods(&mods)?;
    Ok(mods
        .into_iter()
        .map(|mut item| {
            if item.version.trim().is_empty() || item.version == "等待检测" {
                item.version = "本地校验通过".to_string();
            }
            item.update_available = Some(false);
            item
        })
        .collect())
}

#[tauri::command]
pub fn get_instance_config(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<Value, String> {
    get_instance_config_for_runtime(runtime.inner(), &instance_id)
}

#[tauri::command]
pub fn get_instance_mods(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<Vec<ModItem>, String> {
    get_instance_mods_for_runtime(runtime.inner(), &instance_id)
}

#[tauri::command]
pub fn save_instance_config(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    config: Value,
    mods: Vec<ModItem>,
) -> Result<ServerInstance, String> {
    save_config_for_runtime(&app, runtime.inner(), &instance_id, config, mods)
        .map(with_current_server_version)
}

#[tauri::command]
pub fn update_instance_mods(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    mods: Vec<ModItem>,
) -> Result<Vec<ModItem>, String> {
    update_instance_mods_for_runtime(&app, runtime.inner(), &instance_id, mods)
}

#[tauri::command]
pub fn check_mod_updates(mods: Vec<ModItem>) -> Result<Vec<ModItem>, String> {
    check_mod_updates_for_runtime(mods)
}

fn publish_instance_config_changed(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<(), String> {
    let instance = with_current_server_version(runtime.get_instance(instance_id)?);
    let config = runtime.get_config(instance_id)?;
    let mods = runtime.get_mods(instance_id)?;
    publish_sync_event(
        app,
        INSTANCE_CONFIG_CHANGED_EVENT,
        json!({
            "instanceId": instance_id,
            "instance": instance,
            "config": config,
            "mods": mods,
        }),
    )
}

fn publish_instances_changed(app: &AppHandle) {
    publish_sync_event_best_effort(app, INSTANCES_CHANGED_EVENT, json!({}));
}

fn publish_sync_event<T: Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: T,
) -> Result<(), String> {
    window_controls::publish_settings_changed_and_apply(app, event_name, payload)
}

fn publish_sync_event_best_effort<T: Serialize>(app: &AppHandle, event_name: &str, payload: T) {
    let _ = publish_sync_event(app, event_name, payload);
}

fn validate_mods(mods: &[ModItem]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for item in mods {
        if item.id.trim().is_empty() {
            return Err("MOD ID 不能为空".to_string());
        }
        if !item.id.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(format!("MOD ID 只能包含数字：{}", item.id));
        }
        if !seen.insert(item.id.trim().to_string()) {
            return Err(format!("MOD ID 重复：{}", item.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mod_item(id: &str) -> ModItem {
        ModItem {
            id: id.to_string(),
            name: String::new(),
            version: String::new(),
            size: String::new(),
            enabled: true,
            update_available: None,
        }
    }

    #[test]
    fn 模组校验拒绝空_id_非数字和重复项() {
        assert!(validate_mods(&[mod_item("928708")]).is_ok());
        assert!(validate_mods(&[mod_item("")]).is_err());
        assert!(validate_mods(&[mod_item("abc")]).is_err());
        assert!(validate_mods(&[mod_item("928708"), mod_item("928708")]).is_err());
    }

    #[test]
    fn 本地模组检查会填充空版本并标记无更新() {
        let checked = check_mod_updates_for_runtime(vec![mod_item("928708")]).expect("检查模组");

        assert_eq!(checked[0].version, "本地校验通过");
        assert_eq!(checked[0].update_available, Some(false));
    }
}
