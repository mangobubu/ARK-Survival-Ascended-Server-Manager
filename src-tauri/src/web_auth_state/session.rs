use crate::web_auth_utils::{WEB_SESSION_TTL, generate_session_token};
use std::{collections::HashMap, time::Instant};

use super::*;

impl WebAuthState {
    pub(crate) fn create_session(&self, auth_key: String) -> Result<String, String> {
        let token = generate_session_token()?;
        let now = Instant::now();
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Web 登录会话状态锁已损坏".to_string())?;
        let expired_tokens = prune_expired_sessions(&mut sessions, now);
        if sessions.len() >= WEB_SESSION_MAX_SESSIONS {
            return Err("Web 登录会话过多，请先退出旧会话或稍后重试".to_string());
        }
        sessions.insert(
            token.clone(),
            WebSessionState {
                auth_key,
                created_at: now,
                last_seen_at: now,
            },
        );
        drop(sessions);
        self.remove_risk_confirmations_for_sessions(&expired_tokens)?;
        Ok(token)
    }

    pub(crate) fn remove_session(&self, token: &str) -> Result<(), String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Web 登录会话状态锁已损坏".to_string())?;
        sessions.remove(token);
        drop(sessions);
        self.remove_risk_confirmations_for_session(token)?;
        Ok(())
    }

    pub(crate) fn has_session(&self, token: &str, auth_key: &str) -> Result<bool, String> {
        let now = Instant::now();
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Web 登录会话状态锁已损坏".to_string())?;
        let mut invalid_tokens = prune_expired_sessions(&mut sessions, now);
        let valid = match sessions.get_mut(token) {
            Some(state) if state.auth_key == auth_key => {
                state.last_seen_at = now;
                true
            }
            Some(_) => {
                sessions.remove(token);
                invalid_tokens.push(token.to_string());
                false
            }
            None => false,
        };
        drop(sessions);
        self.remove_risk_confirmations_for_sessions(&invalid_tokens)?;
        Ok(valid)
    }
}

fn prune_expired_sessions(
    sessions: &mut HashMap<String, WebSessionState>,
    now: Instant,
) -> Vec<String> {
    let expired = sessions
        .iter()
        .filter_map(|(token, state)| {
            let ttl_expired = state.created_at + WEB_SESSION_TTL <= now;
            let idle_expired = state.last_seen_at + WEB_SESSION_IDLE_TIMEOUT <= now;
            if ttl_expired || idle_expired {
                Some(token.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for token in &expired {
        sessions.remove(token);
    }
    expired
}
