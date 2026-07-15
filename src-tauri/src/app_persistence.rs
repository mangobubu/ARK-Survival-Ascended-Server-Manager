mod settings_config;
mod state_file;

pub(crate) use settings_config::{
    settings_config_contains_unprotected_secret, settings_config_path,
};

use crate::{app_state::ManagerData, web_auth};
use std::{fs, path::Path};

pub(crate) const STATE_FILE_NAME: &str = "manager-state.json";
pub(crate) const SETTINGS_FILE_NAME: &str = "config.toml";

pub(crate) fn read_data(data_dir: &Path) -> Result<ManagerData, String> {
    let state = state_file::read_state_file(data_dir)?;
    let settings = if settings_config::settings_config_path(data_dir).exists() {
        settings_config::read_settings_config(data_dir)?
    } else {
        let mut settings = state.settings.unwrap_or_default();
        settings_config::restore_settings_from_storage(&mut settings)?;
        settings
    };

    Ok(ManagerData {
        settings,
        instances: state.instances,
        configs: state.configs,
        mods: state.mods,
        logs: state.logs,
    })
}

pub(crate) fn write_data(data_dir: &Path, data: &ManagerData) -> Result<(), String> {
    fs::create_dir_all(data_dir)
        .map_err(|error| format!("无法创建应用数据目录 {}：{error}", data_dir.display()))?;
    settings_config::write_settings_config(data_dir, &data.settings)?;
    state_file::write_state_file(data_dir, data)
}

pub(crate) fn migrate_manager_data_secrets(data: &mut ManagerData) -> Result<bool, String> {
    web_auth::migrate_web_admin_password_hash(&mut data.settings.web_admin_password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GlobalSettings;

    #[test]
    fn 自动迁移_web_管理员明文密码为哈希() {
        let mut data = ManagerData::default();
        data.settings.web_admin_username = "admin".to_string();
        data.settings.web_admin_password = "plain-secret".to_string();

        assert!(migrate_manager_data_secrets(&mut data).expect("迁移明文 Web 密码"));
        assert_ne!(data.settings.web_admin_password, "plain-secret");
        assert!(web_auth::is_web_admin_password_hash(
            &data.settings.web_admin_password
        ));
        assert!(
            web_auth::verify_web_admin_password("plain-secret", &data.settings.web_admin_password)
                .expect("校验迁移后的 Web 密码")
        );
    }

    #[test]
    fn 全局设置写入_config_toml_且状态文件不再重复保存() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let mut data = ManagerData::default();
        data.settings.theme = "light".to_string();
        data.settings.language = "en-US".to_string();

        write_data(temp.path(), &data).expect("写入拆分后的管理器数据");

        let config_toml =
            fs::read_to_string(temp.path().join(SETTINGS_FILE_NAME)).expect("读取 config.toml");
        assert!(config_toml.contains("theme = \"light\""));
        assert!(config_toml.contains("language = \"en-US\""));

        let state_json =
            fs::read_to_string(temp.path().join(STATE_FILE_NAME)).expect("读取状态文件");
        assert!(!state_json.contains("\"settings\""));

        let loaded = read_data(temp.path()).expect("重新读取拆分后的管理器数据");
        assert_eq!(loaded.settings.theme, "light");
        assert_eq!(loaded.settings.language, "en-US");
    }

    #[cfg(windows)]
    #[test]
    fn 腾讯云_secret_key_写入_config_toml_前会使用_dpapi_保护() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let mut data = ManagerData::default();
        data.settings.web_acme_tencent_secret_id = "AKIDTEST".to_string();
        data.settings.web_acme_tencent_secret_key = "very-secret-key-123".to_string();

        write_data(temp.path(), &data).expect("写入配置");

        let config_toml =
            fs::read_to_string(temp.path().join(SETTINGS_FILE_NAME)).expect("读取 config.toml");
        assert!(!config_toml.contains("very-secret-key-123"));
        assert!(config_toml.contains("webAcmeTencentSecretKey = \"asa-dpapi-v1$"));
        assert!(!settings_config_contains_unprotected_secret(temp.path()).expect("检查明文凭据"));

        let loaded = read_data(temp.path()).expect("读取加密配置");
        assert_eq!(
            loaded.settings.web_acme_tencent_secret_key,
            "very-secret-key-123"
        );
    }

    #[cfg(windows)]
    #[test]
    fn curseforge_api_key_写入_config_toml_前会使用_dpapi_保护() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let mut data = ManagerData::default();
        data.settings.curseforge_api_key = "curseforge-secret-key-123".to_string();

        write_data(temp.path(), &data).expect("写入配置");

        let config_toml =
            fs::read_to_string(temp.path().join(SETTINGS_FILE_NAME)).expect("读取 config.toml");
        assert!(!config_toml.contains("curseforge-secret-key-123"));
        assert!(config_toml.contains("curseforgeApiKey = \"asa-dpapi-v1$"));
        assert!(!settings_config_contains_unprotected_secret(temp.path()).expect("检查明文凭据"));

        let loaded = read_data(temp.path()).expect("读取加密配置");
        assert_eq!(
            loaded.settings.curseforge_api_key,
            "curseforge-secret-key-123"
        );
    }

    #[test]
    fn 能识别旧版明文腾讯云_secret_key_需要重写保护() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let settings = GlobalSettings {
            web_acme_tencent_secret_id: "AKIDTEST".to_string(),
            web_acme_tencent_secret_key: "legacy-secret-key".to_string(),
            ..GlobalSettings::default()
        };
        let content = toml::to_string_pretty(&settings).expect("序列化旧版明文配置");
        fs::write(temp.path().join(SETTINGS_FILE_NAME), content).expect("写入旧版明文配置");

        assert!(
            settings_config_contains_unprotected_secret(temp.path()).expect("检查旧版明文凭据")
        );
    }

    #[test]
    fn 旧状态文件中的_settings_可迁移到_config_toml() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let mut legacy = ManagerData::default();
        legacy.settings.theme = "light".to_string();
        fs::write(
            temp.path().join(STATE_FILE_NAME),
            serde_json::to_string_pretty(&legacy).expect("序列化旧状态文件"),
        )
        .expect("写入旧状态文件");

        let loaded = read_data(temp.path()).expect("读取旧状态文件");
        assert_eq!(loaded.settings.theme, "light");

        write_data(temp.path(), &loaded).expect("写入迁移后的配置文件");
        let config_toml =
            fs::read_to_string(temp.path().join(SETTINGS_FILE_NAME)).expect("读取 config.toml");
        assert!(config_toml.contains("theme = \"light\""));
    }
}
