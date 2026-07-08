use crate::{app_state::AppRuntime, models::GlobalSettings, web_auth};

pub(super) fn merge_settings_update(
    runtime: &AppRuntime,
    settings: GlobalSettings,
) -> Result<GlobalSettings, String> {
    let current = runtime.settings()?;
    prepare_settings_for_save(&current, settings)
}

pub(crate) fn prepare_settings_for_save(
    current: &GlobalSettings,
    mut settings: GlobalSettings,
) -> Result<GlobalSettings, String> {
    if settings.web_admin_username.trim().is_empty() && settings.web_admin_password.is_empty() {
        settings.web_admin_password.clear();
    } else if settings.web_admin_password.is_empty() {
        settings.web_admin_password = current.web_admin_password.clone();
    } else {
        if settings.web_admin_password.len() > 128 {
            return Err("Web 管理员密码不能超过 128 个字符".to_string());
        }
        settings.web_admin_password =
            web_auth::hash_web_admin_password(&settings.web_admin_password)?;
    }

    if settings.web_acme_tencent_secret_key.trim().is_empty()
        && !current.web_acme_tencent_secret_key.trim().is_empty()
    {
        settings.web_acme_tencent_secret_key = current.web_acme_tencent_secret_key.clone();
    }

    Ok(settings)
}
