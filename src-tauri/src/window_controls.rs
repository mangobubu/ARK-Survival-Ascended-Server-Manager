mod theme;
mod tray;

pub use crate::window_hotkey::is_supported_shortcut_key;
use crate::{
    MAIN_WINDOW_LABEL,
    app_state::AppRuntime,
    models::{GlobalSettings, WindowCloseBehavior},
    sync_events::{SETTINGS_CHANGED_EVENT, SyncEventBus},
    window_hotkey::PlatformHotkeyState,
};
use serde::Serialize;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tauri::{
    App, AppHandle, Emitter, Manager, Runtime, WebviewWindow, WindowEvent, tray::TrayIcon,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
pub(crate) use tray::toggle_main_window;

pub struct WindowControlState {
    tray: Mutex<Option<TrayIcon>>,
    hotkey: PlatformHotkeyState,
    is_shutting_down: AtomicBool,
}

impl Default for WindowControlState {
    fn default() -> Self {
        Self {
            tray: Mutex::new(None),
            hotkey: PlatformHotkeyState::default(),
            is_shutting_down: AtomicBool::new(false),
        }
    }
}

impl WindowControlState {
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::SeqCst)
    }
}

pub fn setup_window_controls(app: &mut App, runtime: &AppRuntime) -> tauri::Result<()> {
    let state = Arc::new(WindowControlState::default());
    app.manage(Arc::clone(&state));

    tray::build_tray(app, Arc::clone(&state))?;
    apply_settings(
        app.handle(),
        &state,
        &runtime.settings().unwrap_or_default(),
    );

    Ok(())
}

pub fn handle_main_window_event<R: Runtime>(
    window: &WebviewWindow<R>,
    event: &WindowEvent,
    state: &Arc<WindowControlState>,
    runtime: &AppRuntime,
) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            if state.is_shutting_down() {
                return;
            }

            let behavior = runtime
                .settings()
                .map(|settings| settings.window_close_behavior)
                .unwrap_or_default();

            match behavior {
                WindowCloseBehavior::AskEveryTime => {
                    api.prevent_close();
                    ask_before_close_or_hide(window, state);
                }
                WindowCloseBehavior::MinimizeToTray => {
                    api.prevent_close();
                    tray::hide_to_tray(window.app_handle(), state);
                }
                WindowCloseBehavior::ExitApp => {
                    request_full_shutdown(window.app_handle(), state);
                }
            }
        }
        WindowEvent::Destroyed if !state.is_shutting_down() => {
            request_full_shutdown(window.app_handle(), state);
        }
        WindowEvent::Focused(true) => {
            tray::apply_tray_visibility_from_runtime(window.app_handle(), state, runtime);
        }
        _ => {}
    }
}

pub fn request_full_shutdown<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: &Arc<WindowControlState>,
) {
    if state.is_shutting_down.swap(true, Ordering::SeqCst) {
        return;
    }

    state.hotkey.unregister();

    for (label, window) in app_handle.webview_windows() {
        if label != MAIN_WINDOW_LABEL {
            let _ = window.close();
        }
    }

    app_handle.exit(0);
}

pub fn handle_settings_changed(app: &AppHandle, settings: &GlobalSettings) {
    let Some(state) = app.try_state::<Arc<WindowControlState>>() else {
        return;
    };
    apply_settings(app, &state, settings);
}

fn apply_settings(app: &AppHandle, state: &Arc<WindowControlState>, settings: &GlobalSettings) {
    theme::apply_window_theme_preference(app, settings);
    state.hotkey.register(
        settings.global_toggle_shortcut_key.clone(),
        app.clone(),
        Arc::clone(state),
    );
    tray::apply_tray_visibility(app, state, settings);
}

fn ask_before_close_or_hide<R: Runtime>(
    window: &WebviewWindow<R>,
    state: &Arc<WindowControlState>,
) {
    let app_handle = window.app_handle().clone();
    let state = Arc::clone(state);
    window
        .dialog()
        .message("要退出应用，还是最小化到托盘继续后台管理服务器？")
        .title("关闭窗口行为")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "退出应用".to_string(),
            "最小化到托盘".to_string(),
        ))
        .show(move |should_exit| {
            if should_exit {
                request_full_shutdown(&app_handle, &state);
            } else {
                tray::hide_to_tray(&app_handle, &state);
            }
        });
}

pub fn publish_settings_changed_and_apply<T: Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: T,
) -> Result<(), String> {
    let payload =
        serde_json::to_value(payload).map_err(|error| format!("序列化同步事件失败：{error}"))?;

    app.emit(event_name, payload.clone())
        .map_err(|error| format!("发送同步事件失败：{error}"))?;

    if event_name == SETTINGS_CHANGED_EVENT
        && let Ok(settings) = serde_json::from_value::<GlobalSettings>(payload.clone())
    {
        handle_settings_changed(app, &settings);
    }

    if let Some(bus) = app.try_state::<SyncEventBus>() {
        bus.publish(event_name, payload);
    }

    Ok(())
}
