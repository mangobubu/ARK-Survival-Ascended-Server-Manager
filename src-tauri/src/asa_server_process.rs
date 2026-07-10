#[cfg(windows)]
use std::sync::{Mutex, OnceLock};
#[cfg(windows)]
use std::time::Duration;

use tokio::process::Command;
#[cfg(windows)]
use tokio::time::sleep;

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM},
    System::Diagnostics::Debug::{
        GetErrorMode, SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SEM_NOOPENFILEERRORBOX,
        SetErrorMode,
    },
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
static ASA_SERVER_SPAWN_ERROR_MODE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
#[cfg(windows)]
const ASA_SERVER_SUPPRESSED_ERROR_MODE_FLAGS: u32 =
    SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX;

#[cfg(windows)]
struct ProcessErrorModeGuard {
    previous_error_mode: u32,
}

#[cfg(windows)]
impl ProcessErrorModeGuard {
    fn suppress_asa_server_system_dialogs() -> Self {
        let previous_error_mode = unsafe { GetErrorMode() };
        let inherited_error_mode =
            asa_server_error_mode_with_suppressed_dialogs(previous_error_mode);

        unsafe {
            SetErrorMode(inherited_error_mode);
        }

        Self {
            previous_error_mode,
        }
    }
}

#[cfg(windows)]
impl Drop for ProcessErrorModeGuard {
    fn drop(&mut self) {
        unsafe {
            SetErrorMode(self.previous_error_mode);
        }
    }
}

#[cfg(windows)]
fn asa_server_error_mode_with_suppressed_dialogs(previous_error_mode: u32) -> u32 {
    previous_error_mode | ASA_SERVER_SUPPRESSED_ERROR_MODE_FLAGS
}

#[cfg(windows)]
pub(crate) fn configure_asa_server_hidden_process(command: &mut Command) {
    command.creation_flags(ASA_SERVER_HIDDEN_CREATION_FLAGS);
}

pub(crate) fn spawn_asa_server(command: &mut Command) -> std::io::Result<tokio::process::Child> {
    #[cfg(windows)]
    // SetErrorMode 是进程级状态，串行化并发启动可避免恢复顺序互相覆盖。
    let _error_mode_guard = ASA_SERVER_SPAWN_ERROR_MODE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    #[cfg(windows)]
    // Windows Server 2016 上部分 ASA DLL 会因 SetThreadDescription 链接方式触发兼容性硬错误。
    // Windows 错误模式会被新建的 ASA 子进程继承，从而抑制会阻塞服务端启动的系统模态框。
    let _process_error_mode_guard = ProcessErrorModeGuard::suppress_asa_server_system_dialogs();

    command.spawn()
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

#[cfg(all(test, windows))]
mod tests {
    use std::process::Stdio;

    use super::{
        ASA_SERVER_SUPPRESSED_ERROR_MODE_FLAGS, asa_server_error_mode_with_suppressed_dialogs,
        spawn_asa_server,
    };
    use tokio::process::Command;
    use windows_sys::Win32::System::Diagnostics::Debug::GetErrorMode;

    const CHILD_ERROR_MODE_ENV: &str = "ASA_SERVER_MANAGER_REPORT_ERROR_MODE";
    const CHILD_ERROR_MODE_PREFIX: &str = "ASA_CHILD_ERROR_MODE=";

    #[test]
    fn asa_server_error_mode_preserves_existing_flags_and_suppresses_dialogs() {
        const EXISTING_ERROR_MODE: u32 = 0x0000_0004;

        let error_mode = asa_server_error_mode_with_suppressed_dialogs(EXISTING_ERROR_MODE);

        assert_eq!(error_mode & EXISTING_ERROR_MODE, EXISTING_ERROR_MODE);
        assert_eq!(
            error_mode & ASA_SERVER_SUPPRESSED_ERROR_MODE_FLAGS,
            ASA_SERVER_SUPPRESSED_ERROR_MODE_FLAGS
        );
    }

    #[test]
    fn child_process_reports_inherited_error_mode() {
        if std::env::var_os(CHILD_ERROR_MODE_ENV).is_none() {
            return;
        }

        println!("{CHILD_ERROR_MODE_PREFIX}{}", unsafe { GetErrorMode() });
    }

    #[tokio::test]
    async fn spawn_asa_server_restores_parent_mode_and_sets_child_mode() {
        let parent_error_mode = unsafe { GetErrorMode() };
        let expected_child_error_mode =
            asa_server_error_mode_with_suppressed_dialogs(parent_error_mode);
        let current_test_executable = std::env::current_exe().expect("无法获取当前测试程序路径");
        let mut command = Command::new(current_test_executable);
        command
            .args([
                "--exact",
                "asa_server_process::tests::child_process_reports_inherited_error_mode",
                "--nocapture",
            ])
            .env(CHILD_ERROR_MODE_ENV, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = spawn_asa_server(&mut command).expect("无法启动错误模式测试子进程");

        assert_eq!(unsafe { GetErrorMode() }, parent_error_mode);

        let output = child
            .wait_with_output()
            .await
            .expect("无法等待错误模式测试子进程");
        assert!(
            output.status.success(),
            "错误模式测试子进程执行失败：{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let inherited_error_mode = stdout
            .lines()
            .find_map(|line| line.strip_prefix(CHILD_ERROR_MODE_PREFIX))
            .and_then(|value| value.parse::<u32>().ok())
            .expect("错误模式测试子进程未返回有效结果");

        assert_eq!(inherited_error_mode, expected_child_error_mode);
    }
}
