#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlayerServerEventKind {
    Joined,
    Left,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayerServerEvent {
    pub(crate) player_name: String,
    pub(crate) kind: PlayerServerEventKind,
}

impl PlayerServerEvent {
    pub(crate) fn level(&self) -> &'static str {
        match self.kind {
            PlayerServerEventKind::Joined => "success",
            PlayerServerEventKind::Left => "info",
        }
    }

    pub(crate) fn application_message(&self) -> String {
        match self.kind {
            PlayerServerEventKind::Joined => format!("{} 加入了服务器", self.player_name),
            PlayerServerEventKind::Left => format!("{} 退出了服务器", self.player_name),
        }
    }
}

const JOINED_BEFORE_MARKERS: &[&str] = &[
    " has joined the server",
    " has joined server",
    " joined this ark",
    " joined the server",
    " joined server",
    " joined the game",
    " logged in",
];

const LEFT_BEFORE_MARKERS: &[&str] = &[
    " has left the server",
    " has left server",
    " left this ark",
    " left the server",
    " left server",
    " left the game",
    " logged out",
];

const JOINED_AFTER_MARKERS: &[&str] = &["join succeeded:"];

pub(crate) fn parse_player_server_event(line: &str) -> Option<PlayerServerEvent> {
    let normalized = line.to_ascii_lowercase();

    parse_after_marker(
        line,
        &normalized,
        JOINED_AFTER_MARKERS,
        PlayerServerEventKind::Joined,
    )
    .or_else(|| {
        parse_before_marker(
            line,
            &normalized,
            JOINED_BEFORE_MARKERS,
            PlayerServerEventKind::Joined,
        )
    })
    .or_else(|| {
        parse_before_marker(
            line,
            &normalized,
            LEFT_BEFORE_MARKERS,
            PlayerServerEventKind::Left,
        )
    })
}

fn parse_before_marker(
    line: &str,
    normalized: &str,
    markers: &[&str],
    kind: PlayerServerEventKind,
) -> Option<PlayerServerEvent> {
    markers.iter().find_map(|marker| {
        let index = normalized.find(marker)?;
        let player_name = normalize_player_name(&line[..index], true)?;
        Some(PlayerServerEvent {
            player_name,
            kind: kind.clone(),
        })
    })
}

fn parse_after_marker(
    line: &str,
    normalized: &str,
    markers: &[&str],
    kind: PlayerServerEventKind,
) -> Option<PlayerServerEvent> {
    markers.iter().find_map(|marker| {
        let index = normalized.find(marker)?;
        let start = index + marker.len();
        let player_name = normalize_player_name(&line[start..], false)?;
        Some(PlayerServerEvent {
            player_name,
            kind: kind.clone(),
        })
    })
}

fn normalize_player_name(raw: &str, strip_log_prefix: bool) -> Option<String> {
    let mut candidate = raw.trim();
    if strip_log_prefix {
        candidate = strip_server_log_prefix(candidate);
    }
    candidate = strip_player_word(candidate);
    candidate = strip_wrapping_marks(candidate);
    candidate = strip_player_details(candidate);
    candidate = strip_wrapping_marks(candidate);

    if is_valid_player_name(candidate) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn strip_server_log_prefix(raw: &str) -> &str {
    let mut candidate = raw.trim();
    if let Some(index) = candidate.rfind(']') {
        candidate = candidate[index + 1..].trim();
    }
    if let Some(index) = candidate.rfind(':') {
        candidate = candidate[index + 1..].trim();
    }
    candidate
}

fn strip_player_word(raw: &str) -> &str {
    raw.strip_prefix("Player ")
        .or_else(|| raw.strip_prefix("player "))
        .unwrap_or(raw)
        .trim()
}

fn strip_wrapping_marks(raw: &str) -> &str {
    raw.trim()
        .trim_matches(|ch| matches!(ch, '\'' | '"' | '`' | '“' | '”' | '‘' | '’' | ' ' | '\t'))
        .trim_end_matches(|ch| matches!(ch, '.' | '!' | ',' | ';'))
        .trim()
}

fn strip_player_details(raw: &str) -> &str {
    let mut end = raw.len();
    for marker in [
        " [",
        " (",
        " - ",
        "\t",
        " UniqueNetId",
        " SteamID",
        " EOSID",
    ] {
        if let Some(index) = raw.find(marker) {
            end = end.min(index);
        }
    }
    raw[..end].trim()
}

fn is_valid_player_name(candidate: &str) -> bool {
    if candidate.is_empty() || candidate.len() > 80 {
        return false;
    }

    let normalized = candidate.to_ascii_lowercase();
    ![
        "connection",
        "netdriver",
        "socket",
        "unet",
        "lognet",
        "logshooter",
    ]
    .iter()
    .any(|blocked| normalized.contains(blocked))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_joined_this_ark_game_log() {
        let parsed = parse_player_server_event(
            "[2026.07.09-03.31.07:000][123]LogShooterGame: Mango joined this ARK!",
        )
        .expect("应识别加入日志");

        assert_eq!(parsed.player_name, "Mango");
        assert_eq!(parsed.kind, PlayerServerEventKind::Joined);
        assert_eq!(parsed.application_message(), "Mango 加入了服务器");
    }

    #[test]
    fn parses_quoted_player_joined_server_log() {
        let parsed = parse_player_server_event("LogShooterGame: Player '甜甜圈' joined the server")
            .expect("应识别带引号的加入日志");

        assert_eq!(parsed.player_name, "甜甜圈");
        assert_eq!(parsed.kind, PlayerServerEventKind::Joined);
    }

    #[test]
    fn parses_join_succeeded_log() {
        let parsed = parse_player_server_event("LogNet: Join succeeded: Tribe Runner")
            .expect("应识别 Join succeeded 加入日志");

        assert_eq!(parsed.player_name, "Tribe Runner");
        assert_eq!(parsed.kind, PlayerServerEventKind::Joined);
    }

    #[test]
    fn parses_left_this_ark_game_log() {
        let parsed = parse_player_server_event("Mango left this ARK!").expect("应识别退出日志");

        assert_eq!(parsed.player_name, "Mango");
        assert_eq!(parsed.kind, PlayerServerEventKind::Left);
        assert_eq!(parsed.application_message(), "Mango 退出了服务器");
    }

    #[test]
    fn parses_logged_out_log() {
        let parsed = parse_player_server_event("LogShooterGame: Player 'Mango' logged out")
            .expect("应识别登出日志");

        assert_eq!(parsed.player_name, "Mango");
        assert_eq!(parsed.kind, PlayerServerEventKind::Left);
    }

    #[test]
    fn ignores_unrelated_server_logs() {
        assert!(parse_player_server_event("No Players Connected").is_none());
        assert!(
            parse_player_server_event(
                "Current ARK Official Server Network Servers Version: v89.24"
            )
            .is_none()
        );
    }
}
