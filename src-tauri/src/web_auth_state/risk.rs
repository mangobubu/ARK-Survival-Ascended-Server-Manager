use crate::web_auth_utils::generate_session_token;
use std::collections::HashMap;

use super::*;

impl WebAuthState {
    pub(crate) fn create_risk_confirmation(
        &self,
        session_token: &str,
        auth_key: &str,
        command: &str,
    ) -> Result<String, String> {
        let confirmation_token = generate_session_token()?;
        let mut confirmations = self
            .risk_confirmations
            .lock()
            .map_err(|_| "Web 高风险确认状态锁已损坏".to_string())?;
        prune_expired_risk_confirmations(&mut confirmations);
        if confirmations.len() >= RISK_CONFIRMATION_MAX_TOKENS {
            return Err("待确认的 Web 高风险操作过多，请稍后重试".to_string());
        }
        confirmations.insert(
            confirmation_token.clone(),
            RiskConfirmationState {
                session_token: session_token.to_string(),
                auth_key: auth_key.to_string(),
                command: command.to_string(),
                expires_at: Instant::now() + RISK_CONFIRMATION_DURATION,
            },
        );
        Ok(confirmation_token)
    }

    pub(crate) fn consume_risk_confirmation(
        &self,
        confirmation_token: &str,
        session_token: &str,
        auth_key: &str,
        command: &str,
    ) -> Result<(), String> {
        let confirmation_token = confirmation_token.trim();
        if confirmation_token.is_empty() {
            return Err("Web 高风险操作缺少服务端确认令牌".to_string());
        }

        let mut confirmations = self
            .risk_confirmations
            .lock()
            .map_err(|_| "Web 高风险确认状态锁已损坏".to_string())?;
        prune_expired_risk_confirmations(&mut confirmations);
        let Some(state) = confirmations.remove(confirmation_token) else {
            return Err("Web 高风险确认令牌不存在或已过期".to_string());
        };
        if state.expires_at <= Instant::now()
            || state.session_token != session_token
            || state.auth_key != auth_key
            || state.command != command
        {
            return Err("Web 高风险确认令牌与当前会话或命令不匹配".to_string());
        }
        Ok(())
    }

    pub(super) fn remove_risk_confirmations_for_session(
        &self,
        session_token: &str,
    ) -> Result<(), String> {
        self.remove_risk_confirmations_for_sessions(&[session_token.to_string()])
    }

    pub(super) fn remove_risk_confirmations_for_sessions(
        &self,
        session_tokens: &[String],
    ) -> Result<(), String> {
        if session_tokens.is_empty() {
            return Ok(());
        }
        let mut confirmations = self
            .risk_confirmations
            .lock()
            .map_err(|_| "Web 高风险确认状态锁已损坏".to_string())?;
        confirmations.retain(|_, state| !session_tokens.contains(&state.session_token));
        Ok(())
    }
}

pub(crate) fn risk_confirmation_expires_in_seconds() -> u64 {
    RISK_CONFIRMATION_DURATION.as_secs()
}

fn prune_expired_risk_confirmations(confirmations: &mut HashMap<String, RiskConfirmationState>) {
    let now = Instant::now();
    confirmations.retain(|_, confirmation| confirmation.expires_at > now);
}
