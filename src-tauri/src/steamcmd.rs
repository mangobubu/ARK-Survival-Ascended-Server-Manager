use futures_util::StreamExt;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::ipc::Channel;
use tokio::{io::AsyncWriteExt, process::Command, time::timeout};
use zip::ZipArchive;

const STEAMCMD_DOWNLOAD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamCmdCheck {
    pub path: String,
    pub executable_path: String,
    pub valid: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamCmdProgress {
    pub phase: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub bytes_per_second: u64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamCmdInstallResult {
    pub path: String,
    pub executable_path: String,
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn inspect_steamcmd(path: &Path) -> SteamCmdCheck {
    let executable = path.join("steamcmd.exe");
    let reason = if !path.exists() {
        Some("目录不存在".to_string())
    } else if !path.is_dir() {
        Some("所选路径不是目录".to_string())
    } else if !executable.is_file() {
        Some("目录中未找到 steamcmd.exe".to_string())
    } else {
        None
    };

    SteamCmdCheck {
        path: path_text(path),
        executable_path: path_text(&executable),
        valid: reason.is_none(),
        reason,
    }
}

#[tauri::command]
pub fn check_steamcmd(path: String) -> Result<SteamCmdCheck, String> {
    if path.trim().is_empty() {
        return Ok(SteamCmdCheck {
            path,
            executable_path: String::new(),
            valid: false,
            reason: Some("SteamCMD 目录不能为空".to_string()),
        });
    }

    Ok(inspect_steamcmd(Path::new(&path)))
}

fn emit_progress(
    channel: &Channel<SteamCmdProgress>,
    phase: &str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    bytes_per_second: u64,
    message: &str,
) -> Result<(), String> {
    channel
        .send(SteamCmdProgress {
            phase: phase.to_string(),
            downloaded_bytes,
            total_bytes,
            bytes_per_second,
            message: message.to_string(),
        })
        .map_err(|error| format!("发送下载进度失败：{error}"))
}

fn calculate_speed(bytes_delta: u64, elapsed: Duration) -> u64 {
    let seconds = elapsed.as_secs_f64();
    if seconds <= f64::EPSILON {
        0
    } else {
        (bytes_delta as f64 / seconds).round() as u64
    }
}

fn ensure_install_target(
    parent: &Path,
) -> Result<(PathBuf, Option<SteamCmdInstallResult>), String> {
    if !parent.exists() {
        return Err("选择的上级目录不存在".to_string());
    }
    if !parent.is_dir() {
        return Err("选择的上级路径不是目录".to_string());
    }

    let target = parent.join("SteamCMD");
    let current = inspect_steamcmd(&target);
    if current.valid {
        return Ok((
            target,
            Some(SteamCmdInstallResult {
                path: current.path,
                executable_path: current.executable_path,
            }),
        ));
    }

    if target.exists() {
        let mut entries =
            fs::read_dir(&target).map_err(|error| format!("无法读取目标目录：{error}"))?;
        if entries.next().is_some() {
            return Err(format!(
                "目标目录 {} 已存在且不为空，请选择其他上级目录",
                target.display()
            ));
        }
    }

    Ok((target, None))
}

async fn download_archive(
    archive_path: &Path,
    progress: &Channel<SteamCmdProgress>,
) -> Result<(u64, Option<u64>), String> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(300))
        .user_agent("ASA-Server-Manager/0.1")
        .build()
        .map_err(|error| format!("创建下载客户端失败：{error}"))?;

    let response = client
        .get(STEAMCMD_DOWNLOAD_URL)
        .send()
        .await
        .map_err(|error| format!("连接 SteamCMD 下载服务器失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("SteamCMD 下载服务器返回错误：{error}"))?;

    let total = response.content_length();
    let mut file = tokio::fs::File::create(archive_path)
        .await
        .map_err(|error| format!("无法创建临时下载文件：{error}"))?;
    let mut stream = response.bytes_stream();
    let mut downloaded = 0_u64;
    let mut sample_bytes = 0_u64;
    let mut sample_time = Instant::now();
    let mut current_speed = 0_u64;

    emit_progress(progress, "downloading", 0, total, 0, "正在下载 SteamCMD")?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("下载 SteamCMD 时连接中断：{error}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|error| format!("写入 SteamCMD 下载文件失败：{error}"))?;
        downloaded += chunk.len() as u64;
        sample_bytes += chunk.len() as u64;

        let elapsed = sample_time.elapsed();
        if elapsed >= Duration::from_millis(250) {
            current_speed = calculate_speed(sample_bytes, elapsed);
            emit_progress(
                progress,
                "downloading",
                downloaded,
                total,
                current_speed,
                "正在下载 SteamCMD",
            )?;
            sample_bytes = 0;
            sample_time = Instant::now();
        }
    }

    if sample_bytes > 0 {
        current_speed = calculate_speed(sample_bytes, sample_time.elapsed());
    }

    file.flush()
        .await
        .map_err(|error| format!("刷新 SteamCMD 下载文件失败：{error}"))?;
    emit_progress(
        progress,
        "downloading",
        downloaded,
        total,
        current_speed,
        "SteamCMD 下载完成",
    )?;
    Ok((downloaded, total))
}

fn cleanup_staging(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let archive_file =
        File::open(archive_path).map_err(|error| format!("无法打开 SteamCMD 压缩包：{error}"))?;
    let mut archive =
        ZipArchive::new(archive_file).map_err(|error| format!("SteamCMD 压缩包无效：{error}"))?;

    fs::create_dir_all(destination).map_err(|error| format!("无法创建解压目录：{error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("读取压缩包条目失败：{error}"))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "SteamCMD 压缩包包含不安全路径，已终止安装".to_string())?;
        let output_path = destination.join(enclosed);

        if entry.is_dir() {
            fs::create_dir_all(&output_path)
                .map_err(|error| format!("创建解压目录失败：{error}"))?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建解压目录失败：{error}"))?;
        }
        let mut output =
            File::create(&output_path).map_err(|error| format!("创建解压文件失败：{error}"))?;
        io::copy(&mut entry, &mut output)
            .map_err(|error| format!("解压 SteamCMD 文件失败：{error}"))?;
        output
            .flush()
            .map_err(|error| format!("写入 SteamCMD 文件失败：{error}"))?;
    }

    if !destination.join("steamcmd.exe").is_file() {
        return Err("解压完成后未找到 steamcmd.exe".to_string());
    }
    Ok(())
}

async fn initialize_steamcmd(directory: &Path) -> Result<(), String> {
    let executable = directory.join("steamcmd.exe");
    let mut command = Command::new(&executable);
    command
        .current_dir(directory)
        .arg("+quit")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    #[cfg(not(windows))]
    return Err("SteamCMD 静默安装目前仅支持 Windows".to_string());

    #[cfg(windows)]
    {
        let output = timeout(Duration::from_secs(600), command.output())
            .await
            .map_err(|_| "SteamCMD 首次初始化超时（10 分钟）".to_string())?
            .map_err(|error| format!("无法启动 SteamCMD 初始化：{error}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");
        let initialization_verified = executable.is_file()
            && directory.join("steamclient.dll").is_file()
            && combined.contains("Loading Steam API...OK")
            && combined.contains("Unloading Steam API...OK");

        if !output.status.success() && !initialization_verified {
            let mut detail = stderr.trim().to_string();
            if detail.is_empty() {
                detail = stdout.trim().to_string();
            }
            if detail.len() > 500 {
                detail.truncate(500);
            }
            return Err(if detail.is_empty() {
                format!("SteamCMD 初始化失败，退出代码：{}", output.status)
            } else {
                format!("SteamCMD 初始化失败（{}）：{detail}", output.status)
            });
        }
    }

    Ok(())
}

async fn install_inner(
    parent: &Path,
    progress: &Channel<SteamCmdProgress>,
) -> Result<SteamCmdInstallResult, String> {
    let (target, existing) = ensure_install_target(parent)?;
    if let Some(existing) = existing {
        emit_progress(progress, "completed", 0, None, 0, "已找到可用的 SteamCMD")?;
        return Ok(existing);
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let staging = parent.join(format!(
        ".steamcmd-installing-{}-{nonce}",
        std::process::id()
    ));
    let archive_path = staging.join("steamcmd.zip");
    let content_path = staging.join("content");
    fs::create_dir(&staging).map_err(|error| format!("无法创建安装临时目录：{error}"))?;

    let result = async {
        let (downloaded, total) = download_archive(&archive_path, progress).await?;
        emit_progress(
            progress,
            "extracting",
            downloaded,
            total,
            0,
            "正在安全解压 SteamCMD",
        )?;

        let archive_for_extract = archive_path.clone();
        let content_for_extract = content_path.clone();
        tokio::task::spawn_blocking(move || {
            extract_archive(&archive_for_extract, &content_for_extract)
        })
        .await
        .map_err(|error| format!("SteamCMD 解压任务异常：{error}"))??;

        emit_progress(
            progress,
            "initializing",
            downloaded,
            total,
            0,
            "正在后台初始化 SteamCMD",
        )?;
        initialize_steamcmd(&content_path).await?;

        if target.exists() {
            fs::remove_dir(&target).map_err(|error| format!("无法使用空目标目录：{error}"))?;
        }
        fs::rename(&content_path, &target)
            .map_err(|error| format!("无法完成 SteamCMD 安装：{error}"))?;

        let check = inspect_steamcmd(&target);
        if !check.valid {
            return Err(check
                .reason
                .unwrap_or_else(|| "SteamCMD 安装验证失败".to_string()));
        }

        emit_progress(
            progress,
            "completed",
            downloaded,
            total,
            0,
            "SteamCMD 安装完成",
        )?;
        Ok(SteamCmdInstallResult {
            path: check.path,
            executable_path: check.executable_path,
        })
    }
    .await;

    cleanup_staging(&staging);
    result
}

#[tauri::command]
pub async fn install_steamcmd(
    parent_path: String,
    progress: Channel<SteamCmdProgress>,
) -> Result<SteamCmdInstallResult, String> {
    #[cfg(not(windows))]
    {
        let _ = parent_path;
        let _ = progress;
        return Err("SteamCMD 静默安装目前仅支持 Windows".to_string());
    }

    #[cfg(windows)]
    {
        if parent_path.trim().is_empty() {
            return Err("SteamCMD 上级目录不能为空".to_string());
        }
        install_inner(Path::new(&parent_path), &progress).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tauri::ipc::InvokeResponseBody;
    use tempfile::tempdir;
    use zip::{ZipWriter, write::SimpleFileOptions};

    #[test]
    fn 检测有效与无效目录() {
        let temp = tempdir().expect("创建临时目录");
        let invalid = inspect_steamcmd(temp.path());
        assert!(!invalid.valid);

        File::create(temp.path().join("steamcmd.exe")).expect("创建测试程序");
        let valid = inspect_steamcmd(temp.path());
        assert!(valid.valid);
        assert!(valid.reason.is_none());
    }

    #[test]
    fn 拒绝覆盖非空目标目录() {
        let temp = tempdir().expect("创建临时目录");
        let target = temp.path().join("SteamCMD");
        fs::create_dir(&target).expect("创建目标目录");
        File::create(target.join("其他文件.txt")).expect("创建占位文件");

        let error = ensure_install_target(temp.path()).expect_err("应拒绝非空目录");
        assert!(error.contains("不为空"));
    }

    #[test]
    fn 安全解压并验证程序() {
        let temp = tempdir().expect("创建临时目录");
        let archive_path = temp.path().join("steamcmd.zip");
        let destination = temp.path().join("output");
        let file = File::create(&archive_path).expect("创建压缩包");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("steamcmd.exe", SimpleFileOptions::default())
            .expect("写入压缩包条目");
        writer.write_all(b"test").expect("写入测试数据");
        writer.finish().expect("完成压缩包");

        extract_archive(&archive_path, &destination).expect("解压成功");
        let mut content = String::new();
        File::open(destination.join("steamcmd.exe"))
            .expect("打开解压文件")
            .read_to_string(&mut content)
            .expect("读取解压文件");
        assert_eq!(content, "test");
    }

    #[test]
    fn 拒绝压缩包目录穿越() {
        let temp = tempdir().expect("创建临时目录");
        let archive_path = temp.path().join("unsafe.zip");
        let file = File::create(&archive_path).expect("创建压缩包");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("../outside.exe", SimpleFileOptions::default())
            .expect("写入压缩包条目");
        writer.write_all(b"test").expect("写入测试数据");
        writer.finish().expect("完成压缩包");

        let error = extract_archive(&archive_path, &temp.path().join("output"))
            .expect_err("应拒绝不安全路径");
        assert!(error.contains("不安全路径"));
        assert!(!temp.path().join("outside.exe").exists());
    }

    #[test]
    fn 正确计算下载速度() {
        assert_eq!(calculate_speed(1_000, Duration::from_secs(2)), 500);
        assert_eq!(calculate_speed(0, Duration::from_secs(1)), 0);
    }

    #[test]
    fn 清理安装临时目录() {
        let temp = tempdir().expect("创建临时目录");
        let staging = temp.path().join(".steamcmd-installing-test");
        fs::create_dir(&staging).expect("创建安装临时目录");
        File::create(staging.join("steamcmd.zip")).expect("创建临时下载文件");

        cleanup_staging(&staging);
        assert!(!staging.exists());
    }

    #[tokio::test]
    #[ignore = "需要访问 Valve 下载服务器并运行 SteamCMD 首次初始化"]
    async fn 真实下载并静默初始化() {
        let temp = tempdir().expect("创建真实下载测试目录");
        let progress_count = Arc::new(AtomicUsize::new(0));
        let progress_count_for_channel = Arc::clone(&progress_count);
        let channel = Channel::new(move |body| {
            if matches!(body, InvokeResponseBody::Json(_)) {
                progress_count_for_channel.fetch_add(1, Ordering::Relaxed);
            }
            Ok(())
        });

        let result = install_inner(temp.path(), &channel)
            .await
            .expect("真实下载安装应成功");
        assert!(Path::new(&result.executable_path).is_file());
        assert!(progress_count.load(Ordering::Relaxed) >= 4);
        assert!(!temp.path().join("steamcmd.zip").exists());
        assert!(
            fs::read_dir(temp.path())
                .expect("读取真实下载测试目录")
                .all(|entry| !entry
                    .expect("读取目录项")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".steamcmd-installing-"))
        );
    }
}
