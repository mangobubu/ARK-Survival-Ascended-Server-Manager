use crate::{
    models::GlobalSettings,
    web_auth_utils::{
        generate_captcha_answer, generate_session_token, normalize_captcha_answer,
        render_captcha_svg,
    },
};
use std::collections::HashMap;

use super::*;

impl WebAuthState {
    pub(crate) fn is_captcha_required(&self, client_identity: &str) -> Result<bool, String> {
        let mut requirements = self
            .captcha_requirements
            .lock()
            .map_err(|_| "Web 验证码状态锁已损坏".to_string())?;
        let Some(state) = requirements.get(client_identity) else {
            return Ok(false);
        };

        if state.required_until <= Instant::now() {
            requirements.remove(client_identity);
            return Ok(false);
        }

        Ok(true)
    }

    pub(crate) fn require_captcha(&self, client_identity: &str) -> Result<(), String> {
        let mut requirements = self
            .captcha_requirements
            .lock()
            .map_err(|_| "Web 验证码状态锁已损坏".to_string())?;
        requirements.insert(
            client_identity.to_string(),
            CaptchaRequirementState {
                required_until: Instant::now() + CAPTCHA_REQUIRED_DURATION,
            },
        );
        Ok(())
    }

    pub(crate) fn create_captcha_challenge(
        &self,
        client_identity: &str,
        settings: &GlobalSettings,
    ) -> Result<CaptchaChallengePayload, String> {
        let answer = generate_captcha_answer(settings)?;
        let token = generate_session_token()?;
        let expires_at = Instant::now() + CAPTCHA_CHALLENGE_DURATION;
        let image_svg = render_captcha_svg(
            &answer,
            settings.web_captcha_font_size,
            settings.web_captcha_noise_points,
        )?;

        let mut challenges = self
            .captcha_challenges
            .lock()
            .map_err(|_| "Web 验证码题库锁已损坏".to_string())?;
        prune_expired_captcha_challenges(&mut challenges);
        if challenges.len() >= CAPTCHA_MAX_CHALLENGES {
            challenges.clear();
        }
        challenges.insert(
            token.clone(),
            CaptchaChallengeState {
                answer: normalize_captcha_answer(&answer),
                client_identity: client_identity.to_string(),
                expires_at,
            },
        );

        Ok(CaptchaChallengePayload {
            token,
            image_svg,
            expires_in_seconds: CAPTCHA_CHALLENGE_DURATION.as_secs(),
        })
    }

    pub(crate) fn verify_captcha(
        &self,
        client_identity: &str,
        captcha_token: Option<&str>,
        captcha_answer: Option<&str>,
    ) -> Result<(), String> {
        let token = captcha_token
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "请输入验证码".to_string())?;
        let answer = captcha_answer
            .map(normalize_captcha_answer)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "请输入验证码".to_string())?;

        let mut challenges = self
            .captcha_challenges
            .lock()
            .map_err(|_| "Web 验证码题库锁已损坏".to_string())?;
        prune_expired_captcha_challenges(&mut challenges);
        let Some(challenge) = challenges.remove(token) else {
            return Err("验证码已过期，请刷新后重试".to_string());
        };

        if challenge.expires_at <= Instant::now() || challenge.client_identity != client_identity {
            return Err("验证码已过期，请刷新后重试".to_string());
        }
        if challenge.answer != answer {
            return Err("验证码不正确，请重新输入".to_string());
        }
        Ok(())
    }
}

fn prune_expired_captcha_challenges(challenges: &mut HashMap<String, CaptchaChallengeState>) {
    let now = Instant::now();
    challenges.retain(|_, challenge| challenge.expires_at > now);
}
