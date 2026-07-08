use crate::models::ModItem;

pub(crate) fn active_mod_ids(mods: &[ModItem]) -> Vec<String> {
    mods.iter()
        .filter(|item| item.enabled)
        .map(|item| item.id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect()
}
