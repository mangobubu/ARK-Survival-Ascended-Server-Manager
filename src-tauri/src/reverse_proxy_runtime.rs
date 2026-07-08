use std::{
    path::PathBuf,
    process::{Command, Output},
};

const OPENRESTY_EXECUTABLE_NAME: &str = "nginx.exe";

pub(crate) fn resolve_openresty_executable_path(path_text: &str) -> Result<PathBuf, String> {
    let trimmed = path_text.trim().trim_matches('"');
    if trimmed.is_empty() {
        return Err("启用 Web 反向代理时必须填写 OpenResty 安装目录或 nginx.exe 路径".to_string());
    }

    let path = PathBuf::from(trimmed);
    let executable = if path.is_dir() {
        path.join(OPENRESTY_EXECUTABLE_NAME)
    } else {
        path
    };

    let file_name = executable
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !file_name.eq_ignore_ascii_case(OPENRESTY_EXECUTABLE_NAME) {
        return Err("OpenResty 路径必须指向安装目录或 nginx.exe".to_string());
    }
    if !executable.is_file() {
        return Err(format!(
            "未找到 OpenResty nginx.exe：{}",
            executable.display()
        ));
    }
    Ok(executable)
}

pub(crate) fn command_output_detail(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|value| !value.is_empty())
        .unwrap_or("无详细输出");
    detail.chars().take(1000).collect()
}

#[cfg(windows)]
pub(crate) fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub(crate) fn hide_console_window(_command: &mut Command) {}
