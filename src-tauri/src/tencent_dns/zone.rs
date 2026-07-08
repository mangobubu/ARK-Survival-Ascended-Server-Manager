pub fn split_for_dnspod_zone(fqdn: &str) -> Result<(String, String), String> {
    let labels = fqdn
        .trim()
        .trim_end_matches('.')
        .split('.')
        .filter(|label| !label.is_empty())
        .collect::<Vec<_>>();
    if labels.len() < 2 {
        return Err(format!("域名 {fqdn} 无法拆分为 DNSPod 主域名和子域名"));
    }
    let zone_labels = if labels.len() >= 3
        && is_common_second_level_suffix(labels[labels.len() - 2], labels[labels.len() - 1])
    {
        3
    } else {
        2
    };
    if labels.len() < zone_labels {
        return Err(format!("域名 {fqdn} 无法识别主域名"));
    }
    let domain = labels[labels.len() - zone_labels..].join(".");
    let sub_domain = labels[..labels.len() - zone_labels].join(".");
    Ok((domain, sub_domain))
}

fn is_common_second_level_suffix(second: &str, top: &str) -> bool {
    matches!(
        (second, top),
        ("com", "cn")
            | ("net", "cn")
            | ("org", "cn")
            | ("gov", "cn")
            | ("edu", "cn")
            | ("co", "uk")
            | ("org", "uk")
            | ("ac", "uk")
    )
}

pub fn acme_challenge_record_for_domain(domain: &str) -> Result<(String, String), String> {
    let challenge_fqdn = format!("_acme-challenge.{}", domain.trim().trim_end_matches('.'));
    split_for_dnspod_zone(&challenge_fqdn)
}
