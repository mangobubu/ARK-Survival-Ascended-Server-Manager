use crate::window_controls::{WindowControlState, toggle_main_window};
use std::sync::Arc;
use tauri::AppHandle;

#[cfg(windows)]
use std::{sync::Mutex, thread};

#[cfg(windows)]
pub(crate) struct PlatformHotkeyState {
    handle: Mutex<Option<WindowsHotkeyHandle>>,
}

#[cfg(windows)]
impl PlatformHotkeyState {
    pub(crate) fn register(&self, key: String, app: AppHandle, state: Arc<WindowControlState>) {
        let normalized = normalize_shortcut_key(&key);
        if !is_supported_shortcut_key(&normalized) {
            self.unregister();
            return;
        }

        let Some(vk) = shortcut_key_to_vk(&normalized) else {
            self.unregister();
            return;
        };

        let mut handle = self.handle.lock().expect("快捷键状态锁已损坏");
        if handle.as_ref().map(|item| item.key.as_str()) == Some(normalized.as_str()) {
            return;
        }
        if let Some(old) = handle.take() {
            old.stop();
        }
        match WindowsHotkeyHandle::start(normalized, vk, app, state) {
            Ok(new_handle) => *handle = Some(new_handle),
            Err(error) => eprintln!("{error}"),
        }
    }

    pub(crate) fn unregister(&self) {
        if let Ok(mut handle) = self.handle.lock()
            && let Some(old) = handle.take()
        {
            old.stop();
        }
    }
}

#[cfg(windows)]
impl Default for PlatformHotkeyState {
    fn default() -> Self {
        Self {
            handle: Mutex::new(None),
        }
    }
}

#[cfg(not(windows))]
#[derive(Default)]
pub(crate) struct PlatformHotkeyState;

#[cfg(not(windows))]
impl PlatformHotkeyState {
    pub(crate) fn register(&self, _key: String, _app: AppHandle, _state: Arc<WindowControlState>) {}

    pub(crate) fn unregister(&self) {}
}

#[cfg(windows)]
struct WindowsHotkeyHandle {
    key: String,
    thread_id: u32,
}

#[cfg(windows)]
impl WindowsHotkeyHandle {
    fn start(
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
        thread::spawn(move || unsafe {
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

    if let Some(number) = key
        .strip_prefix('F')
        .and_then(|value| value.parse::<u32>().ok())
        && (1..=24).contains(&number)
    {
        return Some(0x70 + number - 1);
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
