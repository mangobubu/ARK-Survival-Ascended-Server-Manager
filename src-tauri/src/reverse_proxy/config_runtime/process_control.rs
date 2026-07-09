use std::{
    fs, io,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use crate::reverse_proxy_runtime::{command_output_detail, hide_console_window};

use super::super::{
    CONFIG_RELATIVE_PATH, ReverseProxyConfig, STARTUP_PID_WAIT_ATTEMPTS, STARTUP_PID_WAIT_STEP,
};

impl ReverseProxyConfig {
    pub(in crate::reverse_proxy) fn test_config(&self) -> Result<(), String> {
        let output = self.base_command().arg("-t").output().map_err(|error| {
            format!(
                "无法执行 OpenResty 配置校验 {}：{error}",
                self.openresty_executable_path.display()
            )
        })?;

        if output.status.success() {
            return Ok(());
        }

        Err(format!(
            "OpenResty 配置校验失败：{}",
            command_output_detail(&output)
        ))
    }

    pub(in crate::reverse_proxy) fn start(&self) -> Result<(), String> {
        self.remove_pid_file_if_exists()?;
        let mut command = self.base_command();
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let mut child = command.spawn().map_err(|error| {
            format!(
                "无法启动 OpenResty 反向代理 {}：{error}",
                self.openresty_executable_path.display()
            )
        })?;

        for _ in 0..STARTUP_PID_WAIT_ATTEMPTS {
            if self.running_pid()?.is_some() {
                return Ok(());
            }
            if child
                .try_wait()
                .map_err(|error| format!("检查 OpenResty 启动进程状态失败：{error}"))?
                .is_some_and(|status| !status.success())
            {
                break;
            }
            thread::sleep(STARTUP_PID_WAIT_STEP);
        }

        Err(format!(
            "OpenResty 已尝试启动，但未检测到有效运行进程；{}；请检查日志目录：{}",
            self.startup_failure_detail(),
            self.proxy_root_path.join("logs").display(),
        ))
    }

    pub(in crate::reverse_proxy) fn stop(&self) -> Result<(), String> {
        if !self.pid_path().exists() {
            return Ok(());
        }

        let _ = self
            .base_command()
            .arg("-s")
            .arg("quit")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(100));
            if !self.pid_path().exists() || self.running_pid().unwrap_or(None).is_none() {
                let _ = self.remove_pid_file_if_exists();
                return Ok(());
            }
        }
        Ok(())
    }

    pub(in crate::reverse_proxy) fn stop_stale_instance_best_effort(&self) {
        let _ = self.stop();
    }

    fn base_command(&self) -> Command {
        let mut command = Command::new(&self.openresty_executable_path);
        command
            .current_dir(&self.openresty_root_path)
            .arg("-p")
            .arg(&self.proxy_root_path)
            .arg("-c")
            .arg(CONFIG_RELATIVE_PATH);
        hide_console_window(&mut command);
        command
    }

    fn read_pid(&self) -> Result<Option<u32>, String> {
        let path = self.pid_path();
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "无法读取 OpenResty PID 文件 {}：{error}",
                    path.display()
                ));
            }
        };
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        trimmed.parse::<u32>().map(Some).map_err(|error| {
            format!(
                "OpenResty PID 文件内容无效 {}：{}（{error}）",
                path.display(),
                trimmed
            )
        })
    }

    pub(in crate::reverse_proxy) fn running_pid(&self) -> Result<Option<u32>, String> {
        let Some(pid) = self.read_pid()? else {
            return Ok(None);
        };
        Ok(process_is_running(pid).then_some(pid))
    }

    fn remove_pid_file_if_exists(&self) -> Result<(), String> {
        let path = self.pid_path();
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "无法清理 OpenResty 旧 PID 文件 {}：{error}",
                path.display()
            )),
        }
    }

    fn startup_failure_detail(&self) -> String {
        let pid_detail = match fs::read_to_string(self.pid_path()) {
            Ok(content) if content.trim().is_empty() => "PID 文件为空".to_string(),
            Ok(content) => format!("PID 文件内容：{}", content.trim()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => "PID 文件不存在".to_string(),
            Err(error) => format!("PID 文件读取失败：{error}"),
        };
        let log_detail = match fs::read_to_string(self.error_log_path()) {
            Ok(content) if !content.trim().is_empty() => {
                let tail: String = content
                    .chars()
                    .rev()
                    .take(2000)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                format!("最近错误日志：{}", tail.trim())
            }
            Ok(_) => "错误日志为空".to_string(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => "错误日志不存在".to_string(),
            Err(error) => format!("错误日志读取失败：{error}"),
        };
        format!("{pid_detail}，{log_detail}")
    }
}

#[cfg(windows)]
fn process_is_running(pid: u32) -> bool {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    };

    const STILL_ACTIVE_EXIT_CODE: u32 = 259;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut exit_code = 0_u32;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        let _ = CloseHandle(handle);
        ok != 0 && exit_code == STILL_ACTIVE_EXIT_CODE
    }
}

#[cfg(not(windows))]
fn process_is_running(_pid: u32) -> bool {
    true
}
