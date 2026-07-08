use crate::{
    instance_config_import_ini::parse_ini_file,
    instance_config_import_mapping::{
        bool_from_config, text_from_config, u16_from_config, u32_from_config,
    },
    instance_config_import_paths::{
        infer_install_path, infer_map_from_saved_arks, locate_config_dir, path_text,
    },
    instance_config_import_readers::{read_engine_ini, read_game_ini, read_game_user_settings},
    models::ImportedServerConfigPreview,
};
use serde_json::{Map, Value};
use std::{fs, path::Path};

pub fn read_server_directory_config(path: &Path) -> Result<ImportedServerConfigPreview, String> {
    validate_selected_directory(path)?;

    let selected_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let config_dir = locate_config_dir(&selected_path);
    let install_path = infer_install_path(&selected_path, &config_dir);
    let game_user_settings_path = config_dir.join("GameUserSettings.ini");
    let game_ini_path = config_dir.join("Game.ini");
    let engine_ini_path = config_dir.join("Engine.ini");

    let mut config = Map::new();
    let mut found_files = Vec::new();
    let mut warnings = Vec::new();
    let mut mods = Vec::new();

    if game_user_settings_path.is_file() {
        let document = parse_ini_file(&game_user_settings_path)?;
        found_files.push(path_text(&game_user_settings_path));
        read_game_user_settings(&document, &mut config, &mut mods);
    } else {
        warnings.push(format!(
            "未找到 GameUserSettings.ini：{}",
            path_text(&game_user_settings_path)
        ));
    }

    if game_ini_path.is_file() {
        let document = parse_ini_file(&game_ini_path)?;
        found_files.push(path_text(&game_ini_path));
        read_game_ini(&document, &mut config);
    } else {
        warnings.push(format!("未找到 Game.ini：{}", path_text(&game_ini_path)));
    }

    if engine_ini_path.is_file() {
        let document = parse_ini_file(&engine_ini_path)?;
        found_files.push(path_text(&engine_ini_path));
        read_engine_ini(&document, &mut config);
    }

    if found_files.is_empty() {
        warnings.push("未在所选目录下找到 ASA 服务端配置文件".to_string());
    }

    let mut preview = build_preview(&install_path, config, mods, found_files, warnings);
    fill_missing_preview_name(&mut preview, &install_path);
    fill_preview_map_from_saved_arks(&mut preview, &install_path);

    Ok(preview)
}

fn validate_selected_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("服务端目录不存在：{}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!(
            "请选择服务端文件夹，而不是文件：{}",
            path.display()
        ));
    }
    Ok(())
}

fn build_preview(
    install_path: &Path,
    config: Map<String, Value>,
    mods: Vec<crate::models::ModItem>,
    found_files: Vec<String>,
    warnings: Vec<String>,
) -> ImportedServerConfigPreview {
    ImportedServerConfigPreview {
        install_path: path_text(install_path),
        name: text_from_config(&config, "sessionName"),
        map: None,
        map_code: None,
        mode: bool_from_config(&config, "pve").map(|pve| {
            if pve {
                "PvE".to_string()
            } else {
                "PvP".to_string()
            }
        }),
        game_port: u16_from_config(&config, "gamePort"),
        query_port: u16_from_config(&config, "queryPort"),
        rcon_port: u16_from_config(&config, "rconPort"),
        max_players: u32_from_config(&config, "maxPlayers"),
        cluster_id: text_from_config(&config, "clusterId"),
        server_password: text_from_config(&config, "serverPassword"),
        admin_password: text_from_config(&config, "adminPassword"),
        config: Value::Object(config),
        mods,
        found_files,
        warnings,
    }
}

fn fill_missing_preview_name(preview: &mut ImportedServerConfigPreview, install_path: &Path) {
    if preview
        .name
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && !preview.found_files.is_empty()
    {
        preview.name = install_path
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToString::to_string);
    }
}

fn fill_preview_map_from_saved_arks(
    preview: &mut ImportedServerConfigPreview,
    install_path: &Path,
) {
    if let Some((map_code, map_name)) = infer_map_from_saved_arks(install_path) {
        preview.map_code = Some(map_code);
        preview.map = Some(map_name);
    }
}
