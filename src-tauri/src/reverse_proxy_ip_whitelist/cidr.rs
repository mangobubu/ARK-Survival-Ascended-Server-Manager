use std::net::Ipv4Addr;

pub(super) fn default_mainland_cidr_file() -> &'static str {
    include_str!("../../resources/asa-mainland-cidrs.txt")
}

pub(super) fn clean_cidr_line(line: &str) -> Option<&str> {
    let cidr = line
        .split_once('#')
        .map(|(value, _)| value)
        .unwrap_or(line)
        .trim();
    (!cidr.is_empty()).then_some(cidr)
}

pub(super) fn normalize_ipv4_whitelist_entry(entry: &str) -> Result<String, String> {
    if let Ok(ip) = entry.parse::<Ipv4Addr>() {
        return Ok(format!("{ip}/32"));
    }

    let Some((ip_text, prefix_text)) = entry.split_once('/') else {
        return Err(format!(
            "Web IP 白名单条目无效：{entry}，仅支持 CN_MAINLAND、IPv4 或 IPv4 CIDR"
        ));
    };
    if prefix_text.contains('/') {
        return Err(format!(
            "Web IP 白名单条目无效：{entry}，CIDR 前缀格式不正确"
        ));
    }

    let ip = ip_text
        .parse::<Ipv4Addr>()
        .map_err(|_| format!("Web IP 白名单条目无效：{entry}，IP 必须是 IPv4 地址"))?;
    let prefix = prefix_text
        .parse::<u8>()
        .map_err(|_| format!("Web IP 白名单条目无效：{entry}，CIDR 前缀必须是 0-32"))?;
    if prefix > 32 {
        return Err(format!(
            "Web IP 白名单条目无效：{entry}，CIDR 前缀必须是 0-32"
        ));
    }

    Ok(format!("{ip}/{prefix}"))
}
