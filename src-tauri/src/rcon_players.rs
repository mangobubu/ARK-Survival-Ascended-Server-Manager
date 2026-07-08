pub(crate) fn parse_list_players_count(response: &str) -> u32 {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return 0;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("no players") || lower.contains("no player connected") {
        return 0;
    }

    let indexed_players = response
        .lines()
        .filter(|line| is_indexed_player_line(line))
        .count();
    if indexed_players > 0 {
        return indexed_players as u32;
    }

    response
        .lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("uniquenetid") || lower.contains("steamid") || lower.contains("eosid")
        })
        .count() as u32
}

fn is_indexed_player_line(line: &str) -> bool {
    let Some((index, detail)) = line.trim_start().split_once('.') else {
        return false;
    };
    !detail.trim().is_empty() && index.chars().all(|ch| ch.is_ascii_digit())
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
    }

    #[test]
    fn listplayers_unique_net_id_rows_are_counted() {
        let response = "Mango [UniqueNetId:0002fb4340044418b2898e39b]\nBob [UniqueNetId:0002fb4340044418b2898e40c]";

        assert_eq!(parse_list_players_count(response), 2);
    }
}
