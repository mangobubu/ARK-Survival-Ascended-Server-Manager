use super::AppRuntime;
use crate::rcon_players::RconPlayer;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlayerPresenceChanges {
    pub(crate) joined: Vec<RconPlayer>,
    pub(crate) left: Vec<RconPlayer>,
}

impl AppRuntime {
    pub(crate) fn reconcile_online_players(
        &self,
        instance_id: &str,
        players: Vec<RconPlayer>,
    ) -> Result<PlayerPresenceChanges, String> {
        let mut snapshots = self.lock_online_players()?;
        let Some(previous) = snapshots.insert(instance_id.to_string(), players.clone()) else {
            return Ok(PlayerPresenceChanges::default());
        };

        Ok(PlayerPresenceChanges {
            joined: players
                .iter()
                .filter(|player| !contains_same_player(&previous, player))
                .cloned()
                .collect(),
            left: previous
                .iter()
                .filter(|player| !contains_same_player(&players, player))
                .cloned()
                .collect(),
        })
    }

    pub(crate) fn clear_online_players(
        &self,
        instance_id: &str,
    ) -> Result<Vec<RconPlayer>, String> {
        Ok(self
            .lock_online_players()?
            .remove(instance_id)
            .unwrap_or_default())
    }

    pub(crate) fn record_player_joined_name(
        &self,
        instance_id: &str,
        player_name: &str,
    ) -> Result<bool, String> {
        let player = RconPlayer::name_only(player_name);
        let mut snapshots = self.lock_online_players()?;
        let players = snapshots.entry(instance_id.to_string()).or_default();
        if contains_same_player(players, &player) {
            return Ok(false);
        }
        players.push(player);
        Ok(true)
    }

    pub(crate) fn record_player_left_name(
        &self,
        instance_id: &str,
        player_name: &str,
    ) -> Result<bool, String> {
        let player = RconPlayer::name_only(player_name);
        let mut snapshots = self.lock_online_players()?;
        let Some(players) = snapshots.get_mut(instance_id) else {
            return Ok(true);
        };

        let previous_len = players.len();
        players.retain(|current| !is_same_player(current, &player));
        Ok(players.len() != previous_len)
    }

    fn lock_online_players(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Vec<RconPlayer>>>, String> {
        self.online_players
            .lock()
            .map_err(|_| "在线玩家状态锁已损坏".to_string())
    }
}

fn contains_same_player(players: &[RconPlayer], target: &RconPlayer) -> bool {
    players.iter().any(|player| is_same_player(player, target))
}

fn is_same_player(left: &RconPlayer, right: &RconPlayer) -> bool {
    left.presence_key() == right.presence_key() || left.name_key() == right.name_key()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::ManagerData;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    fn test_runtime() -> AppRuntime {
        AppRuntime {
            data_dir: Arc::new(std::env::temp_dir()),
            data: Arc::new(Mutex::new(ManagerData::default())),
            processes: Arc::new(Mutex::new(HashMap::new())),
            configuration_operation: Arc::new(Mutex::new(false)),
            lifecycle_operations: Arc::new(Mutex::new(HashSet::new())),
            update_cancels: Arc::new(Mutex::new(HashMap::new())),
            online_players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[test]
    fn 首次同步在线玩家只建立基线不写入进出事件() {
        let runtime = test_runtime();
        let changes = runtime
            .reconcile_online_players(
                "island",
                vec![
                    RconPlayer::new("Mango", "steam-a"),
                    RconPlayer::new("Bob", "steam-b"),
                ],
            )
            .expect("同步在线玩家");

        assert!(changes.joined.is_empty());
        assert!(changes.left.is_empty());
    }

    #[test]
    fn 后续同步能识别新增和退出玩家() {
        let runtime = test_runtime();
        runtime
            .reconcile_online_players(
                "island",
                vec![
                    RconPlayer::new("Mango", "steam-a"),
                    RconPlayer::new("Bob", "steam-b"),
                ],
            )
            .expect("建立基线");

        let changes = runtime
            .reconcile_online_players(
                "island",
                vec![
                    RconPlayer::new("Mango", "steam-a"),
                    RconPlayer::new("甜甜圈", "steam-c"),
                ],
            )
            .expect("同步在线玩家");

        assert_eq!(changes.joined, vec![RconPlayer::new("甜甜圈", "steam-c")]);
        assert_eq!(changes.left, vec![RconPlayer::new("Bob", "steam-b")]);
    }

    #[test]
    fn 先从服务端日志记录的玩家不会被后续_rcon_id_补全重复识别为加入() {
        let runtime = test_runtime();
        assert!(
            runtime
                .record_player_joined_name("island", "Mango")
                .expect("记录日志加入")
        );

        let changes = runtime
            .reconcile_online_players("island", vec![RconPlayer::new("Mango", "steam-a")])
            .expect("同步在线玩家");

        assert!(changes.joined.is_empty());
        assert!(changes.left.is_empty());
    }
}
