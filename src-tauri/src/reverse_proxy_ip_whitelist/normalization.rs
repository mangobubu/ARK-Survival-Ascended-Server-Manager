use crate::models::{WEB_IP_WHITELIST_CHINA_MAINLAND, WebIpWhitelistEntry};
use std::collections::HashSet;

pub(crate) fn normalize_ip_whitelist_entries(
    entries: &[WebIpWhitelistEntry],
) -> Vec<WebIpWhitelistEntry> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for entry in entries {
        let value = entry.value.trim();
        if value.is_empty() {
            continue;
        }
        let normalized_value = if value.eq_ignore_ascii_case(WEB_IP_WHITELIST_CHINA_MAINLAND) {
            WEB_IP_WHITELIST_CHINA_MAINLAND.to_string()
        } else {
            value.to_string()
        };
        let normalized_entry = WebIpWhitelistEntry::new(
            normalized_value,
            entry.group.trim().to_string(),
            entry.note.trim().to_string(),
        );

        if seen.insert(normalized_entry.value.clone()) {
            normalized.push(normalized_entry);
        }
    }

    if normalized.is_empty() {
        vec![WebIpWhitelistEntry::new(
            WEB_IP_WHITELIST_CHINA_MAINLAND,
            "默认",
            "内置中国大陆 IPv4 CIDR",
        )]
    } else {
        normalized
    }
}
