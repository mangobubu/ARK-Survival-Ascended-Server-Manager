#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RconPlayer {
    pub(crate) name: String,
    pub(crate) identity: String,
}

impl RconPlayer {
    pub(crate) fn new(name: impl Into<String>, identity: impl Into<String>) -> Self {
        Self {
            name: name.into().trim().to_string(),
            identity: identity.into().trim().to_string(),
        }
    }

    pub(crate) fn name_only(name: impl Into<String>) -> Self {
        Self::new(name, "")
    }

    pub(crate) fn presence_key(&self) -> String {
        if self.identity.is_empty() {
            self.name_key()
        } else {
            self.identity.to_ascii_lowercase()
        }
    }

    pub(crate) fn name_key(&self) -> String {
        self.name.to_ascii_lowercase()
    }
}

pub(crate) fn parse_list_players_count(response: &str) -> u32 {
    parse_list_players(response).len() as u32
}

pub(crate) fn parse_list_players(response: &str) -> Vec<RconPlayer> {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("no players") || lower.contains("no player connected") {
        return Vec::new();
    }

    let indexed_players: Vec<_> = response
        .lines()
        .filter_map(parse_indexed_player_line)
        .collect();
    if !indexed_players.is_empty() {
        return indexed_players;
    }

    response
        .lines()
        .filter_map(parse_identity_player_line)
        .collect()
}

fn parse_indexed_player_line(line: &str) -> Option<RconPlayer> {
    let (index, detail) = line.trim_start().split_once('.')?;
    if detail.trim().is_empty() || !index.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    parse_name_and_identity(detail).or_else(|| {
        let name = normalize_player_name(detail);
        (!name.is_empty()).then(|| RconPlayer::name_only(name))
    })
}

fn parse_identity_player_line(line: &str) -> Option<RconPlayer> {
    let lower = line.to_ascii_lowercase();
    if !(lower.contains("uniquenetid") || lower.contains("steamid") || lower.contains("eosid")) {
        return None;
    }

    parse_name_and_identity(line)
}

fn parse_name_and_identity(raw: &str) -> Option<RconPlayer> {
    if let Some((name, identity)) = raw.trim().split_once(',') {
        let name = normalize_player_name(name);
        let identity = normalize_identity(identity);
        if !name.is_empty() {
            return Some(RconPlayer::new(name, identity));
        }
    }

    if let Some(start) = raw.find('[')
        && let Some(end) = raw[start + 1..].find(']')
    {
        let name = normalize_player_name(&raw[..start]);
        let identity = normalize_identity(&raw[start + 1..start + 1 + end]);
        if !name.is_empty() {
            return Some(RconPlayer::new(name, identity));
        }
    }

    None
}

fn normalize_player_name(raw: &str) -> String {
    raw.trim()
        .trim_matches(['\'', '"', '`', '“', '”', '‘', '’', ' ', '\t'])
        .trim()
        .to_string()
}

fn normalize_identity(raw: &str) -> String {
    let value = raw.trim();
    for separator in [':', '='] {
        if let Some((_, identity)) = value.split_once(separator) {
            return identity.trim().to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listplayers_empty_response_counts_zero() {
        assert_eq!(parse_list_players_count(""), 0);
        assert_eq!(parse_list_players_count("No Players Connected"), 0);
    }

    #[test]
    fn listplayers_indexed_rows_are_counted() {
        let response = "0. Mango, 0002fb4340044418b2898e39b\n1. Survivor, 76561198000000000";

        assert_eq!(parse_list_players_count(response), 2);
        assert_eq!(
            parse_list_players(response),
            vec![
                RconPlayer::new("Mango", "0002fb4340044418b2898e39b"),
                RconPlayer::new("Survivor", "76561198000000000"),
            ]
        );
    }

    #[test]
    fn listplayers_unique_net_id_rows_are_counted() {
        let response = "Mango [UniqueNetId:0002fb4340044418b2898e39b]\nBob [UniqueNetId:0002fb4340044418b2898e40c]";

        assert_eq!(parse_list_players_count(response), 2);
        assert_eq!(
            parse_list_players(response),
            vec![
                RconPlayer::new("Mango", "0002fb4340044418b2898e39b"),
                RconPlayer::new("Bob", "0002fb4340044418b2898e40c"),
            ]
        );
    }

    #[test]
    fn listplayers_indexed_rows_without_id_keep_player_name() {
        let response = "0. 甜甜圈\n1. Tribe Runner";

        assert_eq!(
            parse_list_players(response),
            vec![
                RconPlayer::name_only("甜甜圈"),
                RconPlayer::name_only("Tribe Runner"),
            ]
        );
    }
}
