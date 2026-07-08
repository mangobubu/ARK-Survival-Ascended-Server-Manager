use crate::models::GlobalSettings;
use serde_json::{Value, json};

pub(crate) fn public_settings(mut settings: GlobalSettings) -> Result<Value, String> {
    let web_admin_password_configured =
        !settings.web_admin_username.trim().is_empty() && !settings.web_admin_password.is_empty();
    let web_acme_tencent_secret_key_configured =
        !settings.web_acme_tencent_secret_key.trim().is_empty();
    settings.web_admin_password.clear();
    settings.web_acme_tencent_secret_key.clear();

    let mut value =
        serde_json::to_value(settings).map_err(|error| format!("序列化全局设置失败：{error}"))?;
    let Value::Object(map) = &mut value else {
        return Err("序列化全局设置失败：结果不是对象".to_string());
    };
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
