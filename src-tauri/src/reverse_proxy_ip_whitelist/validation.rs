use crate::models::{GlobalSettings, WEB_IP_WHITELIST_CHINA_MAINLAND, WebIpWhitelistEntry};

use super::{cidr::normalize_ipv4_whitelist_entry, normalization::normalize_ip_whitelist_entries};

pub(crate) fn validate_security_settings(settings: &GlobalSettings) -> Result<(), String> {
    if !(1..=100).contains(&settings.web_login_failure_ban_threshold) {
        return Err("Web 登录失败封禁阈值必须在 1-100 次之间".to_string());
    }
    if !(1..=86_400).contains(&settings.web_login_failure_ban_seconds) {
        return Err("Web 登录失败封禁时长必须在 1-86400 秒之间".to_string());
    }
    validate_ip_whitelist_entries(&settings.web_ip_whitelist)
}

pub(super) fn validate_ip_whitelist_entries(entries: &[WebIpWhitelistEntry]) -> Result<(), String> {
    for entry in normalize_ip_whitelist_entries(entries) {
        if entry.group.chars().count() > 32 {
            return Err(format!(
                "Web IP 白名单条目 {} 的分组不能超过 32 个字符",
                entry.value
            ));
        }
        if entry.note.chars().count() > 120 {
            return Err(format!(
                "Web IP 白名单条目 {} 的备注不能超过 120 个字符",
                entry.value
            ));
        }
        if entry
            .value
            .eq_ignore_ascii_case(WEB_IP_WHITELIST_CHINA_MAINLAND)
        {
            continue;
        }
        let _ = normalize_ipv4_whitelist_entry(&entry.value)?;
    }
    Ok(())
}
