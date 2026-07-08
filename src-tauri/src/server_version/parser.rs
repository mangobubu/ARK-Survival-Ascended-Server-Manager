pub(crate) fn parse_asa_server_version(content: &str) -> Option<String> {
    content
        .lines()
        .rev()
        .find_map(parse_asa_server_version_line)
}

pub(crate) fn normalize_server_version_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    scan_semantic_version(value, 0, true).or_else(|| scan_semantic_version(value, 0, false))
}

fn parse_asa_server_version_line(line: &str) -> Option<String> {
    if let Some(version) = scan_semantic_version(line, 0, true) {
        return Some(version);
    }

    let lower = line.to_ascii_lowercase();
    [
        "server version",
        "servers version",
        "buildversion",
        "version",
    ]
    .into_iter()
    .filter_map(|anchor| lower.find(anchor).map(|index| index + anchor.len()))
    .min()
    .and_then(|start| scan_semantic_version(line, start, false))
}

fn scan_semantic_version(source: &str, start: usize, require_v_prefix: bool) -> Option<String> {
    let bytes = source.as_bytes();
    let mut index = start.min(bytes.len());
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'v' | b'V') {
            if is_version_boundary_before(bytes, index)
                && let Some(version) = parse_numeric_version_at(bytes, index + 1, index)
            {
                return Some(version);
            }
        } else if !require_v_prefix
            && byte.is_ascii_digit()
            && is_version_boundary_before(bytes, index)
            && let Some(version) = parse_numeric_version_at(bytes, index, index)
        {
            return Some(version);
        }
        index += 1;
    }
    None
}

fn parse_numeric_version_at(
    bytes: &[u8],
    number_start: usize,
    boundary_start: usize,
) -> Option<String> {
    let major_start = number_start;
    let mut dot_index = major_start;
    while dot_index < bytes.len() && bytes[dot_index].is_ascii_digit() {
        dot_index += 1;
    }
    let major_len = dot_index.checked_sub(major_start)?;
    if major_len == 0 || major_len > 4 || bytes.get(dot_index) != Some(&b'.') {
        return None;
    }

    let minor_start = dot_index + 1;
    let mut end = minor_start;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    let minor_len = end.checked_sub(minor_start)?;
    if minor_len == 0 || minor_len > 4 || !is_version_boundary_after(bytes, end) {
        return None;
    }

    let major = std::str::from_utf8(&bytes[major_start..dot_index]).ok()?;
    let minor = std::str::from_utf8(&bytes[minor_start..end]).ok()?;
    if boundary_start > 0 && bytes[boundary_start - 1].is_ascii_alphanumeric() {
        return None;
    }
    Some(format!("v{major}.{minor}"))
}

fn is_version_boundary_before(bytes: &[u8], index: usize) -> bool {
    index == 0 || (!bytes[index - 1].is_ascii_alphanumeric() && bytes[index - 1] != b'.')
}

fn is_version_boundary_after(bytes: &[u8], index: usize) -> bool {
    index == bytes.len() || (!bytes[index].is_ascii_alphanumeric() && bytes[index] != b'.')
}
