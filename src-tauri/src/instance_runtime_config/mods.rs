use crate::models::{AddInstancePayload, ModItem};
use std::collections::HashSet;

pub(crate) fn sanitize_imported_mods(payload: &AddInstancePayload) -> Vec<ModItem> {
    let mut seen = HashSet::new();
    payload
        .imported_mods
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|item| {
            let id = item.id.trim();
            if id.is_empty() || !id.chars().all(|ch| ch.is_ascii_digit()) {
                return None;
            }
            if !seen.insert(id.to_string()) {
                return None;
            }
            Some(ModItem {
                id: id.to_string(),
                name: if item.name.trim().is_empty() {
                    format!("MOD {id}")
                } else {
                    item.name.trim().to_string()
                },
                version: if item.version.trim().is_empty() {
                    "配置导入".to_string()
                } else {
                    item.version.trim().to_string()
                },
                size: if item.size.trim().is_empty() {
                    "未知大小".to_string()
                } else {
                    item.size.trim().to_string()
                },
                enabled: item.enabled,
                update_available: item.update_available.or(Some(false)),
            })
        })
        .collect()
}
