use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[cfg(test)]
use crate::models::GlobalSettings;

mod captcha;
mod login_throttle;
mod risk;
mod session;

pub(crate) use risk::risk_confirmation_expires_in_seconds;
const LOGIN_MAX_FAILED_ATTEMPTS: u32 = 5;
const LOGIN_LOCK_DURATION: Duration = Duration::from_secs(30);
const CAPTCHA_REQUIRED_DURATION: Duration = Duration::from_secs(60 * 60);
const CAPTCHA_CHALLENGE_DURATION: Duration = Duration::from_secs(5 * 60);
const CAPTCHA_MAX_CHALLENGES: usize = 1024;
#[cfg(not(test))]
const WEB_SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);
#[cfg(test)]
const WEB_SESSION_IDLE_TIMEOUT: Duration = Duration::from_millis(25);
const WEB_SESSION_MAX_SESSIONS: usize = 1024;
const RISK_CONFIRMATION_DURATION: Duration = Duration::from_secs(2 * 60);
const RISK_CONFIRMATION_MAX_TOKENS: usize = 1024;

#[derive(Clone, Default)]
pub(crate) struct WebAuthState {
    sessions: Arc<Mutex<HashMap<String, WebSessionState>>>,
    login_failures: Arc<Mutex<HashMap<String, LoginFailureState>>>,
    captcha_requirements: Arc<Mutex<HashMap<String, CaptchaRequirementState>>>,
    captcha_challenges: Arc<Mutex<HashMap<String, CaptchaChallengeState>>>,
    risk_confirmations: Arc<Mutex<HashMap<String, RiskConfirmationState>>>,
}

#[derive(Clone, Debug)]
struct WebSessionState {
    auth_key: String,
    created_at: Instant,
    last_seen_at: Instant,
}

#[derive(Clone, Debug)]
struct LoginFailureState {
    failed_attempts: u32,
    locked_until: Option<Instant>,
}

#[derive(Clone, Debug)]
struct CaptchaRequirementState {
    required_until: Instant,
}

#[derive(Clone, Debug)]
struct CaptchaChallengeState {
    answer: String,
    client_identity: String,
    expires_at: Instant,
}

#[derive(Clone, Debug)]
struct RiskConfirmationState {
    session_token: String,
    auth_key: String,
    command: String,
    expires_at: Instant,
}

#[derive(Clone, Debug)]
pub(crate) struct CaptchaChallengePayload {
    pub(crate) token: String,
    pub(crate) image_svg: String,
    pub(crate) expires_in_seconds: u64,
}

#[cfg(test)]
mod tests;
