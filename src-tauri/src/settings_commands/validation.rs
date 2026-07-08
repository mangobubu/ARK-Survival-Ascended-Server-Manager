use crate::{
    models::{GlobalSettings, WindowCloseBehavior},
    reverse_proxy, window_controls,
};
use std::path::Path;

pub(super) fn validate_settings(settings: &GlobalSettings) -> Result<(), String> {
    if settings.steam_cmd_path.trim().is_empty() {
        return Err("SteamCMD 目录不能为空".to_string());
    }
    if settings.server_storage_path.trim().is_empty() {
        return Err("服务器存储目录不能为空".to_string());
    }
    if settings.backup_storage_path.trim().is_empty() {
        return Err("备份存储目录不能为空".to_string());
    }
    validate_optional_directory(&settings.server_storage_path, "服务器存储目录")?;
    validate_optional_directory(&settings.backup_storage_path, "备份存储目录")?;
    if !matches!(settings.language.as_str(), "zh-CN" | "en-US") {
        return Err("应用语言仅支持 zh-CN 或 en-US".to_string());
    }
    if !matches!(settings.theme.as_str(), "dark" | "light" | "system") {
        return Err("应用主题仅支持 dark、light 或 system".to_string());
    }
    if !matches!(
        settings.window_close_behavior,
        WindowCloseBehavior::AskEveryTime
            | WindowCloseBehavior::MinimizeToTray
            | WindowCloseBehavior::ExitApp
    ) {
        return Err("窗口关闭行为无效".to_string());
    }
    if !window_controls::is_supported_shortcut_key(&settings.global_toggle_shortcut_key) {
        return Err("快捷键必须是 Ctrl + Alt + 一个字母、数字、F1-F24 或常用功能键".to_string());
    }
    if !(1..=100).contains(&settings.max_backup_retention) {
        return Err("自动备份保留数量必须在 1-100 之间".to_string());
    }
    if !(1024..=65535).contains(&settings.web_server_port) {
        return Err("Web 访问端口必须在 1024-65535 之间".to_string());
    }
    if settings.web_admin_username.trim().len() > 64 {
        return Err("Web 管理员账号不能超过 64 个字符".to_string());
    }
    if settings.web_admin_password.len() > 128 {
        return Err("Web 管理员密码不能超过 128 个字符".to_string());
    }
    if settings.web_admin_username.trim().is_empty() && !settings.web_admin_password.is_empty() {
        return Err("设置 Web 管理员密码时，管理员账号不能为空".to_string());
    }
    if settings.web_https_enabled && !settings.web_reverse_proxy_enabled {
        return Err("启用 HTTPS 时必须同时启用域名反向代理".to_string());
    }
    if settings.web_acme_auto_issue_enabled && !settings.web_https_enabled {
        return Err("启用 Let's Encrypt 自动申请/续期时必须先启用 HTTPS".to_string());
    }
    if settings.web_acme_auto_issue_enabled {
        if settings.web_acme_account_email.trim().is_empty()
            || !settings.web_acme_account_email.contains('@')
        {
            return Err("Let's Encrypt ACME 账户邮箱格式无效".to_string());
        }
        if settings.web_acme_directory_url.trim() != crate::models::default_web_acme_directory_url()
        {
            return Err("当前仅支持 Let's Encrypt 正式环境 ACME 目录地址".to_string());
        }
        if settings.web_acme_tencent_secret_id.trim().is_empty() {
            return Err("启用 ACME DNS-01 自动申请时必须填写腾讯云 Secret ID".to_string());
        }
        if settings.web_acme_tencent_secret_key.trim().is_empty() {
            return Err("启用 ACME DNS-01 自动申请时必须填写腾讯云 Secret Key".to_string());
        }
    }
    let web_captcha_visible_chars = settings
        .web_captcha_charset
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .count();
    if web_captcha_visible_chars < 2 {
        return Err("Web 验证码字符池至少需要包含 2 个可见字符".to_string());
    }
    if settings.web_captcha_charset.chars().count() > 128 {
        return Err("Web 验证码字符池不能超过 128 个字符".to_string());
    }
    if !(1..=12).contains(&settings.web_captcha_length) {
        return Err("Web 验证码字符数量必须在 1-12 之间".to_string());
    }
    if !(18..=56).contains(&settings.web_captcha_font_size) {
        return Err("Web 验证码字体大小必须在 18-56 之间".to_string());
    }
    if settings.web_captcha_noise_points > 120 {
        return Err("Web 验证码杂点数量必须在 0-120 之间".to_string());
    }
    reverse_proxy::validate_settings(settings)?;
    Ok(())
}

fn validate_optional_directory(path: &str, label: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.exists() && !path.is_dir() {
        return Err(format!("{label}不是目录：{}", path.display()));
    }
    Ok(())
}
