mod public_view;
mod save;
mod validation;

pub(crate) use public_view::public_settings;
#[cfg(test)]
pub(crate) use save::prepare_settings_for_save;

use crate::{
    app_state::AppRuntime, models::GlobalSettings, reverse_proxy,
    sync_events::SETTINGS_CHANGED_EVENT, web_server, window_controls,
};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_settings(runtime: State<'_, AppRuntime>) -> Result<Value, String> {
    public_settings(runtime.settings()?)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    settings: GlobalSettings,
) -> Result<Value, String> {
    save_settings_for_runtime(&app, runtime.inner(), settings)
}

#[tauri::command]
pub fn list_web_security_bans(app: AppHandle) -> Result<Value, String> {
    list_web_security_bans_for_app(&app)
}

#[tauri::command]
pub fn unban_web_security_ip(app: AppHandle, ip: String) -> Result<Value, String> {
    unban_web_security_ip_for_app(&app, &ip)
}

pub(crate) fn save_settings_for_runtime(
    app: &AppHandle,
    runtime: &AppRuntime,
    settings: GlobalSettings,
) -> Result<Value, String> {
    let settings = save::merge_settings_update(runtime, settings)?;
    validation::validate_settings(&settings)?;
    let saved = runtime.save_settings(settings)?;
    publish_settings_changed(app, saved.clone())?;
    public_settings(saved)
}

pub(crate) fn list_web_security_bans_for_app(app: &AppHandle) -> Result<Value, String> {
    to_json(reverse_proxy::list_security_bans_from_app(app)?)
}

pub(crate) fn unban_web_security_ip_for_app(app: &AppHandle, ip: &str) -> Result<Value, String> {
    to_json(reverse_proxy::unban_security_ip_from_app(app, ip)?)
}

fn to_json<T: Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| format!("序列化 Web API 响应失败：{error}"))
}

fn publish_sync_event<T: Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: T,
) -> Result<(), String> {
    window_controls::publish_settings_changed_and_apply(app, event_name, payload)
}

fn publish_settings_changed(app: &AppHandle, settings: GlobalSettings) -> Result<(), String> {
    web_server::apply_settings_from_app(app, &settings)?;
    reverse_proxy::apply_settings_from_app(app, &settings)?;
    window_controls::handle_settings_changed(app, &settings);
    publish_sync_event(app, SETTINGS_CHANGED_EVENT, public_settings(settings)?)
}

#[cfg(test)]
mod tests;
