use crate::{
    app_state::{AppRuntime, now_millis},
    ark_config, backup, import_export,
    models::{
        ASA_DEDICATED_SERVER_APP_ID, AddInstancePayload, BackupItem, ExportResult, GlobalSettings,
        ImportResult, JobProgress, LogLine, ModItem, PortCheckResult, ServerInstance, ServerStatus,
    },
    rcon,
};
use serde_json::Value;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, State, ipc::Channel};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    process::Command,
    task::JoinHandle,
    time::{MissedTickBehavior, interval, timeout},
};

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Threading::{
        GetProcessIoCounters, IO_COUNTERS, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    },
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const UPDATE_CANCELLED_MESSAGE: &str = "服务端安装/更新已取消";

#[derive(Clone)]
struct SteamCmdProgressSink {
    app: AppHandle,
    channel: Channel<JobProgress>,
    instance_id: String,
    tracker: Arc<Mutex<SteamCmdProgressTracker>>,
}

#[derive(Default)]
struct SteamCmdProgressTracker {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    bytes_per_second: u64,
    percent: Option<f64>,
    last_sample: Option<(Instant, u64)>,
    last_emit_at: Option<Instant>,
}

#[derive(Clone, Copy, Default)]
struct TransferSnapshot {
    percent: Option<f64>,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    bytes_per_second: u64,
}

struct ParsedSteamCmdProgress {
    phase: String,
    message: String,
    percent: Option<f64>,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

struct ManifestProgress {
    phase: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

#[derive(Clone, Copy, Default)]
struct ProcessTransferCounters {
    read: u64,
    write: u64,
    other: u64,
}

impl ProcessTransferCounters {
    fn estimated_download_delta_since(self, baseline: Self) -> u64 {
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

#[tauri::command]
pub fn get_settings(runtime: State<'_, AppRuntime>) -> Result<GlobalSettings, String> {
    runtime.settings()
}

#[tauri::command]
pub fn save_settings(
    runtime: State<'_, AppRuntime>,
    settings: GlobalSettings,
) -> Result<GlobalSettings, String> {
    validate_settings(&settings)?;
    runtime.save_settings(settings)
}

#[tauri::command]
pub fn list_instances(runtime: State<'_, AppRuntime>) -> Result<Vec<ServerInstance>, String> {
    runtime.list_instances()
}

#[tauri::command]
pub fn check_instance_port(
    runtime: State<'_, AppRuntime>,
    port: u16,
    port_kind: String,
) -> Result<PortCheckResult, String> {
    runtime.check_instance_port(port, &port_kind)
}

#[tauri::command]
pub fn create_instance(
    runtime: State<'_, AppRuntime>,
    payload: AddInstancePayload,
) -> Result<ServerInstance, String> {
    runtime.create_instance(payload)
}

#[tauri::command]
pub fn get_instance_config(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<Value, String> {
    runtime.get_config(&instance_id)
}

#[tauri::command]
pub fn get_instance_mods(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<Vec<ModItem>, String> {
    runtime.get_mods(&instance_id)
}

#[tauri::command]
pub fn save_instance_config(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    config: Value,
    mods: Vec<ModItem>,
) -> Result<(), String> {
    let instance = runtime.get_instance(&instance_id)?;
    runtime.save_config_and_mods(&instance_id, config.clone(), mods.clone())?;
    let applied = ark_config::apply_instance_config(&instance, &config, &mods)?;
    runtime.add_log(
        &instance.name,
        "success",
        &format!(
            "配置已写入 ARK 配置文件：{}、{}，启动参数 {} 项",
            applied.game_user_settings_path.to_string_lossy(),
            applied.game_ini_path.to_string_lossy(),
            applied.launch_arguments.len()
        ),
    )?;
    Ok(())
}

#[tauri::command]
pub async fn apply_instance_config(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    config: Value,
    mods: Vec<ModItem>,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    save_config_for_runtime(&runtime, &instance_id, config, mods)?;
    let instance = runtime.get_instance(&instance_id)?;
    if instance.status == ServerStatus::Running {
        restart_instance_inner(app, runtime, instance_id).await
    } else {
        Ok(instance)
    }
}

#[tauri::command]
pub fn update_instance_mods(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    mods: Vec<ModItem>,
) -> Result<Vec<ModItem>, String> {
    validate_mods(&mods)?;
    let config = runtime.get_config(&instance_id)?;
    runtime.save_config_and_mods(&instance_id, config, mods.clone())?;
    Ok(mods)
}

#[tauri::command]
pub fn check_mod_updates(mods: Vec<ModItem>) -> Result<Vec<ModItem>, String> {
    validate_mods(&mods)?;
    Ok(mods
        .into_iter()
        .map(|mut item| {
            if item.version.trim().is_empty() || item.version == "等待检测" {
                item.version = "本地校验通过".to_string();
            }
            item.update_available = Some(false);
            item
        })
        .collect())
}

#[tauri::command]
pub async fn install_or_update_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    progress: Channel<JobProgress>,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    let mut instance = runtime.get_instance(&instance_id)?;
    let settings = runtime.settings()?;
    let steamcmd = Path::new(&settings.steam_cmd_path).join("steamcmd.exe");
    if !steamcmd.is_file() {
        let error = format!("SteamCMD 不存在：{}", steamcmd.display());
        let _ =
            runtime.update_instance_status(&instance.id, ServerStatus::Error, Some(error.clone()));
        let _ = emit_instance_log(&app, &runtime, &instance.name, "error", &error);
        let _ = emit_status(&app, &runtime, &instance.id);
        return Err(error);
    }
    if let Err(error) = std::fs::create_dir_all(&instance.install_path) {
        let error = format!("无法创建实例目录 {}：{error}", instance.install_path);
        let _ =
            runtime.update_instance_status(&instance.id, ServerStatus::Error, Some(error.clone()));
        let _ = emit_instance_log(&app, &runtime, &instance.name, "error", &error);
        let _ = emit_status(&app, &runtime, &instance.id);
        return Err(error);
    }
    let update_cancel = runtime.begin_update(&instance.id)?;

    emit_progress(
        &progress,
        &instance.id,
        "preparing",
        Some(5.0),
        "正在准备服务端安装/更新",
        None,
    )?;
    runtime.update_instance_status(&instance.id, ServerStatus::Updating, None)?;
    emit_status(&app, &runtime, &instance.id)?;
    emit_instance_log(
        &app,
        &runtime,
        &instance.name,
        "info",
        "开始安装/更新服务端文件",
    )?;

    let output = run_steamcmd_update(
        &app,
        &runtime,
        &steamcmd,
        Path::new(&instance.install_path),
        &progress,
        &instance,
        update_cancel,
    )
    .await;
    runtime.finish_update(&instance.id);
    match output {
        Ok(detail) => {
            emit_progress(
                &progress,
                &instance.id,
                "completed",
                Some(100.0),
                "服务端安装/更新完成",
                Some(detail),
            )?;
            instance.version_state = "已安装/已更新".to_string();
            instance.status = ServerStatus::Stopped;
            instance.last_error = None;
            let updated = runtime.upsert_instance(instance.clone())?;
            emit_instance_log(
                &app,
                &runtime,
                &updated.name,
                "success",
                "服务端安装/更新完成",
            )?;
            emit_status(&app, &runtime, &updated.id)?;
            Ok(updated)
        }
        Err(error) => {
            let cancelled = error == UPDATE_CANCELLED_MESSAGE;
            runtime.update_instance_status(
                &instance.id,
                if cancelled {
                    ServerStatus::Stopped
                } else {
                    ServerStatus::Error
                },
                if cancelled { None } else { Some(error.clone()) },
            )?;
            let log_level = if cancelled { "warn" } else { "error" };
            let log_message = if cancelled {
                UPDATE_CANCELLED_MESSAGE.to_string()
            } else {
                format!("服务端安装/更新失败：{error}")
            };
            emit_instance_log(&app, &runtime, &instance.name, log_level, &log_message)?;
            emit_status(&app, &runtime, &instance.id)?;
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn start_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    start_instance_inner(app, runtime, instance_id).await
}

#[tauri::command]
pub async fn stop_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    stop_instance_inner(app, runtime, instance_id).await
}

#[tauri::command]
pub async fn restart_instance(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    restart_instance_inner(app, runtime, instance_id).await
}

#[tauri::command]
pub async fn refresh_instance_status(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let runtime = runtime.inner().clone();
    refresh_status_inner(&app, &runtime, &instance_id).await
}

#[tauri::command]
pub fn query_logs(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<LogLine>, String> {
    runtime.query_logs(limit)
}

#[tauri::command]
pub fn clear_logs(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    runtime.clear_logs()
}

#[tauri::command]
pub fn create_backup(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<BackupItem, String> {
    let settings = runtime.settings()?;
    let instance = runtime.get_instance(&instance_id)?;
    let backup =
        backup::create_instance_backup(Path::new(&settings.backup_storage_path), &instance)?;
    runtime.add_log(
        &instance.name,
        "success",
        &format!("备份已创建：{}", backup.path),
    )?;
    Ok(backup)
}

#[tauri::command]
pub fn list_backups(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<Vec<BackupItem>, String> {
    let settings = runtime.settings()?;
    let instance = runtime.get_instance(&instance_id)?;
    backup::list_instance_backups(Path::new(&settings.backup_storage_path), &instance)
}

#[tauri::command]
pub fn restore_backup(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
    backup_path: String,
) -> Result<(), String> {
    let instance = runtime.get_instance(&instance_id)?;
    backup::restore_instance_backup(&instance, Path::new(&backup_path))?;
    runtime.add_log(
        &instance.name,
        "warn",
        &format!("已恢复备份：{backup_path}"),
    )?;
    Ok(())
}

#[tauri::command]
pub fn export_instance_config(
    runtime: State<'_, AppRuntime>,
    instance_ids: Vec<String>,
) -> Result<ExportResult, String> {
    import_export::export_instances(&runtime, instance_ids)
}

#[tauri::command]
pub fn export_cluster(runtime: State<'_, AppRuntime>) -> Result<ExportResult, String> {
    import_export::export_instances(&runtime, Vec::new())
}

#[tauri::command]
pub fn import_instance_config(
    runtime: State<'_, AppRuntime>,
    path: String,
) -> Result<ImportResult, String> {
    import_export::import_instances(&runtime, Path::new(&path))
}

#[tauri::command]
pub fn open_instance_directory(
    runtime: State<'_, AppRuntime>,
    instance_id: String,
) -> Result<(), String> {
    let instance = runtime.get_instance(&instance_id)?;
    let path = Path::new(&instance.install_path);
    if !path.exists() {
        return Err(format!("实例目录不存在：{}", path.display()));
    }
    open_directory(path)
}

fn save_config_for_runtime(
    runtime: &AppRuntime,
    instance_id: &str,
    config: Value,
    mods: Vec<ModItem>,
) -> Result<(), String> {
    let instance = runtime.get_instance(instance_id)?;
    runtime.save_config_and_mods(instance_id, config.clone(), mods.clone())?;
    let applied = ark_config::apply_instance_config(&instance, &config, &mods)?;
    runtime.add_log(
        &instance.name,
        "success",
        &format!(
            "配置已保存：{}、{}，配置目录：{}，启动参数 {} 项",
            applied.game_user_settings_path.to_string_lossy(),
            applied.game_ini_path.to_string_lossy(),
            applied.config_dir.to_string_lossy(),
            applied.launch_arguments.len()
        ),
    )?;
    Ok(())
}

async fn run_steamcmd_update(
    app: &AppHandle,
    runtime: &AppRuntime,
    steamcmd: &Path,
    install_path: &Path,
    progress: &Channel<JobProgress>,
    instance: &ServerInstance,
    cancel: Arc<AtomicBool>,
) -> Result<String, String> {
    emit_progress(
        progress,
        &instance.id,
        "running",
        None,
        "正在调用 SteamCMD 安装/更新 ASA Dedicated Server",
        None,
    )?;
    emit_instance_log(
        app,
        runtime,
        &instance.name,
        "info",
        &format!(
            "SteamCMD 命令：{} +force_install_dir {} +login anonymous +app_update {} validate +quit",
            steamcmd.display(),
            install_path.display(),
            ASA_DEDICATED_SERVER_APP_ID
        ),
    )?;

    let mut command = Command::new(steamcmd);
    command
        .arg("+force_install_dir")
        .arg(install_path)
        .arg("+login")
        .arg("anonymous")
        .arg("+app_update")
        .arg(ASA_DEDICATED_SERVER_APP_ID)
        .arg("validate")
        .arg("+quit")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    #[cfg(not(windows))]
    return Err("ASA 服务端自动安装/更新目前仅支持 Windows".to_string());

    #[cfg(windows)]
    {
        let mut child = command
            .spawn()
            .map_err(|error| format!("无法启动 SteamCMD：{error}"))?;
        let output_tail = Arc::new(Mutex::new(VecDeque::with_capacity(24)));
        let progress_sink = SteamCmdProgressSink {
            app: app.clone(),
            channel: progress.clone(),
            instance_id: instance.id.clone(),
            tracker: Arc::new(Mutex::new(SteamCmdProgressTracker::default())),
        };
        let child_pid = child.id();
        let manifest_monitor_stop = Arc::new(AtomicBool::new(false));
        let manifest_monitor = spawn_manifest_progress_monitor(
            install_path,
            child_pid,
            progress_sink.clone(),
            Arc::clone(&manifest_monitor_stop),
        );
        let readers = vec![
            spawn_command_log_reader(
                app,
                runtime,
                &instance.name,
                child.stdout.take(),
                "info",
                Arc::clone(&output_tail),
                Some(progress_sink.clone()),
            ),
            spawn_command_log_reader(
                app,
                runtime,
                &instance.name,
                child.stderr.take(),
                "error",
                Arc::clone(&output_tail),
                Some(progress_sink.clone()),
            ),
        ];

        let started_at = Instant::now();
        let status = loop {
            if cancel.load(Ordering::SeqCst) {
                emit_instance_log(
                    app,
                    runtime,
                    &instance.name,
                    "warn",
                    "已取消安装/更新，正在结束 SteamCMD 进程树",
                )?;
                if let Some(pid) = child_pid {
                    kill_process_tree(pid).await;
                }
                let _ = child.kill().await;
                let _ = child.wait().await;
                manifest_monitor_stop.store(true, Ordering::SeqCst);
                manifest_monitor.abort();
                wait_for_log_readers(readers).await;
                return Err(UPDATE_CANCELLED_MESSAGE.to_string());
            }

            match timeout(Duration::from_millis(500), child.wait()).await {
                Ok(Ok(status)) => break status,
                Ok(Err(error)) => {
                    manifest_monitor_stop.store(true, Ordering::SeqCst);
                    manifest_monitor.abort();
                    wait_for_log_readers(readers).await;
                    return Err(format!("等待 SteamCMD 结束失败：{error}"));
                }
                Err(_) if started_at.elapsed() >= Duration::from_secs(60 * 60) => {
                    if let Some(pid) = child_pid {
                        kill_process_tree(pid).await;
                    }
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    manifest_monitor_stop.store(true, Ordering::SeqCst);
                    manifest_monitor.abort();
                    wait_for_log_readers(readers).await;
                    return Err("SteamCMD 安装/更新超时（60 分钟）".to_string());
                }
                Err(_) => {}
            }
        };
        manifest_monitor_stop.store(true, Ordering::SeqCst);
        manifest_monitor.abort();
        wait_for_log_readers(readers).await;

        if !status.success() {
            let fallback = format!("SteamCMD 安装/更新失败，退出代码：{status}");
            return Err(tail_detail(&output_tail, &fallback));
        }
        let transfer = transfer_snapshot(&progress_sink);
        emit_progress_with_transfer(
            progress,
            &instance.id,
            "verifying",
            transfer.percent,
            "正在验证服务端文件",
            None,
            transfer,
        )?;
        if ark_config::server_executable(instance).is_none() {
            return Err(tail_detail(
                &output_tail,
                "SteamCMD 执行完成，但未找到 ASA 服务端可执行文件",
            ));
        }
        Ok(tail_detail(&output_tail, "SteamCMD 安装/更新完成"))
    }
}

async fn start_instance_inner(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let instance = runtime.get_instance(&instance_id)?;
    if instance.status == ServerStatus::Running {
        return Ok(instance);
    }
    let config = runtime.get_config(&instance_id)?;
    let mods = runtime.get_mods(&instance_id)?;
    save_config_for_runtime(&runtime, &instance_id, config.clone(), mods.clone())?;
    let executable = ark_config::server_executable(&instance).ok_or_else(|| {
        format!(
            "未找到 ASA 服务端可执行文件，请先安装/更新实例：{}",
            instance.install_path
        )
    })?;

    runtime.update_instance_status(&instance_id, ServerStatus::Starting, None)?;
    emit_status(&app, &runtime, &instance_id)?;
    let args = ark_config::build_launch_arguments(&instance, &config, &mods);

    let mut command = Command::new(&executable);
    command
        .current_dir(
            executable
                .parent()
                .unwrap_or_else(|| Path::new(&instance.install_path)),
        )
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(false);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let mut child = command
        .spawn()
        .map_err(|error| format!("启动 ASA 服务端失败：{error}"))?;
    let pid = child.id();
    attach_process_log_reader(&app, &runtime, &instance, child.stdout.take(), "info");
    attach_process_log_reader(&app, &runtime, &instance, child.stderr.take(), "error");

    {
        let mut processes = runtime.lock_processes()?;
        processes.insert(instance_id.clone(), child);
    }

    runtime.add_log(
        &instance.name,
        "success",
        &format!("实例启动命令已执行，PID：{}", pid.unwrap_or_default()),
    )?;
    let updated = runtime.set_instance_pid(&instance_id, pid, ServerStatus::Running)?;
    emit_status(&app, &runtime, &instance_id)?;
    Ok(updated)
}

async fn stop_instance_inner(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    let instance = runtime.get_instance(&instance_id)?;
    let config = runtime.get_config(&instance_id)?;
    if instance.status == ServerStatus::Stopped {
        return Ok(instance);
    }
    if instance.status == ServerStatus::Updating {
        if runtime.cancel_update(&instance_id)? {
            emit_instance_log(
                &app,
                &runtime,
                &instance.name,
                "warn",
                "正在取消安装/更新任务",
            )?;
            for _ in 0..20 {
                if !runtime.is_update_running(&instance_id)? {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }

        let updated = runtime.set_instance_pid(&instance_id, None, ServerStatus::Stopped)?;
        emit_status(&app, &runtime, &instance_id)?;
        return Ok(updated);
    }

    if bool_from_config(&config, "saveOnStop", true) {
        let password = config
            .get("adminPassword")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let rcon_port = config
            .get("rconPort")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(instance.rcon_port);
        match rcon::execute("127.0.0.1", rcon_port, password, "saveworld").await {
            Ok(_) => {
                runtime.add_log(&instance.name, "success", "RCON 已执行 saveworld")?;
            }
            Err(error) => {
                runtime.add_log(
                    &instance.name,
                    "warn",
                    &format!("RCON 保存失败，将继续停止进程：{error}"),
                )?;
            }
        }
        let _ = rcon::execute("127.0.0.1", rcon_port, password, "doexit").await;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    let mut child = {
        let mut processes = runtime.lock_processes()?;
        processes.remove(&instance_id)
    };
    if let Some(child) = child.as_mut() {
        if child.try_wait().ok().flatten().is_none() {
            if let Some(pid) = child.id() {
                kill_process_tree(pid).await;
            }
            let _ = child.kill().await;
        }
    }

    runtime.add_log(&instance.name, "warn", "实例已停止")?;
    let updated = runtime.set_instance_pid(&instance_id, None, ServerStatus::Stopped)?;
    emit_status(&app, &runtime, &instance_id)?;
    Ok(updated)
}

async fn restart_instance_inner(
    app: AppHandle,
    runtime: AppRuntime,
    instance_id: String,
) -> Result<ServerInstance, String> {
    stop_instance_inner(app.clone(), runtime.clone(), instance_id.clone()).await?;
    start_instance_inner(app, runtime, instance_id).await
}

async fn refresh_status_inner(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<ServerInstance, String> {
    let exited = {
        let mut processes = runtime.lock_processes()?;
        if let Some(child) = processes.get_mut(instance_id) {
            child
                .try_wait()
                .map_err(|error| format!("刷新进程状态失败：{error}"))?
                .is_some()
        } else {
            false
        }
    };
    if exited {
        let instance = runtime.get_instance(instance_id)?;
        {
            let mut processes = runtime.lock_processes()?;
            processes.remove(instance_id);
        }
        runtime.add_log(&instance.name, "warn", "检测到服务端进程已退出")?;
        let updated = runtime.set_instance_pid(instance_id, None, ServerStatus::Stopped)?;
        emit_status(app, runtime, instance_id)?;
        return Ok(updated);
    }
    runtime.get_instance(instance_id)
}

fn attach_process_log_reader(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance: &ServerInstance,
    stream: Option<impl tokio::io::AsyncRead + Unpin + Send + 'static>,
    level: &'static str,
) {
    let Some(stream) = stream else {
        return;
    };
    let app = app.clone();
    let runtime = runtime.clone();
    let instance_name = instance.name.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(log_line) = runtime.add_log(&instance_name, level, &line) {
                let _ = app.emit("asa:log-line", log_line);
            }
        }
    });
}

fn emit_status(app: &AppHandle, runtime: &AppRuntime, instance_id: &str) -> Result<(), String> {
    let instance = runtime.get_instance(instance_id)?;
    app.emit("asa:instance-status", instance)
        .map_err(|error| format!("发送实例状态事件失败：{error}"))
}

fn emit_instance_log(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    level: &str,
    message: &str,
) -> Result<(), String> {
    let line = runtime.add_log(instance_name, level, message)?;
    app.emit("asa:log-line", line)
        .map_err(|error| format!("发送日志事件失败：{error}"))
}

fn emit_progress(
    channel: &Channel<JobProgress>,
    instance_id: &str,
    phase: &str,
    percent: Option<f64>,
    message: &str,
    detail: Option<String>,
) -> Result<(), String> {
    emit_progress_with_transfer(
        channel,
        instance_id,
        phase,
        percent,
        message,
        detail,
        TransferSnapshot {
            percent,
            downloaded_bytes: 0,
            total_bytes: None,
            bytes_per_second: 0,
        },
    )
}

fn emit_progress_with_transfer(
    channel: &Channel<JobProgress>,
    instance_id: &str,
    phase: &str,
    percent: Option<f64>,
    message: &str,
    detail: Option<String>,
    transfer: TransferSnapshot,
) -> Result<(), String> {
    channel
        .send(JobProgress {
            job_id: format!("job-{}", now_millis()),
            instance_id: Some(instance_id.to_string()),
            phase: phase.to_string(),
            percent,
            message: message.to_string(),
            detail,
            downloaded_bytes: transfer.downloaded_bytes,
            total_bytes: transfer.total_bytes,
            bytes_per_second: transfer.bytes_per_second,
        })
        .map_err(|error| format!("发送任务进度失败：{error}"))
}

fn validate_settings(settings: &GlobalSettings) -> Result<(), String> {
    if settings.steam_cmd_path.trim().is_empty() {
        return Err("SteamCMD 目录不能为空".to_string());
    }
    if settings.server_storage_path.trim().is_empty() {
        return Err("服务器存储目录不能为空".to_string());
    }
    if settings.backup_storage_path.trim().is_empty() {
        return Err("备份存储目录不能为空".to_string());
    }
    std::fs::create_dir_all(&settings.server_storage_path)
        .map_err(|error| format!("无法创建服务器存储目录：{error}"))?;
    std::fs::create_dir_all(&settings.backup_storage_path)
        .map_err(|error| format!("无法创建备份存储目录：{error}"))?;
    Ok(())
}

fn validate_mods(mods: &[ModItem]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for item in mods {
        if item.id.trim().is_empty() {
            return Err("MOD ID 不能为空".to_string());
        }
        if !item.id.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(format!("MOD ID 只能包含数字：{}", item.id));
        }
        if !seen.insert(item.id.trim().to_string()) {
            return Err(format!("MOD ID 重复：{}", item.id));
        }
    }
    Ok(())
}

fn bool_from_config(config: &Value, key: &str, fallback: bool) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}

fn trim_detail(content: &str, fallback: &str) -> String {
    let detail = content
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or(fallback);
    let mut text = detail.to_string();
    if text.len() > 500 {
        text.truncate(500);
    }
    text
}

fn remember_tail(tail: &Arc<Mutex<VecDeque<String>>>, line: &str) {
    if let Ok(mut tail) = tail.lock() {
        if tail.len() >= 24 {
            tail.pop_front();
        }
        tail.push_back(line.to_string());
    }
}

fn tail_detail(tail: &Arc<Mutex<VecDeque<String>>>, fallback: &str) -> String {
    let detail = tail
        .lock()
        .ok()
        .and_then(|tail| {
            tail.iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
        })
        .unwrap_or_else(|| fallback.to_string());
    trim_detail(&detail, fallback)
}

impl SteamCmdProgressTracker {
    fn update(
        &mut self,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        percent: Option<f64>,
        now: Instant,
        force_emit: bool,
    ) -> (TransferSnapshot, bool) {
        self.downloaded_bytes = downloaded_bytes;
        self.total_bytes = total_bytes;
        self.percent = match total_bytes {
            Some(total) if total > 0 => Some(percent_from_bytes(downloaded_bytes, total)),
            _ => percent,
        };

        let should_sample = self
            .last_sample
            .map(|(last_at, last_bytes)| {
                downloaded_bytes < last_bytes
                    || now.duration_since(last_at) >= Duration::from_millis(250)
            })
            .unwrap_or(true);

        if should_sample {
            if let Some((last_at, last_bytes)) = self.last_sample {
                let elapsed = now.duration_since(last_at).as_secs_f64();
                if elapsed > 0.0 && downloaded_bytes >= last_bytes {
                    self.bytes_per_second =
                        ((downloaded_bytes - last_bytes) as f64 / elapsed).round() as u64;
                } else if downloaded_bytes < last_bytes {
                    self.bytes_per_second = 0;
                }
            }
            self.last_sample = Some((now, downloaded_bytes));
        }

        let reached_total = total_bytes
            .map(|total| total > 0 && downloaded_bytes >= total)
            .unwrap_or(false);
        let should_emit = force_emit
            || self
                .last_emit_at
                .map(|last_at| now.duration_since(last_at) >= Duration::from_millis(200))
                .unwrap_or(true)
            || reached_total;

        if should_emit {
            self.last_emit_at = Some(now);
        }

        (self.snapshot(), should_emit)
    }

    fn snapshot(&self) -> TransferSnapshot {
        TransferSnapshot {
            percent: self.percent,
            downloaded_bytes: self.downloaded_bytes,
            total_bytes: self.total_bytes,
            bytes_per_second: self.bytes_per_second,
        }
    }
}

fn percent_from_bytes(downloaded_bytes: u64, total_bytes: u64) -> f64 {
    if total_bytes == 0 {
        return 0.0;
    }
    ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0)
}

fn parse_percent(value: &str) -> Option<f64> {
    let number = value
        .trim()
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()?;
    Some(number.clamp(0.0, 100.0))
}

fn parse_byte_count(value: &str) -> Option<u64> {
    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

fn normalize_steamcmd_phase(state: &str) -> String {
    let lower = state.to_ascii_lowercase();
    if lower.contains("download") {
        "downloading"
    } else if lower.contains("verify") || lower.contains("validat") {
        "verifying"
    } else if lower.contains("prealloc") {
        "preallocating"
    } else if lower.contains("commit") {
        "committing"
    } else {
        "running"
    }
    .to_string()
}

fn steamcmd_progress_message(phase: &str, state: &str) -> String {
    match phase {
        "downloading" => "SteamCMD 正在下载服务端文件".to_string(),
        "verifying" => "SteamCMD 正在校验服务端文件".to_string(),
        "preallocating" => "SteamCMD 正在预分配文件".to_string(),
        "committing" => "SteamCMD 正在提交更新".to_string(),
        _ if !state.trim().is_empty() => format!("SteamCMD {}", state.trim()),
        _ => "SteamCMD 正在处理更新".to_string(),
    }
}

fn parse_steamcmd_progress_line(line: &str) -> Option<ParsedSteamCmdProgress> {
    let lower_line = line.to_ascii_lowercase();
    let progress_index = lower_line.find("progress:")?;
    let after_progress = &line[progress_index + "progress:".len()..];
    let open_paren = after_progress.find('(')?;
    let close_paren = after_progress[open_paren + 1..].find(')')? + open_paren + 1;
    let percent = parse_percent(&after_progress[..open_paren]);
    let byte_pair = &after_progress[open_paren + 1..close_paren];
    let (downloaded_text, total_text) = byte_pair.split_once('/')?;
    let downloaded_bytes = parse_byte_count(downloaded_text)?;
    let total_bytes = parse_byte_count(total_text).filter(|total| *total > 0);

    let state_source = line[..progress_index].trim().trim_end_matches(',');
    let state = state_source
        .rsplit_once(')')
        .map(|(_, state)| state)
        .unwrap_or(state_source)
        .trim()
        .trim_start_matches(',')
        .trim();
    let phase = normalize_steamcmd_phase(state);
    let message = steamcmd_progress_message(&phase, state);
    let percent = total_bytes
        .map(|total| percent_from_bytes(downloaded_bytes, total))
        .or(percent);

    Some(ParsedSteamCmdProgress {
        phase,
        message,
        percent,
        downloaded_bytes,
        total_bytes,
    })
}

fn acf_u64(content: &str, key: &str) -> Option<u64> {
    content.lines().find_map(|line| {
        let mut parts = line.split('"');
        let _ = parts.next()?;
        let found_key = parts.next()?;
        let _ = parts.next()?;
        let value = parts.next()?;
        if found_key == key {
            value.parse::<u64>().ok()
        } else {
            None
        }
    })
}

fn parse_manifest_progress(content: &str) -> Option<ManifestProgress> {
    let bytes_to_download = acf_u64(content, "BytesToDownload").unwrap_or(0);
    let bytes_downloaded = acf_u64(content, "BytesDownloaded").unwrap_or(0);
    let bytes_to_stage = acf_u64(content, "BytesToStage").unwrap_or(0);
    let bytes_staged = acf_u64(content, "BytesStaged").unwrap_or(0);

    if bytes_to_download > 0 && bytes_downloaded < bytes_to_download {
        return Some(ManifestProgress {
            phase: "downloading".to_string(),
            downloaded_bytes: bytes_downloaded,
            total_bytes: Some(bytes_to_download),
        });
    }

    if bytes_to_stage > 0 {
        return Some(ManifestProgress {
            phase: if bytes_staged < bytes_to_stage {
                "committing"
            } else {
                "verifying"
            }
            .to_string(),
            downloaded_bytes: bytes_staged.min(bytes_to_stage),
            total_bytes: Some(bytes_to_stage),
        });
    }

    None
}

fn spawn_manifest_progress_monitor(
    install_path: &Path,
    process_id: Option<u32>,
    progress: SteamCmdProgressSink,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let manifest_path = install_path
        .join("steamapps")
        .join(format!("appmanifest_{ASA_DEDICATED_SERVER_APP_ID}.acf"));
    let downloading_path = install_path
        .join("steamapps")
        .join("downloading")
        .join(ASA_DEDICATED_SERVER_APP_ID);

    tokio::spawn(async move {
        let mut io_baseline: Option<ProcessTransferCounters> = None;
        let mut downloading_size_baseline: Option<u64> = None;
        let mut manifest_baseline = 0_u64;
        let mut ticker = interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        ticker.tick().await;

        while !stop.load(Ordering::SeqCst) {
            if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                if let Some(mut manifest) = parse_manifest_progress(&content) {
                    if manifest.phase == "downloading" {
                        let manifest_changed = manifest.downloaded_bytes != manifest_baseline;
                        let mut estimated_delta = 0_u64;

                        if let Some(current_transfer) = process_transfer_counters(process_id) {
                            if io_baseline.is_none() || manifest_changed {
                                io_baseline = Some(current_transfer);
                            }

                            if let Some(base_transfer) = io_baseline {
                                estimated_delta = estimated_delta.max(
                                    current_transfer.estimated_download_delta_since(base_transfer),
                                );
                            }
                        }

                        if let Some(current_size) = directory_size_bytes(&downloading_path) {
                            if downloading_size_baseline.is_none() || manifest_changed {
                                downloading_size_baseline = Some(current_size);
                            }

                            if let Some(base_size) = downloading_size_baseline {
                                estimated_delta =
                                    estimated_delta.max(current_size.saturating_sub(base_size));
                            }
                        }

                        manifest_baseline = manifest.downloaded_bytes;

                        if let Some(total) = manifest.total_bytes {
                            manifest.downloaded_bytes = manifest
                                .downloaded_bytes
                                .saturating_add(estimated_delta)
                                .min(total);
                        }
                    }

                    let percent = manifest
                        .total_bytes
                        .map(|total| percent_from_bytes(manifest.downloaded_bytes, total));
                    emit_steamcmd_transfer_progress(
                        &progress,
                        ParsedSteamCmdProgress {
                            phase: manifest.phase,
                            message: "SteamCMD 正在下载/更新服务端文件".to_string(),
                            percent,
                            downloaded_bytes: manifest.downloaded_bytes,
                            total_bytes: manifest.total_bytes,
                        },
                        Some(format!("manifest: {}", manifest_path.display())),
                        true,
                    );
                }
            }
            ticker.tick().await;
        }
    })
}

fn directory_size_bytes(path: &Path) -> Option<u64> {
    if !path.is_dir() {
        return None;
    }

    let mut total = 0_u64;
    let mut pending = vec![path.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(current) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                total = total.saturating_add(metadata.len());
            }
        }
    }

    Some(total)
}

#[cfg(windows)]
fn process_transfer_counters(process_id: Option<u32>) -> Option<ProcessTransferCounters> {
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
fn process_transfer_counters(_process_id: Option<u32>) -> Option<ProcessTransferCounters> {
    None
}

fn handle_steamcmd_progress_line(sink: &SteamCmdProgressSink, line: &str) -> bool {
    let Some(parsed) = parse_steamcmd_progress_line(line) else {
        return false;
    };
    emit_steamcmd_transfer_progress(sink, parsed, Some(line.to_string()), false);
    true
}

fn emit_steamcmd_transfer_progress(
    sink: &SteamCmdProgressSink,
    parsed: ParsedSteamCmdProgress,
    detail: Option<String>,
    force_emit: bool,
) {
    let now = Instant::now();
    let (snapshot, should_emit) = match sink.tracker.lock() {
        Ok(mut tracker) => tracker.update(
            parsed.downloaded_bytes,
            parsed.total_bytes,
            parsed.percent,
            now,
            force_emit,
        ),
        Err(_) => return,
    };

    if should_emit {
        let payload = JobProgress {
            job_id: format!("job-{}", now_millis()),
            instance_id: Some(sink.instance_id.clone()),
            phase: parsed.phase,
            percent: snapshot.percent,
            message: parsed.message,
            detail,
            downloaded_bytes: snapshot.downloaded_bytes,
            total_bytes: snapshot.total_bytes,
            bytes_per_second: snapshot.bytes_per_second,
        };
        let _ = sink.channel.send(payload.clone());
        let _ = sink.app.emit("asa:job-progress", payload);
    }
}

fn transfer_snapshot(sink: &SteamCmdProgressSink) -> TransferSnapshot {
    sink.tracker
        .lock()
        .map(|tracker| tracker.snapshot())
        .unwrap_or_default()
}

fn handle_command_output_line(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    level: &'static str,
    tail: &Arc<Mutex<VecDeque<String>>>,
    progress: Option<&SteamCmdProgressSink>,
    line: &str,
) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    remember_tail(tail, line);

    if progress
        .map(|sink| handle_steamcmd_progress_line(sink, line))
        .unwrap_or(false)
    {
        return;
    }

    let _ = emit_instance_log(app, runtime, instance_name, level, line);
}

fn spawn_command_log_reader<R>(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_name: &str,
    stream: Option<R>,
    level: &'static str,
    tail: Arc<Mutex<VecDeque<String>>>,
    progress: Option<SteamCmdProgressSink>,
) -> Option<JoinHandle<()>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let stream = stream?;
    let app = app.clone();
    let runtime = runtime.clone();
    let instance_name = instance_name.to_string();
    Some(tokio::spawn(async move {
        let mut stream = stream;
        let mut buffer = [0_u8; 4096];
        let mut pending = Vec::new();

        loop {
            let read = match stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };
            for byte in &buffer[..read] {
                if *byte == b'\r' || *byte == b'\n' {
                    let line = String::from_utf8_lossy(&pending);
                    handle_command_output_line(
                        &app,
                        &runtime,
                        &instance_name,
                        level,
                        &tail,
                        progress.as_ref(),
                        &line,
                    );
                    pending.clear();
                } else {
                    pending.push(*byte);
                    if pending.len() > 8192 {
                        let line = String::from_utf8_lossy(&pending);
                        handle_command_output_line(
                            &app,
                            &runtime,
                            &instance_name,
                            level,
                            &tail,
                            progress.as_ref(),
                            &line,
                        );
                        pending.clear();
                    }
                }
            }
        }

        if !pending.is_empty() {
            let line = String::from_utf8_lossy(&pending);
            handle_command_output_line(
                &app,
                &runtime,
                &instance_name,
                level,
                &tail,
                progress.as_ref(),
                &line,
            );
        }
    }))
}

#[cfg(windows)]
async fn kill_process_tree(pid: u32) {
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

    let _ = timeout(Duration::from_secs(5), command.status()).await;
}

#[cfg(not(windows))]
async fn kill_process_tree(_pid: u32) {}

async fn wait_for_log_readers(readers: Vec<Option<JoinHandle<()>>>) {
    for reader in readers.into_iter().flatten() {
        let _ = timeout(Duration::from_secs(2), reader).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_steamcmd_download_progress() {
        let parsed = parse_steamcmd_progress_line(
            "Update state (0x61) downloading, Progress: 12.34 (1234000 / 10000000)",
        )
        .expect("SteamCMD progress line should parse");

        assert_eq!(parsed.phase, "downloading");
        assert!(matches!(parsed.percent, Some(percent) if (percent - 12.34).abs() < 0.001));
        assert_eq!(parsed.downloaded_bytes, 1_234_000);
        assert_eq!(parsed.total_bytes, Some(10_000_000));
    }

    #[test]
    fn treats_zero_total_progress_as_unknown_size() {
        let parsed = parse_steamcmd_progress_line(
            "Update state (0x3) reconfiguring, progress: 0.00 (0 / 0)",
        )
        .expect("SteamCMD zero-size progress line should parse");

        assert_eq!(parsed.phase, "running");
        assert_eq!(parsed.percent, Some(0.0));
        assert_eq!(parsed.downloaded_bytes, 0);
        assert_eq!(parsed.total_bytes, None);
    }

    #[test]
    fn parses_manifest_download_progress() {
        let manifest = r#"
"AppState"
{
    "BytesToDownload"        "8248424336"
    "BytesDownloaded"        "1678229152"
    "BytesToStage"        "13202439198"
    "BytesStaged"        "4081357112"
}
"#;
        let parsed = parse_manifest_progress(manifest).expect("manifest progress should parse");

        assert_eq!(parsed.phase, "downloading");
        assert_eq!(parsed.downloaded_bytes, 1_678_229_152);
        assert_eq!(parsed.total_bytes, Some(8_248_424_336));
    }
}

fn open_directory(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .map_err(|error| format!("无法打开目录 {}：{error}", path.display()))?;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        Err("打开目录目前仅支持 Windows".to_string())
    }
}

#[allow(dead_code)]
fn path_text(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
