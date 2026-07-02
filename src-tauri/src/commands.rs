use crate::{
    app_state::{AppRuntime, now_millis},
    ark_config,
    backup,
    import_export,
    models::{
        AddInstancePayload, ASA_DEDICATED_SERVER_APP_ID, BackupItem, ExportResult, GlobalSettings,
        ImportResult, JobProgress, LogLine, ModItem, ServerInstance, ServerStatus,
    },
    rcon,
};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tauri::{AppHandle, Emitter, State, ipc::Channel};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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
        return Err(format!("SteamCMD 不存在：{}", steamcmd.display()));
    }
    std::fs::create_dir_all(&instance.install_path)
        .map_err(|error| format!("无法创建实例目录 {}：{error}", instance.install_path))?;

    emit_progress(
        &progress,
        &instance.id,
        "preparing",
        Some(5),
        "正在准备服务端安装/更新",
        None,
    )?;
    runtime.update_instance_status(&instance.id, ServerStatus::Updating, None)?;
    emit_status(&app, &runtime, &instance.id)?;

    let output = run_steamcmd_update(&steamcmd, Path::new(&instance.install_path), &progress, &instance).await;
    match output {
        Ok(detail) => {
            emit_progress(
                &progress,
                &instance.id,
                "completed",
                Some(100),
                "服务端安装/更新完成",
                Some(detail),
            )?;
            instance.version_state = "已安装/已更新".to_string();
            instance.status = ServerStatus::Stopped;
            instance.last_error = None;
            let updated = runtime.upsert_instance(instance.clone())?;
            runtime.add_log(&updated.name, "success", "服务端安装/更新完成")?;
            emit_status(&app, &runtime, &updated.id)?;
            Ok(updated)
        }
        Err(error) => {
            runtime.update_instance_status(
                &instance.id,
                ServerStatus::Error,
                Some(error.clone()),
            )?;
            runtime.add_log(&instance.name, "error", &format!("服务端安装/更新失败：{error}"))?;
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
    let backup = backup::create_instance_backup(Path::new(&settings.backup_storage_path), &instance)?;
    runtime.add_log(&instance.name, "success", &format!("备份已创建：{}", backup.path))?;
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
    runtime.add_log(&instance.name, "warn", &format!("已恢复备份：{backup_path}"))?;
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
    steamcmd: &Path,
    install_path: &Path,
    progress: &Channel<JobProgress>,
    instance: &ServerInstance,
) -> Result<String, String> {
    emit_progress(
        progress,
        &instance.id,
        "running",
        Some(30),
        "正在调用 SteamCMD 安装/更新 ASA Dedicated Server",
        None,
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
        let output = timeout(Duration::from_secs(60 * 60), command.output())
            .await
            .map_err(|_| "SteamCMD 安装/更新超时（60 分钟）".to_string())?
            .map_err(|error| format!("无法启动 SteamCMD：{error}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");
        if !output.status.success() {
            return Err(trim_detail(&combined, "SteamCMD 安装/更新失败"));
        }
        emit_progress(
            progress,
            &instance.id,
            "verifying",
            Some(88),
            "正在验证服务端文件",
            None,
        )?;
        if ark_config::server_executable(instance).is_none() {
            return Err("SteamCMD 执行完成，但未找到 ASA 服务端可执行文件".to_string());
        }
        Ok(trim_detail(&combined, "SteamCMD 安装/更新完成"))
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
        .current_dir(executable.parent().unwrap_or_else(|| Path::new(&instance.install_path)))
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

fn emit_status(
    app: &AppHandle,
    runtime: &AppRuntime,
    instance_id: &str,
) -> Result<(), String> {
    let instance = runtime.get_instance(instance_id)?;
    app.emit("asa:instance-status", instance)
        .map_err(|error| format!("发送实例状态事件失败：{error}"))
}

fn emit_progress(
    channel: &Channel<JobProgress>,
    instance_id: &str,
    phase: &str,
    percent: Option<u8>,
    message: &str,
    detail: Option<String>,
) -> Result<(), String> {
    channel
        .send(JobProgress {
            job_id: format!("job-{}", now_millis()),
            instance_id: Some(instance_id.to_string()),
            phase: phase.to_string(),
            percent,
            message: message.to_string(),
            detail,
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
