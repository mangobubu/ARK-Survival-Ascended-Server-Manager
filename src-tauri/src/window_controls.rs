use crate::{
    app_state::AppRuntime,
    models::{GlobalSettings, WindowCloseBehavior},
    sync_events::{SETTINGS_CHANGED_EVENT, SyncEventBus},
    MAIN_WINDOW_LABEL,
};
use serde::Serialize;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use tauri::{
    App, AppHandle, Emitter, Manager, Runtime, WebviewWindow, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

const TRAY_ID: &str = "asa-main-tray";
const TRAY_SHOW_ID: &str = "tray-show-main-window";
const TRAY_TOGGLE_ID: &str = "tray-toggle-main-window";
const TRAY_EXIT_ID: &str = "tray-exit-app";

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

    build_tray(app, Arc::clone(&state))?;
    apply_settings(app.handle(), &state, &runtime.settings().unwrap_or_default());

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
                    hide_to_tray(window.app_handle(), state);
                }
                WindowCloseBehavior::ExitApp => {
                    request_full_shutdown(window.app_handle(), state);
                }
            }
        }
        WindowEvent::Destroyed => {
            if !state.is_shutting_down() {
                request_full_shutdown(window.app_handle(), state);
            }
        }
        WindowEvent::Focused(true) => {
            apply_tray_visibility_from_runtime(window.app_handle(), state, runtime);
        }
        _ => {}
    }
}

pub fn request_full_shutdown<R: Runtime>(app_handle: &AppHandle<R>, state: &Arc<WindowControlState>) {
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

fn build_tray(app: &mut App, state: Arc<WindowControlState>) -> tauri::Result<()> {
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

fn apply_settings(app: &AppHandle, state: &Arc<WindowControlState>, settings: &GlobalSettings) {
    state.hotkey.register(
        settings.global_toggle_shortcut_key.clone(),
        app.clone(),
        Arc::clone(state),
    );
    apply_tray_visibility(app, state, settings);
}

fn apply_tray_visibility_from_runtime<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: &Arc<WindowControlState>,
    runtime: &AppRuntime,
) {
    if let Ok(settings) = runtime.settings() {
        apply_tray_visibility(app_handle, state, &settings);
    }
}

fn apply_tray_visibility<R: Runtime>(
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
    if let Ok(tray) = state.tray.lock() {
        if let Some(tray) = tray.as_ref() {
            let _ = tray.set_visible(visible);
        }
    }
}

fn ask_before_close_or_hide<R: Runtime>(window: &WebviewWindow<R>, state: &Arc<WindowControlState>) {
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
                hide_to_tray(&app_handle, &state);
            }
        });
}

fn hide_to_tray<R: Runtime>(app_handle: &AppHandle<R>, state: &Arc<WindowControlState>) {
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

fn toggle_main_window<R: Runtime>(app_handle: &AppHandle<R>, state: &Arc<WindowControlState>) {
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

#[cfg(windows)]
#[derive(Default)]
struct PlatformHotkeyState {
    inner: Mutex<Option<WindowsHotkeyHandle>>,
}

#[cfg(windows)]
impl PlatformHotkeyState {
    fn register(&self, key: String, app: AppHandle, state: Arc<WindowControlState>) {
        let normalized = normalize_shortcut_key(&key);
        let Some(vk) = shortcut_key_to_vk(&normalized) else {
            self.unregister();
            return;
        };

        let mut guard = self.inner.lock().expect("快捷键状态锁已损坏");
        if guard.as_ref().is_some_and(|handle| handle.key == normalized) {
            return;
        }
        if let Some(handle) = guard.take() {
            handle.stop();
        }

        match WindowsHotkeyHandle::spawn(normalized, vk, app, state) {
            Ok(handle) => *guard = Some(handle),
            Err(error) => eprintln!("注册全局快捷键失败：{error}"),
        }
    }

    fn unregister(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(handle) = guard.take() {
                handle.stop();
            }
        }
    }
}

#[cfg(not(windows))]
#[derive(Default)]
struct PlatformHotkeyState;

#[cfg(not(windows))]
impl PlatformHotkeyState {
    fn register(&self, _key: String, _app: AppHandle, _state: Arc<WindowControlState>) {}
    fn unregister(&self) {}
}

#[cfg(windows)]
struct WindowsHotkeyHandle {
    key: String,
    thread_id: u32,
}

#[cfg(windows)]
impl WindowsHotkeyHandle {
    fn spawn(
        key: String,
        vk: u32,
        app: AppHandle,
        state: Arc<WindowControlState>,
    ) -> Result<Self, String> {
        use std::sync::mpsc;
        use windows_sys::Win32::{
            System::Threading::GetCurrentThreadId,
            UI::{
                Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, RegisterHotKey},
                WindowsAndMessaging::{MSG, PM_NOREMOVE, PeekMessageW},
            },
        };

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            unsafe {
                let thread_id = GetCurrentThreadId();
                let mut bootstrap = std::mem::zeroed::<MSG>();
                let _ = PeekMessageW(&mut bootstrap, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
                let modifiers = MOD_CONTROL | MOD_ALT | MOD_NOREPEAT;
                if RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID, modifiers, vk) == 0 {
                    let _ = tx.send(Err("快捷键可能已被其他程序占用".to_string()));
                    return;
                }

                let _ = tx.send(Ok(thread_id));
                hotkey_message_loop(app, state);
            }
        });

        let thread_id = rx
            .recv()
            .map_err(|_| "快捷键监听线程启动失败".to_string())??;
        Ok(Self { key, thread_id })
    }

    fn stop(self) {
        drop(self);
    }
}

#[cfg(windows)]
impl Drop for WindowsHotkeyHandle {
    fn drop(&mut self) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_APP};
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_APP + 1, 0, 0);
        }
    }
}

#[cfg(windows)]
const HOTKEY_ID: i32 = 0x4153;
#[cfg(windows)]
fn hotkey_message_loop(app: AppHandle, state: Arc<WindowControlState>) {
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::UnregisterHotKey,
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_APP, WM_HOTKEY,
        },
    };

    unsafe {
        let mut message = std::mem::zeroed::<MSG>();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            if message.message == WM_HOTKEY && message.wParam == HOTKEY_ID as usize {
                toggle_main_window(&app, &state);
                continue;
            }
            if message.message == WM_APP + 1 {
                break;
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        let _ = UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ID);
    }
}

pub fn normalize_shortcut_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return crate::models::default_global_toggle_shortcut_key();
    }

    let normalized = key
        .trim_start_matches("Key")
        .trim_start_matches("Digit")
        .to_ascii_uppercase();

    match normalized.as_str() {
        "ESC" | "ESCAPE" => "ESC".to_string(),
        "SPACE" | " " => "SPACE".to_string(),
        "ARROWUP" | "UP" => "UP".to_string(),
        "ARROWDOWN" | "DOWN" => "DOWN".to_string(),
        "ARROWLEFT" | "LEFT" => "LEFT".to_string(),
        "ARROWRIGHT" | "RIGHT" => "RIGHT".to_string(),
        value => value.to_string(),
    }
}

pub fn is_supported_shortcut_key(key: &str) -> bool {
    shortcut_key_to_vk(&normalize_shortcut_key(key)).is_some()
}

fn shortcut_key_to_vk(key: &str) -> Option<u32> {
    let key = normalize_shortcut_key(key);
    if key.len() == 1 {
        let ch = key.chars().next()?;
        if ch.is_ascii_alphanumeric() {
            return Some(ch as u32);
        }
    }

    if let Some(number) = key.strip_prefix('F').and_then(|value| value.parse::<u32>().ok()) {
        if (1..=24).contains(&number) {
            return Some(0x70 + number - 1);
        }
    }

    match key.as_str() {
        "SPACE" => Some(0x20),
        "ESC" => Some(0x1B),
        "UP" => Some(0x26),
        "DOWN" => Some(0x28),
        "LEFT" => Some(0x25),
        "RIGHT" => Some(0x27),
        "HOME" => Some(0x24),
        "END" => Some(0x23),
        "PAGEUP" => Some(0x21),
        "PAGEDOWN" => Some(0x22),
        "INSERT" => Some(0x2D),
        "DELETE" => Some(0x2E),
        _ => None,
    }
}

pub fn publish_settings_changed_and_apply<T: Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: T,
) -> Result<(), String> {
    let payload = serde_json::to_value(payload)
        .map_err(|error| format!("序列化同步事件失败：{error}"))?;

    app.emit(event_name, payload.clone())
        .map_err(|error| format!("发送同步事件失败：{error}"))?;

    if event_name == SETTINGS_CHANGED_EVENT {
        if let Ok(settings) = serde_json::from_value::<GlobalSettings>(payload.clone()) {
            handle_settings_changed(app, &settings);
        }
    }

    if let Some(bus) = app.try_state::<SyncEventBus>() {
        bus.publish(event_name, payload);
    }

    Ok(())
}
