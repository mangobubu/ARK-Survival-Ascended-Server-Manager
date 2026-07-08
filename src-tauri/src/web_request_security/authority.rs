pub(super) fn authority_from_absolute_url(value: &str) -> Option<String> {
    let (_, rest) = value.trim().split_once("://")?;
    let authority = rest.split('/').next().unwrap_or_default();
    normalized_authority(authority)
}

pub(super) fn normalized_authority(value: &str) -> Option<String> {
    let value = value.trim().trim_end_matches('.').to_ascii_lowercase();
    if value.is_empty()
        || value.contains('/')
        || value.contains('\\')
        || value.contains('@')
        || value.chars().any(char::is_whitespace)
    {
        return None;
    }
    Some(value)
}
