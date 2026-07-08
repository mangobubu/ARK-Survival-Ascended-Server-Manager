use crate::models::GlobalSettings;

pub fn should_enforce_proxy_host(settings: &GlobalSettings) -> bool {
    settings.web_management_enabled
        && settings.web_reverse_proxy_enabled
        && !settings.web_reverse_proxy_domain.trim().is_empty()
        && settings.web_reverse_proxy_port > 0
}

pub fn expected_proxy_host(settings: &GlobalSettings) -> Option<String> {
    if !should_enforce_proxy_host(settings) {
        return None;
    }
    let domain = normalize_domain(&settings.web_reverse_proxy_domain).ok()?;
    Some(format!("{domain}:{}", settings.web_reverse_proxy_port))
}

pub fn is_request_host_allowed(settings: &GlobalSettings, request_host: Option<&str>) -> bool {
    let Some(expected) = expected_proxy_host(settings) else {
        return true;
    };
    normalize_host_header(request_host.unwrap_or_default()).is_some_and(|host| host == expected)
}

pub(crate) fn normalize_domain(domain: &str) -> Result<String, String> {
    let normalized = domain.trim().trim_end_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("启用 Web 反向代理时必须填写访问域名".to_string());
    }
    if normalized.contains("://")
        || normalized.contains('/')
        || normalized.contains('\\')
        || normalized.contains(':')
        || normalized.contains('*')
    {
        return Err("访问域名只填写主机名，不包含协议、端口、路径或通配符".to_string());
    }
    if normalized.len() > 253 {
        return Err("访问域名不能超过 253 个字符".to_string());
    }
    if !normalized.split('.').all(is_valid_domain_label) {
        return Err("访问域名格式无效，只支持字母、数字、短横线和点号".to_string());
    }
    Ok(normalized)
}

fn is_valid_domain_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn normalize_host_header(host: &str) -> Option<String> {
    let value = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if value.is_empty() || value.contains('/') || value.contains('\\') {
        return None;
    }

    let (host, port) = value.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    let domain = normalize_domain(host).ok()?;
    Some(format!("{domain}:{port}"))
}
