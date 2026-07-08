use super::{WindowControlState, request_full_shutdown};
use crate::{MAIN_WINDOW_LABEL, app_state::AppRuntime, models::GlobalSettings};
use std::sync::Arc;
use tauri::{
    App, AppHandle, Manager, Runtime,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

const TRAY_ID: &str = "asa-main-tray";
const TRAY_SHOW_ID: &str = "tray-show-main-window";
const TRAY_TOGGLE_ID: &str = "tray-toggle-main-window";
const TRAY_EXIT_ID: &str = "tray-exit-app";

pub(super) fn build_tray(app: &mut App, state: Arc<WindowControlState>) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, TRAY_SHOW_ID, "显示主窗口", true, None::<&str>)?;
    let toggle_item = MenuItem::with_id(app, TRAY_TOGGLE_ID, "呼出/最小化", true, None::<&str>)?;
    let exit_item = MenuItem::with_id(app, TRAY_EXIT_ID, "退出应用", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &toggle_item, &exit_item])?;
    let icon = app.default_window_icon().cloned();

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("ARK: Survival Ascended 服务器管理器")
        .on_menu_event({
            let state = Arc::clone(&state);
            move |app_handle, event| match event.id().as_ref() {
                TRAY_SHOW_ID => show_main_window(app_handle),
                TRAY_TOGGLE_ID => toggle_main_window(app_handle, &state),
                TRAY_EXIT_ID => request_full_shutdown(app_handle, &state),
                _ => {}
            }
        })
        .on_tray_icon_event({
            let state = Arc::clone(&state);
            move |tray, event| {
                if matches!(
                    event,
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    }
                ) {
                    toggle_main_window(tray.app_handle(), &state);
                }
            }
        });

    let tray = if let Some(icon) = icon {
        tray.icon(icon).build(app)?
    } else {
        tray.build(app)?
    };
    *state.tray.lock().expect("托盘状态锁已损坏") = Some(tray);
    Ok(())
}

pub(super) fn apply_tray_visibility_from_runtime<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: &Arc<WindowControlState>,
    runtime: &AppRuntime,
) {
    if let Ok(settings) = runtime.settings() {
        apply_tray_visibility(app_handle, state, &settings);
    }
}

pub(super) fn apply_tray_visibility<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: &Arc<WindowControlState>,
    settings: &GlobalSettings,
) {
    let should_hide = settings.hide_tray_icon_when_minimized
        && app_handle
            .get_webview_window(MAIN_WINDOW_LABEL)
            .and_then(|window| window.is_visible().ok().map(|visible| !visible))
            .unwrap_or(false);
    set_tray_visible(state, !should_hide);
}

fn set_tray_visible(state: &Arc<WindowControlState>, visible: bool) {
    if let Ok(tray) = state.tray.lock()
        && let Some(tray) = tray.as_ref()
    {
        let _ = tray.set_visible(visible);
    }
}

pub(super) fn hide_to_tray<R: Runtime>(app_handle: &AppHandle<R>, state: &Arc<WindowControlState>) {
    if let Some(window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.hide();
    }

    if let Some(runtime) = app_handle.try_state::<AppRuntime>() {
        apply_tray_visibility(app_handle, state, &runtime.settings().unwrap_or_default());
    }
}

fn show_main_window<R: Runtime>(app_handle: &AppHandle<R>) {
    if let Some(window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    if let Some(state) = app_handle.try_state::<Arc<WindowControlState>>() {
        set_tray_visible(&state, true);
    }
}

pub(crate) fn toggle_main_window<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: &Arc<WindowControlState>,
) {
    let Some(window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };

    let visible = window.is_visible().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(false);
    if visible && !minimized {
        hide_to_tray(app_handle, state);
    } else {
        show_main_window(app_handle);
    }
}
