use crate::web_auth_utils::login_throttle_key;
use std::time::Instant;

use super::*;

impl WebAuthState {
    pub(crate) fn check_login_allowed(&self, username: &str) -> Result<(), String> {
        let key = login_throttle_key(username);
        let mut failures = self
            .login_failures
            .lock()
            .map_err(|_| "Web 登录失败计数状态锁已损坏".to_string())?;

        let Some(state) = failures.get(&key) else {
            return Ok(());
        };

        let Some(locked_until) = state.locked_until else {
            return Ok(());
        };

        let now = Instant::now();
        if locked_until <= now {
            failures.remove(&key);
            return Ok(());
        }

        let remaining = locked_until.saturating_duration_since(now).as_secs().max(1);
        Err(format!("登录失败次数过多，请 {remaining} 秒后再试"))
    }

    pub(crate) fn record_login_failure(&self, username: &str) -> Result<(), String> {
        let key = login_throttle_key(username);
        let mut failures = self
            .login_failures
            .lock()
            .map_err(|_| "Web 登录失败计数状态锁已损坏".to_string())?;
        let state = failures.entry(key).or_insert(LoginFailureState {
            failed_attempts: 0,
            locked_until: None,
        });
        state.failed_attempts = state.failed_attempts.saturating_add(1);
        if state.failed_attempts >= LOGIN_MAX_FAILED_ATTEMPTS {
            state.locked_until = Some(Instant::now() + LOGIN_LOCK_DURATION);
        }
        Ok(())
    }

    pub(crate) fn record_login_success(&self, username: &str) -> Result<(), String> {
        let key = login_throttle_key(username);
        let mut failures = self
            .login_failures
            .lock()
            .map_err(|_| "Web 登录失败计数状态锁已损坏".to_string())?;
        failures.remove(&key);
        Ok(())
    }
}
