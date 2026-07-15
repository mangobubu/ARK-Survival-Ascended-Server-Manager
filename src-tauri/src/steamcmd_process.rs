#[cfg(windows)]
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

#[cfg(windows)]
use tokio::{
    process::Command,
    time::{Instant, sleep, timeout},
};

#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessIoCounters, IO_COUNTERS, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};

#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
const STILL_ACTIVE_PROCESS_EXIT_CODE: u32 = 259;
#[cfg(windows)]
const MAX_PROCESS_IMAGE_PATH: usize = 32_768;
#[cfg(windows)]
const ERROR_INVALID_PARAMETER_CODE: i32 = 87;

#[derive(Clone, Copy, Default)]
pub(crate) struct ProcessTransferCounters {
    read: u64,
    write: u64,
    other: u64,
}

impl ProcessTransferCounters {
    pub(crate) fn estimated_download_delta_since(self, baseline: Self) -> u64 {
        [
            self.read.saturating_sub(baseline.read),
            self.write.saturating_sub(baseline.write),
            self.other.saturating_sub(baseline.other),
        ]
        .into_iter()
        .max()
        .unwrap_or(0)
    }
}

#[cfg(windows)]
pub(crate) fn process_transfer_counters(
    process_id: Option<u32>,
) -> Option<ProcessTransferCounters> {
    let process_id = process_id?;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if handle.is_null() {
            return None;
        }

        let mut counters = IO_COUNTERS::default();
        let ok = GetProcessIoCounters(handle, &mut counters);
        let _ = CloseHandle(handle);
        if ok == 0 {
            None
        } else {
            Some(ProcessTransferCounters {
                read: counters.ReadTransferCount,
                write: counters.WriteTransferCount,
                other: counters.OtherTransferCount,
            })
        }
    }
}

#[cfg(not(windows))]
pub(crate) fn process_transfer_counters(
    _process_id: Option<u32>,
) -> Option<ProcessTransferCounters> {
    None
}

#[cfg(windows)]
pub(crate) fn process_is_running(pid: u32) -> Result<bool, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(ERROR_INVALID_PARAMETER_CODE) {
                return Ok(false);
            }
            return Err(format!("无法打开进程 PID {pid} 检查运行状态：{error}"));
        }

        let mut exit_code = 0_u32;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        let query_error = (ok == 0).then(std::io::Error::last_os_error);
        let _ = CloseHandle(handle);
        if let Some(error) = query_error {
            return Err(format!("无法读取进程 PID {pid} 的退出状态：{error}"));
        }
        Ok(exit_code == STILL_ACTIVE_PROCESS_EXIT_CODE)
    }
}

#[cfg(not(windows))]
pub(crate) fn process_is_running(_pid: u32) -> Result<bool, String> {
    Ok(false)
}

#[cfg(windows)]
pub(crate) fn process_matches_executable(pid: u32, expected: &Path) -> Result<bool, String> {
    Ok(executable_paths_match(
        &process_executable_path(pid)?,
        expected,
    ))
}

#[cfg(not(windows))]
pub(crate) fn process_matches_executable(
    _pid: u32,
    _expected: &std::path::Path,
) -> Result<bool, String> {
    Ok(false)
}

#[cfg(windows)]
fn process_executable_path(pid: u32) -> Result<PathBuf, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return Err(format!(
                "无法打开进程 PID {pid} 核验可执行文件：{}",
                std::io::Error::last_os_error()
            ));
        }

        let mut buffer = vec![0_u16; MAX_PROCESS_IMAGE_PATH];
        let mut length = MAX_PROCESS_IMAGE_PATH as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length);
        let query_error = (ok == 0).then(std::io::Error::last_os_error);
        let _ = CloseHandle(handle);
        if let Some(error) = query_error {
            return Err(format!("无法读取进程 PID {pid} 的可执行文件路径：{error}"));
        }
        if length == 0 {
            return Err(format!("进程 PID {pid} 的可执行文件路径为空"));
        }

        buffer.truncate(length as usize);
        Ok(PathBuf::from(String::from_utf16_lossy(&buffer)))
    }
}

#[cfg(windows)]
fn executable_paths_match(actual: &Path, expected: &Path) -> bool {
    let actual = actual
        .canonicalize()
        .unwrap_or_else(|_| actual.to_path_buf());
    let expected = expected
        .canonicalize()
        .unwrap_or_else(|_| expected.to_path_buf());
    actual
        .to_string_lossy()
        .eq_ignore_ascii_case(&expected.to_string_lossy())
}

#[cfg(windows)]
pub(crate) async fn wait_for_process_exit(pid: u32, max_wait: Duration) -> Result<(), String> {
    let started_at = Instant::now();
    while process_is_running(pid)? {
        if started_at.elapsed() >= max_wait {
            return Err(format!(
                "等待进程 PID {pid} 退出超时（{} 秒）",
                max_wait.as_secs()
            ));
        }
        sleep(Duration::from_millis(100)).await;
    }
    Ok(())
}

#[cfg(not(windows))]
pub(crate) async fn wait_for_process_exit(
    _pid: u32,
    _max_wait: std::time::Duration,
) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
pub(crate) async fn kill_process_tree(pid: u32) -> Result<(), String> {
    let mut command = Command::new("taskkill");
    command
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/T")
        .arg("/F")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.creation_flags(CREATE_NO_WINDOW);

    let status = timeout(Duration::from_secs(5), command.status())
        .await
        .map_err(|_| format!("强制终止进程 PID {pid} 超时"))?
        .map_err(|error| format!("无法执行 taskkill 终止进程 PID {pid}：{error}"))?;
    if !status.success() {
        match process_is_running(pid) {
            Ok(false) => {}
            Ok(true) => {
                return Err(format!(
                    "taskkill 未能终止进程 PID {pid}，退出代码：{}",
                    status.code().unwrap_or_default()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "taskkill 终止进程 PID {pid} 后无法确认退出状态，退出代码：{}；{error}",
                    status.code().unwrap_or_default()
                ));
            }
        }
    }

    wait_for_process_exit(pid, Duration::from_secs(10)).await
}

#[cfg(not(windows))]
pub(crate) async fn kill_process_tree(_pid: u32) -> Result<(), String> {
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::{env, fs, path::Path, process::Command as StdCommand};

    const PROCESS_KILL_FIXTURE_ENV: &str = "ASA_MANAGER_PROCESS_KILL_FIXTURE";
    const PROCESS_KILL_FIXTURE_CHILD_ENV: &str = "ASA_MANAGER_PROCESS_KILL_FIXTURE_CHILD";
    const PROCESS_KILL_FIXTURE_PID_FILE_ENV: &str = "ASA_MANAGER_PROCESS_KILL_FIXTURE_PID_FILE";

    struct ProcessTreeCleanup {
        root_pid: u32,
    }

    impl Drop for ProcessTreeCleanup {
        fn drop(&mut self) {
            let root_pid = self.root_pid.to_string();
            let _ = StdCommand::new("taskkill")
                .args(["/PID", &root_pid, "/T", "/F"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    // 夹具会被父进程树强制终止，不能在此等待其子进程。
    #[allow(clippy::zombie_processes)]
    #[test]
    fn process_kill_fixture() {
        if env::var_os(PROCESS_KILL_FIXTURE_CHILD_ENV).is_some() {
            std::thread::sleep(Duration::from_secs(60));
            return;
        }
        let Some(pid_file) = env::var_os(PROCESS_KILL_FIXTURE_PID_FILE_ENV) else {
            return;
        };
        if env::var_os(PROCESS_KILL_FIXTURE_ENV).is_some() {
            let executable = env::current_exe().expect("无法获取进程树测试程序路径");
            let child = StdCommand::new(executable)
                .args(["--exact", "steamcmd_process::tests::process_kill_fixture"])
                .env(PROCESS_KILL_FIXTURE_CHILD_ENV, "1")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("无法启动进程树测试子进程");
            fs::write(pid_file, child.id().to_string()).expect("无法写入进程树测试子进程 PID");
            std::thread::sleep(Duration::from_secs(60));
        }
    }

    async fn read_fixture_child_pid(path: &Path) -> u32 {
        let started_at = Instant::now();
        loop {
            if let Ok(value) = fs::read_to_string(path)
                && let Ok(pid) = value.trim().parse::<u32>()
            {
                return pid;
            }
            assert!(
                started_at.elapsed() < Duration::from_secs(2),
                "等待进程树测试子进程 PID 超时"
            );
            sleep(Duration::from_millis(50)).await;
        }
    }

    #[tokio::test]
    async fn kill_process_tree_waits_until_target_exits() {
        let test_executable = env::current_exe().expect("无法获取测试程序路径");
        let temp = tempfile::tempdir().expect("创建进程树测试临时目录");
        let child_pid_path = temp.path().join("child-pid.txt");
        let mut command = Command::new(&test_executable);
        command
            .args(["--exact", "steamcmd_process::tests::process_kill_fixture"])
            .env(PROCESS_KILL_FIXTURE_ENV, "1")
            .env(PROCESS_KILL_FIXTURE_PID_FILE_ENV, &child_pid_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command.spawn().expect("无法启动进程终止测试子进程");
        let pid = child.id().expect("测试子进程缺少 PID");
        let _cleanup = ProcessTreeCleanup { root_pid: pid };
        let descendant_pid = read_fixture_child_pid(&child_pid_path).await;

        assert!(process_is_running(pid).expect("检查测试子进程状态"));
        assert!(process_is_running(descendant_pid).expect("检查进程树测试子进程状态"));
        assert!(process_matches_executable(pid, &test_executable).expect("核验测试子进程路径"));

        kill_process_tree(pid).await.expect("终止测试子进程");
        let exit_status = timeout(Duration::from_secs(2), child.wait())
            .await
            .expect("等待测试子进程退出超时")
            .expect("等待测试子进程退出失败");

        assert!(!exit_status.success());
        assert!(!process_is_running(pid).expect("确认测试子进程已退出"));
        assert!(!process_is_running(descendant_pid).expect("确认进程树测试子进程已退出"));
    }
}
