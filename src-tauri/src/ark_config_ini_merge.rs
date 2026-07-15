use crate::ark_config_values::{
    CUSTOM_INI_BEGIN_SUFFIX, CUSTOM_INI_END_SUFFIX, CUSTOM_INI_MARKER_PREFIX,
};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug)]
struct GeneratedSection {
    name: String,
    lines: Vec<String>,
}

#[derive(Debug)]
struct CustomIniBlock {
    begin: String,
    end: String,
}

/// 将管理器渲染的 INI 键合并到已有文件中。
///
/// section 与 key 按 INI 的常见行为进行大小写不敏感匹配。已有文件中的未知行会原样
/// 保留，而托管 key 的所有旧实例会被当前渲染结果整体替换。
pub(crate) fn merge_ini(
    existing: &str,
    generated: &str,
    additional_managed_keys: &[(&str, &str)],
) -> String {
    let generated_sections = parse_generated_sections(generated);
    let mut managed_keys: HashMap<String, Vec<String>> = HashMap::new();

    for section in &generated_sections {
        let keys = managed_keys
            .entry(normalize_name(&section.name))
            .or_default();
        for line in &section.lines {
            if let Some(key) = assignment_key(line)
                && !keys.contains(&key)
            {
                keys.push(key);
            }
        }
    }
    for (section, key) in additional_managed_keys {
        let keys = managed_keys.entry(normalize_name(section)).or_default();
        let key = normalize_key(key);
        if !keys.contains(&key) {
            keys.push(key);
        }
    }

    let generated_by_section = generated_sections
        .iter()
        .map(|section| (normalize_name(&section.name), section))
        .collect::<HashMap<_, _>>();
    let custom_blocks_by_section = generated_sections
        .iter()
        .map(|section| {
            (
                normalize_name(&section.name),
                generated_custom_ini_blocks(section),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut inserted_sections = Vec::new();
    let mut output = Vec::new();
    let mut current_section = String::new();
    let existing_lines = existing
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .collect::<Vec<_>>();
    let mut index = 0;

    while index < existing_lines.len() {
        let line = existing_lines[index];
        if let Some(section) = section_name(line) {
            current_section = normalize_name(section);
            output.push(line.to_string());
            if let Some(generated_section) = generated_by_section.get(&current_section)
                && !inserted_sections.contains(&current_section)
            {
                output.extend(generated_section.lines.iter().cloned());
                inserted_sections.push(current_section.clone());
            }
            index += 1;
            continue;
        }

        if let Some(block) = custom_blocks_by_section
            .get(&current_section)
            .and_then(|blocks| blocks.iter().find(|block| line.trim() == block.begin))
            && let Some(end_index) = find_custom_ini_block_end(&existing_lines, index, &block.end)
        {
            index = end_index + 1;
            continue;
        }

        let is_managed = assignment_key(line).is_some_and(|key| {
            managed_keys
                .get(&current_section)
                .is_some_and(|keys| keys.contains(&key))
        });
        if !is_managed {
            output.push(line.to_string());
        }
        index += 1;
    }

    for section in &generated_sections {
        let normalized = normalize_name(&section.name);
        if inserted_sections.contains(&normalized) {
            continue;
        }
        if output.last().is_some_and(|line| !line.is_empty()) {
            output.push(String::new());
        }
        output.push(format!("[{}]", section.name));
        output.extend(section.lines.iter().cloned());
    }

    while output.last().is_some_and(String::is_empty) {
        output.pop();
    }
    output.push(String::new());
    output.join("\r\n")
}

pub(crate) fn merge_ini_file(
    path: &Path,
    generated: &str,
    additional_managed_keys: &[(&str, &str)],
) -> Result<(), String> {
    let existing = if path.is_file() {
        let bytes = fs::read(path)
            .map_err(|error| format!("无法读取配置文件 {}：{error}", path.display()))?;
        decode_config_bytes(&bytes)
    } else {
        String::new()
    };
    let merged = merge_ini(&existing, generated, additional_managed_keys);
    fs::write(path, merged).map_err(|error| format!("无法写入配置文件 {}：{error}", path.display()))
}

fn parse_generated_sections(content: &str) -> Vec<GeneratedSection> {
    let mut sections: Vec<GeneratedSection> = Vec::new();
    let mut current_index = None;

    for raw_line in content.lines() {
        let line = raw_line
            .trim_end_matches('\r')
            .trim_start_matches('\u{feff}');
        if let Some(section) = section_name(line) {
            let normalized = normalize_name(section);
            current_index = sections
                .iter()
                .position(|item| normalize_name(&item.name) == normalized);
            if current_index.is_none() {
                sections.push(GeneratedSection {
                    name: section.trim().to_string(),
                    lines: Vec::new(),
                });
                current_index = Some(sections.len() - 1);
            }
            continue;
        }
        if (assignment_key(line).is_some() || custom_ini_marker(line).is_some())
            && let Some(index) = current_index
        {
            sections[index].lines.push(line.to_string());
        }
    }
    sections
}

fn generated_custom_ini_blocks(section: &GeneratedSection) -> Vec<CustomIniBlock> {
    section
        .lines
        .iter()
        .filter_map(|line| {
            let (id, is_begin) = custom_ini_marker(line)?;
            is_begin.then(|| CustomIniBlock {
                begin: format!("{CUSTOM_INI_MARKER_PREFIX}{id}{CUSTOM_INI_BEGIN_SUFFIX}"),
                end: format!("{CUSTOM_INI_MARKER_PREFIX}{id}{CUSTOM_INI_END_SUFFIX}"),
            })
        })
        .collect()
}

fn find_custom_ini_block_end(
    lines: &[&str],
    begin_index: usize,
    end_marker: &str,
) -> Option<usize> {
    for (index, line) in lines.iter().enumerate().skip(begin_index + 1) {
        if section_name(line).is_some() {
            return None;
        }
        if line.trim() == end_marker {
            return Some(index);
        }
    }
    None
}

fn custom_ini_marker(line: &str) -> Option<(&str, bool)> {
    let marker = line.trim().strip_prefix(CUSTOM_INI_MARKER_PREFIX)?;
    if let Some(id) = marker.strip_suffix(CUSTOM_INI_BEGIN_SUFFIX) {
        return (!id.is_empty()).then_some((id, true));
    }
    let id = marker.strip_suffix(CUSTOM_INI_END_SUFFIX)?;
    (!id.is_empty()).then_some((id, false))
}

fn section_name(line: &str) -> Option<&str> {
    let line = line.trim().trim_start_matches('\u{feff}');
    line.strip_prefix('[')?.strip_suffix(']')
}

fn assignment_key(line: &str) -> Option<String> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with([';', '#', '[']) {
        return None;
    }
    let (key, _) = line.split_once('=')?;
    let key = normalize_key(key);
    (!key.is_empty()).then_some(key)
}

fn normalize_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_key(value: &str) -> String {
    value
        .trim()
        .trim_start_matches(['+', '-'])
        .trim()
        .to_ascii_lowercase()
}

fn decode_config_bytes(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let values = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16_lossy(&values);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let values = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16_lossy(&values);
    }
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ark_config_values::append_custom_ini_settings;
    use serde_json::json;

    fn generated_with_custom_setting(value: &str) -> String {
        let mut lines = vec!["[ServerSettings]".to_string(), "Known=True".to_string()];
        append_custom_ini_settings(&mut lines, &json!({ "custom": value }), "custom");
        lines.join("\r\n")
    }

    #[test]
    fn 保留未知内容并整体替换托管重复键() {
        let existing = r#"; 顶部注释
[/Script/ShooterGame.ShooterGameMode]
UnknownRepeated=One
+ConfigOverrideItemMaxQuantity=(Old=One)
# 中间注释
UnknownRepeated=Two
-configoverrideitemmaxquantity=(Old=Two)

[Community.Section]
CustomKey=Keep
"#;
        let generated = r#"[/Script/ShooterGame.ShooterGameMode]
MatingIntervalMultiplier=2
ConfigOverrideItemMaxQuantity=(New=One)
ConfigOverrideItemMaxQuantity=(New=Two)
"#;

        let merged = merge_ini(
            existing,
            generated,
            &[(
                "/Script/ShooterGame.ShooterGameMode",
                "ConfigOverrideItemMaxQuantity",
            )],
        );

        assert!(merged.contains("; 顶部注释"));
        assert!(merged.contains("# 中间注释"));
        assert_eq!(merged.matches("UnknownRepeated=").count(), 2);
        assert!(merged.contains("[Community.Section]\r\nCustomKey=Keep"));
        assert!(!merged.contains("Old="));
        assert_eq!(merged.matches("ConfigOverrideItemMaxQuantity=").count(), 2);
    }

    #[test]
    fn 渲染结果为空时也会删除额外声明的托管键组() {
        let existing = "[/Script/ShooterGame.ShooterGameMode]\r\nConfigOverrideItemMaxQuantity=Old\r\nKeep=Yes\r\n";
        let generated = "[/Script/ShooterGame.ShooterGameMode]\r\nMatingIntervalMultiplier=1\r\n";

        let merged = merge_ini(
            existing,
            generated,
            &[(
                "/Script/ShooterGame.ShooterGameMode",
                "ConfigOverrideItemMaxQuantity",
            )],
        );

        assert!(!merged.contains("ConfigOverrideItemMaxQuantity"));
        assert!(merged.contains("Keep=Yes"));
    }

    #[test]
    fn 清空自定义_ini_会删除上次写入的键并保留块外未知配置() {
        let first = merge_ini(
            "[ServerSettings]\r\nCommunityOutside=Keep\r\n",
            &generated_with_custom_setting("Foo=1"),
            &[],
        );
        assert!(first.contains("Foo=1"));

        let cleared = merge_ini(&first, &generated_with_custom_setting(""), &[]);

        assert!(!cleared.contains("Foo=1"));
        assert!(cleared.contains("CommunityOutside=Keep"));
    }

    #[test]
    fn 自定义_ini_键改名后不会残留旧键() {
        let first = merge_ini("", &generated_with_custom_setting("Foo=1"), &[]);
        let renamed = merge_ini(&first, &generated_with_custom_setting("Bar=2"), &[]);

        assert!(!renamed.contains("Foo=1"));
        assert!(renamed.contains("Bar=2"));
    }
}
