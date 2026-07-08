use crate::{
    ark_config_game_user_settings::render_game_user_settings,
    ark_config_ini::{render_engine_ini, render_game_ini},
    ark_config_launch,
    models::{ModItem, ServerInstance},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct AppliedConfig {
    pub config_dir: PathBuf,
    pub game_user_settings_path: PathBuf,
    pub game_ini_path: PathBuf,
    pub engine_ini_path: PathBuf,
    pub launch_arguments: Vec<String>,
}

pub fn validate_visibility_access(config: &Value) -> Result<(), String> {
    crate::ark_config_values::validate_visibility_access(config)
}

pub fn apply_instance_config(
    instance: &ServerInstance,
    config: &Value,
    mods: &[ModItem],
) -> Result<AppliedConfig, String> {
    validate_visibility_access(config)?;

    let config_dir = config_dir(instance);
    fs::create_dir_all(&config_dir)
        .map_err(|error| format!("无法创建 ARK 配置目录 {}：{error}", config_dir.display()))?;

    let game_user_settings_path = config_dir.join("GameUserSettings.ini");
    let game_ini_path = config_dir.join("Game.ini");
    let engine_ini_path = config_dir.join("Engine.ini");
    fs::write(
        &game_user_settings_path,
        render_game_user_settings(instance, config, mods),
    )
    .map_err(|error| {
        format!(
            "无法写入 GameUserSettings.ini {}：{error}",
            game_user_settings_path.display()
        )
    })?;
    fs::write(&game_ini_path, render_game_ini(config))
        .map_err(|error| format!("无法写入 Game.ini {}：{error}", game_ini_path.display()))?;
    fs::write(&engine_ini_path, render_engine_ini(config))
        .map_err(|error| format!("无法写入 Engine.ini {}：{error}", engine_ini_path.display()))?;

    Ok(AppliedConfig {
        config_dir,
        game_user_settings_path,
        game_ini_path,
        engine_ini_path,
        launch_arguments: build_launch_arguments(instance, config, mods),
    })
}

pub fn config_dir(instance: &ServerInstance) -> PathBuf {
    Path::new(&instance.install_path)
        .join("ShooterGame")
        .join("Saved")
        .join("Config")
        .join("WindowsServer")
}

pub fn saved_dir(instance: &ServerInstance) -> PathBuf {
    Path::new(&instance.install_path)
        .join("ShooterGame")
        .join("Saved")
}

pub fn server_executable(instance: &ServerInstance) -> Option<PathBuf> {
    let root = Path::new(&instance.install_path);
    [
        root.join("ShooterGame")
            .join("Binaries")
            .join("Win64")
            .join("ArkAscendedServer.exe"),
        root.join("ShooterGame")
            .join("Binaries")
            .join("Win64")
            .join("ShooterGameServer.exe"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

pub fn build_launch_arguments(
    instance: &ServerInstance,
    config: &Value,
    mods: &[ModItem],
) -> Vec<String> {
    ark_config_launch::build_launch_arguments(instance, config, mods)
}

#[cfg(test)]
mod tests;
