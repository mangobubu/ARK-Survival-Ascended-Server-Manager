use std::{collections::HashMap, fs, path::Path};

#[derive(Default)]
pub(crate) struct IniDocument {
    sections: HashMap<String, HashMap<String, String>>,
    repeated_values: HashMap<String, HashMap<String, Vec<String>>>,
}

impl IniDocument {
    pub(crate) fn get(&self, sections: &[&str], key: &str) -> Option<&str> {
        let key = normalize_ini_name(key);
        sections.iter().find_map(|section| {
            self.sections
                .get(&normalize_ini_name(section))
                .and_then(|values| values.get(&key))
                .map(String::as_str)
        })
    }

    pub(crate) fn get_all(&self, sections: &[&str], key: &str) -> Vec<&str> {
        let key = normalize_ini_name(key);
        sections
            .iter()
            .filter_map(|section| self.repeated_values.get(&normalize_ini_name(section)))
            .filter_map(|values| values.get(&key))
            .flat_map(|values| values.iter().map(String::as_str))
            .collect()
    }
}

pub(crate) fn parse_ini_file(path: &Path) -> Result<IniDocument, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("无法读取配置文件 {}：{error}", path.display()))?;
    Ok(parse_ini(&decode_config_bytes(&bytes)))
}

fn parse_ini(content: &str) -> IniDocument {
    let mut document = IniDocument::default();
    let mut current_section = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = normalize_ini_name(&line[1..line.len() - 1]);
            document
                .sections
                .entry(current_section.clone())
                .or_default();
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = normalize_ini_name(key);
        let value = clean_ini_value(value);
        document
            .sections
            .entry(current_section.clone())
            .or_default()
            .insert(key.clone(), value.clone());
        document
            .repeated_values
            .entry(current_section.clone())
            .or_default()
            .entry(key)
            .or_default()
            .push(value);
    }

    document
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

fn normalize_ini_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn clean_ini_value(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].trim().to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_utf16_le_ini() {
        let text = "\u{feff}[ServerSettings]\r\nSessionName=中文名称\r\n";
        let mut bytes = vec![0xFF, 0xFE];
        for unit in text.encode_utf16().skip(1) {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }

        let document = parse_ini(&decode_config_bytes(&bytes));

        assert_eq!(
            document.get(&["SessionSettings", "ServerSettings"], "SessionName"),
            Some("中文名称")
        );
    }
}
