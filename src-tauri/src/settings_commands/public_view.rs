use crate::models::GlobalSettings;
use serde_json::{Value, json};

pub(crate) fn public_settings(mut settings: GlobalSettings) -> Result<Value, String> {
    let curseforge_api_key_configured = !settings.curseforge_api_key.trim().is_empty()
        || std::env::var("CURSEFORGE_API_KEY").is_ok_and(|value| !value.trim().is_empty());
    let web_admin_password_configured =
        !settings.web_admin_username.trim().is_empty() && !settings.web_admin_password.is_empty();
    let web_acme_tencent_secret_key_configured =
        !settings.web_acme_tencent_secret_key.trim().is_empty();
    settings.curseforge_api_key.clear();
    settings.web_admin_password.clear();
    settings.web_acme_tencent_secret_key.clear();

    let mut value =
        serde_json::to_value(settings).map_err(|error| format!("序列化全局设置失败：{error}"))?;
    let Value::Object(map) = &mut value else {
        return Err("序列化全局设置失败：结果不是对象".to_string());
    };
    map.insert(
        "curseforgeApiKeyConfigured".to_string(),
        json!(curseforge_api_key_configured),
    );
    map.insert(
        "webAdminPasswordConfigured".to_string(),
        json!(web_admin_password_configured),
    );
    map.insert(
        "webAcmeTencentSecretKeyConfigured".to_string(),
        json!(web_acme_tencent_secret_key_configured),
    );
    Ok(value)
}
