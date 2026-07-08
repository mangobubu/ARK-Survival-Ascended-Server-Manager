use crate::{models::GlobalSettings, web_http::HttpRequest};
use std::{fmt::Write as _, time::Duration};

const WEB_AUTH_COOKIE_NAME: &str = "asa-web-auth-token";
pub(crate) const WEB_SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);

pub(crate) fn auth_token_from_request(request: &HttpRequest) -> Option<String> {
    cookie_token_from_request(request, WEB_AUTH_COOKIE_NAME)
}

fn cookie_token_from_request(request: &HttpRequest, name: &str) -> Option<String> {
    let cookie = request.headers.get("cookie")?;
    for part in cookie.split(';') {
        let mut pair = part.trim().splitn(2, '=');
        if let (Some(key), Some(value)) = (pair.next(), pair.next())
            && key.trim() == name
        {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

pub(crate) fn auth_cookie(token: &str, secure: bool) -> String {
    format!(
        "{WEB_AUTH_COOKIE_NAME}={token}; Path=/; Max-Age={}; HttpOnly; SameSite=Strict{}",
        WEB_SESSION_TTL.as_secs(),
        if secure { "; Secure" } else { "" }
    )
}

pub(crate) fn expired_auth_cookie() -> String {
    format!("{WEB_AUTH_COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict")
}

pub(crate) fn login_throttle_key(username: &str) -> String {
    username.trim().to_ascii_lowercase()
}

pub(crate) fn client_identity_from_request(request: &HttpRequest) -> String {
    request
        .headers
        .get("x-real-ip")
        .and_then(|value| first_header_ip(value))
        .or_else(|| {
            request
                .headers
                .get("x-forwarded-for")
                .and_then(|value| first_header_ip(value))
        })
        .unwrap_or_else(|| request.client_addr.ip().to_string())
}

fn first_header_ip(value: &str) -> Option<String> {
    value
        .split(',')
        .map(str::trim)
        .find(|item| !item.is_empty() && item.len() <= 64)
        .map(ToOwned::to_owned)
}

pub(crate) fn generate_captcha_answer(settings: &GlobalSettings) -> Result<String, String> {
    let mut chars: Vec<char> = settings
        .web_captcha_charset
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    if chars.len() < 2 {
        chars = crate::models::default_web_captcha_charset()
            .chars()
            .collect();
    }

    let length = settings.web_captcha_length.clamp(1, 12);
    let mut answer = String::new();
    for _ in 0..length {
        let index = random_u32(chars.len() as u32)? as usize;
        answer.push(chars[index]);
    }
    Ok(answer)
}

pub(crate) fn normalize_captcha_answer(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

pub(crate) fn render_captcha_svg(
    answer: &str,
    font_size: u32,
    noise_points: u32,
) -> Result<String, String> {
    let font_size = font_size.clamp(18, 56);
    let answer_len = answer.chars().count().max(1) as u32;
    let width = (answer_len * font_size * 7 / 10 + 52).clamp(120, 420);
    let height = (font_size * 2 + 18).clamp(56, 140);
    let baseline = height / 2 + font_size / 3;
    let text_x = 22;

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-label="Web 登录验证码"><defs><linearGradient id="bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="#051827"/><stop offset="100%" stop-color="#0d3148"/></linearGradient></defs><rect width="100%" height="100%" rx="12" fill="url(#bg)"/>"##
    );

    let noise_points = noise_points.min(120);
    for _ in 0..noise_points {
        let cx = random_u32(width)?;
        let cy = random_u32(height)?;
        let radius = random_u32(3)? + 1;
        let opacity = 22 + random_u32(46)?;
        let hue = 170 + random_u32(80)?;
        let _ = write!(
            &mut svg,
            r#"<circle cx="{cx}" cy="{cy}" r="{radius}" fill="hsl({hue}, 88%, 62%)" opacity="0.{opacity:02}"/>"#
        );
    }

    let line_count = if noise_points == 0 {
        0
    } else {
        (noise_points / 8).clamp(1, 10)
    };
    for _ in 0..line_count {
        let x1 = random_u32(width)?;
        let y1 = random_u32(height)?;
        let x2 = random_u32(width)?;
        let y2 = random_u32(height)?;
        let opacity = 18 + random_u32(28)?;
        let _ = write!(
            &mut svg,
            r##"<path d="M{x1} {y1} L{x2} {y2}" stroke="#66d9ff" stroke-width="1.1" opacity="0.{opacity:02}" stroke-linecap="round"/>"##
        );
    }

    let safe_answer = escape_svg_text(answer);
    let rotation = (random_u32(9)? as i32) - 4;
    let _ = write!(
        &mut svg,
        r##"<text x="{text_x}" y="{baseline}" fill="#dff9ff" font-family="Consolas, 'SFMono-Regular', monospace" font-size="{font_size}" font-weight="800" letter-spacing="7" transform="rotate({rotation} {text_x} {baseline})" style="paint-order: stroke; stroke: rgba(0,0,0,.38); stroke-width: 2px;">{safe_answer}</text>"##
    );
    svg.push_str("</svg>");
    Ok(svg)
}

fn escape_svg_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(ch),
        }
    }
    output
}

pub(crate) fn generate_session_token() -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| format!("生成 Web 登录令牌失败：{error}"))?;

    let mut token = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut token, "{byte:02x}");
    }
    Ok(token)
}

fn random_u32(max_exclusive: u32) -> Result<u32, String> {
    if max_exclusive == 0 {
        return Ok(0);
    }
    let mut bytes = [0_u8; 4];
    getrandom::fill(&mut bytes).map_err(|error| format!("生成 Web 验证码随机数失败：{error}"))?;
    Ok(u32::from_le_bytes(bytes) % max_exclusive)
}
