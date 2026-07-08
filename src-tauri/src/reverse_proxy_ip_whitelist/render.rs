use crate::models::WEB_IP_WHITELIST_CHINA_MAINLAND;
use std::collections::HashSet;

use super::{
    cidr::{clean_cidr_line, default_mainland_cidr_file, normalize_ipv4_whitelist_entry},
    normalization::normalize_ip_whitelist_entries,
};
use crate::models::WebIpWhitelistEntry;

pub(crate) fn render_ip_whitelist_cidr_file(
    entries: &[WebIpWhitelistEntry],
) -> Result<String, String> {
    let mut rendered = String::from(
        "# 由 ASA 服务端管理器自动生成，请勿手动编辑。\n\
         # 持久化来源：全局设置 webIpWhitelist；本文件只是运行时派生 CIDR 文件。\n\
         # CN_MAINLAND 表示内置中国大陆 IPv4 CIDR；单个 IPv4 会自动转换为 /32。\n",
    );
    let mut seen = HashSet::new();

    for entry in normalize_ip_whitelist_entries(entries) {
        let group = entry.group.trim();
        let note = entry.note.trim();
        if !group.is_empty() || !note.is_empty() {
            rendered.push_str(&format!(
                "# 条目：{}{}{}\n",
                entry.value,
                if group.is_empty() {
                    String::new()
                } else {
                    format!("；分组：{group}")
                },
                if note.is_empty() {
                    String::new()
                } else {
                    format!("；备注：{note}")
                }
            ));
        }

        if entry
            .value
            .eq_ignore_ascii_case(WEB_IP_WHITELIST_CHINA_MAINLAND)
        {
            rendered.push_str("# CN_MAINLAND BEGIN\n");
            for cidr in default_mainland_cidr_file()
                .lines()
                .filter_map(clean_cidr_line)
            {
                if seen.insert(cidr.to_string()) {
                    rendered.push_str(cidr);
                    rendered.push('\n');
                }
            }
            rendered.push_str("# CN_MAINLAND END\n");
            continue;
        }

        let cidr = normalize_ipv4_whitelist_entry(&entry.value)?;
        if seen.insert(cidr.clone()) {
            rendered.push_str(&cidr);
            rendered.push('\n');
        }
    }

    Ok(rendered)
}
