use super::{
    cidr::default_mainland_cidr_file, render_ip_whitelist_cidr_file,
    validation::validate_ip_whitelist_entries,
};
use crate::models::{GlobalSettings, WEB_IP_WHITELIST_CHINA_MAINLAND, WebIpWhitelistEntry};

#[test]
fn 默认全局设置使用中国大陆_ip_白名单() {
    assert_eq!(
        GlobalSettings::default().web_ip_whitelist,
        vec![WebIpWhitelistEntry::new(
            WEB_IP_WHITELIST_CHINA_MAINLAND,
            "默认",
            "内置中国大陆 IPv4 CIDR"
        )]
    );
}

#[test]
fn 旧版字符串白名单会反序列化为结构化条目() {
    let mut value = serde_json::to_value(GlobalSettings::default()).unwrap();
    value["webIpWhitelist"] = serde_json::json!([WEB_IP_WHITELIST_CHINA_MAINLAND, "203.0.113.10"]);

    let settings = serde_json::from_value::<GlobalSettings>(value).unwrap();

    assert_eq!(
        settings.web_ip_whitelist[0],
        WebIpWhitelistEntry::new(WEB_IP_WHITELIST_CHINA_MAINLAND, "", "")
    );
    assert_eq!(
        settings.web_ip_whitelist[1],
        WebIpWhitelistEntry::new("203.0.113.10", "", "")
    );
}

#[test]
fn 默认中国大陆_cidr_准入列表不是空模板() {
    let cidrs: Vec<&str> = default_mainland_cidr_file()
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    assert!(cidrs.len() > 1_000);
    assert!(cidrs.contains(&"1.0.1.0/24"));
    assert!(cidrs.iter().all(|cidr| cidr.contains('/')));
}

#[test]
fn 渲染_ip_白名单会展开中国大陆_cidr() {
    let rendered = render_ip_whitelist_cidr_file(&[WebIpWhitelistEntry::new(
        WEB_IP_WHITELIST_CHINA_MAINLAND,
        "默认",
        "内置中国大陆 IPv4 CIDR",
    )])
    .expect("应该能渲染中国大陆白名单");
    let cidrs: Vec<&str> = rendered
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    assert!(cidrs.len() > 1_000);
    assert!(cidrs.contains(&"1.0.1.0/24"));
    assert!(rendered.contains("持久化来源：全局设置 webIpWhitelist"));
    assert!(rendered.contains("分组：默认"));
}

#[test]
fn 渲染_ip_白名单支持单_ip_和_cidr() {
    let rendered = render_ip_whitelist_cidr_file(&[
        WebIpWhitelistEntry::new("203.0.113.10", "运维", "办公室出口"),
        WebIpWhitelistEntry::new("203.0.113.0/24", "临时", "测试网段"),
        WebIpWhitelistEntry::new("203.0.113.10", "重复", "应去重"),
    ])
    .expect("应该能渲染自定义白名单");

    assert!(rendered.contains("203.0.113.10/32"));
    assert!(rendered.contains("203.0.113.0/24"));
    assert!(rendered.contains("分组：运维"));
    assert!(rendered.contains("备注：办公室出口"));
    assert_eq!(rendered.matches("203.0.113.10/32").count(), 1);
}

#[test]
fn ip_白名单拒绝非法条目() {
    assert!(
        validate_ip_whitelist_entries(&[WebIpWhitelistEntry::new("example.com", "", "")]).is_err()
    );
    assert!(
        validate_ip_whitelist_entries(&[WebIpWhitelistEntry::new("203.0.113.0/33", "", "")])
            .is_err()
    );
    assert!(
        render_ip_whitelist_cidr_file(&[WebIpWhitelistEntry::new("203.0.113.0/test", "", "")])
            .is_err()
    );
}
