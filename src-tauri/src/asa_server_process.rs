#[cfg(windows)]
use std::time::Duration;

use tokio::process::Command;
#[cfg(windows)]
use tokio::time::sleep;

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM},
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SW_HIDE, ShowWindow,
    },
};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x0000_0008;
#[cfg(windows)]
const ASA_SERVER_HIDDEN_CREATION_FLAGS: u32 = CREATE_NO_WINDOW | DETACHED_PROCESS;
#[cfg(windows)]
const ASA_SERVER_WINDOW_HIDE_ATTEMPTS: usize = 20;
#[cfg(windows)]
const ASA_SERVER_WINDOW_HIDE_INTERVAL: Duration = Duration::from_millis(250);

#[cfg(windows)]
pub(crate) fn configure_asa_server_hidden_process(command: &mut Command) {
    command.creation_flags(ASA_SERVER_HIDDEN_CREATION_FLAGS);
}

#[cfg(not(windows))]
pub(crate) fn configure_asa_server_hidden_process(_command: &mut Command) {}

#[cfg(windows)]
pub(crate) fn hide_asa_server_windows_after_spawn(pid: u32) {
    tokio::spawn(async move {
        for _ in 0..ASA_SERVER_WINDOW_HIDE_ATTEMPTS {
            hide_windows_for_process(pid);
            sleep(ASA_SERVER_WINDOW_HIDE_INTERVAL).await;
        }
    });
}

#[cfg(not(windows))]
pub(crate) fn hide_asa_server_windows_after_spawn(_pid: u32) {}

#[cfg(windows)]
fn hide_windows_for_process(pid: u32) -> bool {
    struct WindowSearch {
        pid: u32,
        hidden_any: bool,
    }

    unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> windows_sys::core::BOOL {
        if lparam == 0 {
            return 1;
        }

        let state = unsafe { &mut *(lparam as *mut WindowSearch) };
        let mut window_pid = 0_u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut window_pid);
        }

        if window_pid == state.pid && unsafe { IsWindowVisible(hwnd) } != 0 {
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
            state.hidden_any = true;
        }

        1
    }

    let mut state = WindowSearch {
        pid,
        hidden_any: false,
    };

    unsafe {
        EnumWindows(Some(enum_window), &mut state as *mut WindowSearch as LPARAM);
    }

    state.hidden_any
}
