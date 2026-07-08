use std::{
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
        let mut command = self.base_command();
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        command.spawn().map_err(|error| {
            format!(
                "无法启动 OpenResty 反向代理 {}：{error}",
                self.openresty_executable_path.display()
            )
        })?;

        for _ in 0..STARTUP_PID_WAIT_ATTEMPTS {
            if self.pid_path().is_file() {
                return Ok(());
            }
            thread::sleep(STARTUP_PID_WAIT_STEP);
        }

        Err(format!(
            "OpenResty 已尝试启动，但未生成进程文件；请检查日志目录：{}",
            self.proxy_root_path.join("logs").display()
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
        thread::sleep(Duration::from_millis(300));
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
}
