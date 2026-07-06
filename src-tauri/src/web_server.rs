use crate::{
    app_state::AppRuntime, commands, models::GlobalSettings, reverse_proxy,
    sync_events::SyncEventBus, web_auth,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fmt::Write as _,
    net::{SocketAddr, TcpListener as StdTcpListener},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;
const WEB_RUNTIME_BOOTSTRAP_SCRIPT: &str = r#"<script>window.__ASA_RUNTIME__="web";</script>"#;
const WEB_AUTH_COOKIE_NAME: &str = "asa-web-auth-token";
const LOGIN_MAX_FAILED_ATTEMPTS: u32 = 5;
const LOGIN_LOCK_DURATION: Duration = Duration::from_secs(30);
const CAPTCHA_REQUIRED_DURATION: Duration = Duration::from_secs(60 * 60);
const CAPTCHA_CHALLENGE_DURATION: Duration = Duration::from_secs(5 * 60);
const CAPTCHA_MAX_CHALLENGES: usize = 1024;

struct WebAsset {
    path: &'static str,
    content_type: &'static str,
    content: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/web_assets.rs"));

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    client_addr: SocketAddr,
}

#[derive(Deserialize)]
struct InvokeRequest {
    command: String,
    #[serde(default)]
    args: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginRequest {
    username: String,
    password: String,
    #[serde(default)]
    captcha_token: Option<String>,
    #[serde(default)]
    captcha_answer: Option<String>,
}

#[derive(Clone, Default)]
struct WebAuthState {
    sessions: Arc<Mutex<HashMap<String, String>>>,
    login_failures: Arc<Mutex<HashMap<String, LoginFailureState>>>,
    captcha_requirements: Arc<Mutex<HashMap<String, CaptchaRequirementState>>>,
    captcha_challenges: Arc<Mutex<HashMap<String, CaptchaChallengeState>>>,
}

#[derive(Default)]
pub struct WebServerManager {
    active: Mutex<Option<WebServerHandle>>,
}

struct WebServerHandle {
    port: u16,
    shutdown: oneshot::Sender<()>,
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
struct CaptchaChallengePayload {
    token: String,
    image_svg: String,
    expires_in_seconds: u64,
}

impl WebAuthState {
    fn create_session(&self, auth_key: String) -> Result<String, String> {
        let token = generate_session_token()?;
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Web 登录会话状态锁已损坏".to_string())?;
        sessions.insert(token.clone(), auth_key);
        Ok(token)
    }

    fn remove_session(&self, token: &str) -> Result<(), String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Web 登录会话状态锁已损坏".to_string())?;
        sessions.remove(token);
        Ok(())
    }

    fn has_session(&self, token: &str, auth_key: &str) -> Result<bool, String> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| "Web 登录会话状态锁已损坏".to_string())?;
        Ok(sessions.get(token).is_some_and(|stored| stored == auth_key))
    }

    fn check_login_allowed(&self, username: &str) -> Result<(), String> {
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

    fn record_login_failure(&self, username: &str) -> Result<(), String> {
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

    fn record_login_success(&self, username: &str) -> Result<(), String> {
        let key = login_throttle_key(username);
        let mut failures = self
            .login_failures
            .lock()
            .map_err(|_| "Web 登录失败计数状态锁已损坏".to_string())?;
        failures.remove(&key);
        Ok(())
    }

    fn is_captcha_required(&self, client_identity: &str) -> Result<bool, String> {
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

    fn require_captcha(&self, client_identity: &str) -> Result<(), String> {
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

    fn create_captcha_challenge(
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

    fn verify_captcha(
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

impl WebServerManager {
    pub fn apply_settings(
        &self,
        app: AppHandle,
        runtime: AppRuntime,
        settings: &GlobalSettings,
    ) -> Result<(), String> {
        if !settings.web_management_enabled {
            self.stop_current_best_effort(Some(&runtime));
            return Ok(());
        }

        let desired_port = settings.web_server_port;
        let current_port = self
            .active
            .lock()
            .map_err(|_| "Web 管理服务状态锁已损坏".to_string())?
            .as_ref()
            .map(|active| active.port);

        if current_port == Some(desired_port) {
            return Ok(());
        }

        self.stop_current_best_effort(Some(&runtime));
        let handle = spawn_web_server(app, runtime, desired_port)?;
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Web 管理服务状态锁已损坏".to_string())?;
        *active = Some(handle);
        Ok(())
    }

    pub fn shutdown(&self, runtime: Option<&AppRuntime>) {
        self.stop_current_best_effort(runtime);
    }

    fn stop_current_best_effort(&self, runtime: Option<&AppRuntime>) {
        let current = self.active.lock().ok().and_then(|mut active| active.take());
        if let Some(handle) = current {
            let port = handle.port;
            let _ = handle.shutdown.send(());
            if let Some(runtime) = runtime {
                let _ = runtime.add_log(
                    "Web服务",
                    "info",
                    &format!("Web 管理已关闭，端口 {port} 停止监听"),
                );
            }
        }
    }
}

pub fn apply_settings_from_app(app: &AppHandle, settings: &GlobalSettings) -> Result<(), String> {
    let runtime = app
        .try_state::<AppRuntime>()
        .ok_or_else(|| "应用运行状态尚未初始化，无法应用 Web 管理服务设置".to_string())?;
    let manager = app
        .try_state::<WebServerManager>()
        .ok_or_else(|| "Web 管理服务状态尚未初始化".to_string())?;
    manager.apply_settings(app.clone(), runtime.inner().clone(), settings)
}

pub fn shutdown(app: &AppHandle) {
    let runtime = app.try_state::<AppRuntime>();
    if let Some(manager) = app.try_state::<WebServerManager>() {
        manager.shutdown(runtime.as_deref());
    }
}

fn spawn_web_server(
    app: AppHandle,
    runtime: AppRuntime,
    port: u16,
) -> Result<WebServerHandle, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = StdTcpListener::bind(address)
        .map_err(|error| format!("Web 管理启动失败，端口 {port} 不可用：{error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("Web 管理启动失败，无法切换端口 {port} 为非阻塞模式：{error}"))?;
    let (shutdown, mut shutdown_receiver) = oneshot::channel();

    tauri::async_runtime::spawn(async move {
        let listener = match TcpListener::from_std(listener) {
            Ok(listener) => listener,
            Err(error) => {
                let _ = runtime.add_log(
                    "Web服务",
                    "error",
                    &format!("Web 管理启动失败，无法接管端口 {port}：{error}"),
                );
                return;
            }
        };
        let auth_state = WebAuthState::default();
        let _ = runtime.add_log(
            "Web服务",
            "success",
            &format!("Web 管理已启动：http://127.0.0.1:{port}"),
        );

        loop {
            tokio::select! {
                _ = &mut shutdown_receiver => {
                    break;
                }
                accepted = listener.accept() => {
                    match accepted {
                        Ok((stream, peer_addr)) => {
                            let app = app.clone();
                            let runtime = runtime.clone();
                            let auth_state = auth_state.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = handle_connection(app, runtime, auth_state, stream, peer_addr).await;
                            });
                        }
                        Err(error) => {
                            let _ = runtime.add_log(
                                "Web服务",
                                "warn",
                                &format!("接受 Web 连接失败：{error}"),
                            );
                        }
                    }
                }
            }
        }
    });

    Ok(WebServerHandle { port, shutdown })
}

async fn handle_connection(
    app: AppHandle,
    runtime: AppRuntime,
    auth_state: WebAuthState,
    mut stream: TcpStream,
    client_addr: SocketAddr,
) -> Result<(), String> {
    let response = match read_request(&mut stream, client_addr).await {
        Ok(request) => route_request(app, runtime, auth_state, request).await,
        Err(error) => json_response(
            400,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    };

    write_response(&mut stream, response).await
}

async fn route_request(
    app: AppHandle,
    runtime: AppRuntime,
    auth_state: WebAuthState,
    request: HttpRequest,
) -> HttpResponse {
    if let Err(response) = require_allowed_host(&runtime, &request) {
        return response;
    }

    if request.method == "OPTIONS" {
        return HttpResponse::empty(204, "No Content");
    }

    let request_path = request.path.split('?').next().unwrap_or("/");
    if request.method == "GET" && request_path == "/api/health" {
        return json_response(
            200,
            json!({
                "ok": true,
                "name": "ASA Server Manager Web",
                "assets": WEB_ASSETS.len(),
            })
            .to_string()
            .into_bytes(),
        );
    }

    if request.method == "GET" && request_path == "/api/auth/status" {
        return handle_auth_status(runtime, &auth_state, &request);
    }

    if request.method == "GET" && request_path == "/api/auth/captcha" {
        return handle_captcha(runtime, &auth_state, &request);
    }

    if request.method == "POST" && request_path == "/api/auth/login" {
        return handle_login(runtime, auth_state, request).await;
    }

    if request.method == "POST" && request_path == "/api/auth/logout" {
        if let Some(token) = auth_token_from_request(&request, false) {
            if let Err(error) = auth_state.remove_session(&token) {
                return json_response(
                    500,
                    json!({ "ok": false, "error": error })
                        .to_string()
                        .into_bytes(),
                );
            }
        }
        let mut response = json_response(200, json!({ "ok": true }).to_string().into_bytes());
        response.header("Set-Cookie", expired_auth_cookie());
        return response;
    }

    if request.method == "POST" && request_path == "/api/invoke" {
        if let Err(response) = require_auth(&runtime, &auth_state, &request, false) {
            return response;
        }
        return handle_invoke(app, runtime, request).await;
    }

    if request.method == "GET" && request_path == "/api/events" {
        if let Err(response) = require_auth(&runtime, &auth_state, &request, true) {
            return response;
        }
        return stream_events(app).await;
    }

    if request.method == "GET" || request.method == "HEAD" {
        return serve_asset(request_path, request.method == "HEAD");
    }

    json_response(
        405,
        json!({ "ok": false, "error": "不支持的 HTTP 方法" })
            .to_string()
            .into_bytes(),
    )
}

fn require_allowed_host(runtime: &AppRuntime, request: &HttpRequest) -> Result<(), HttpResponse> {
    let settings = runtime.settings().map_err(|error| {
        json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        )
    })?;

    if reverse_proxy::is_request_host_allowed(
        &settings,
        request.headers.get("host").map(String::as_str),
    ) {
        Ok(())
    } else {
        Err(json_response(
            403,
            json!({
                "ok": false,
                "error": "当前 Web 管理端仅允许通过已配置的域名和端口访问"
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

fn handle_auth_status(
    runtime: AppRuntime,
    auth_state: &WebAuthState,
    request: &HttpRequest,
) -> HttpResponse {
    match runtime.settings() {
        Ok(settings) => {
            let client_identity = client_identity_from_request(request);
            let captcha_required = if auth_configured(&settings) {
                match auth_state.is_captcha_required(&client_identity) {
                    Ok(required) => required,
                    Err(error) => {
                        return json_response(
                            500,
                            json!({ "ok": false, "error": error })
                                .to_string()
                                .into_bytes(),
                        );
                    }
                }
            } else {
                false
            };
            json_response(
                200,
                json!({
                    "ok": true,
                    "data": {
                        "configured": auth_configured(&settings),
                        "captchaRequired": captcha_required
                    }
                })
                .to_string()
                .into_bytes(),
            )
        }
        Err(error) => json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    }
}

fn handle_captcha(
    runtime: AppRuntime,
    auth_state: &WebAuthState,
    request: &HttpRequest,
) -> HttpResponse {
    let settings = match runtime.settings() {
        Ok(settings) => settings,
        Err(error) => {
            return json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    if !auth_configured(&settings) {
        return json_response(
            403,
            json!({
                "ok": false,
                "error": "尚未配置 Web 管理员账号和密码，请先回到桌面端的全局设置中部署。"
            })
            .to_string()
            .into_bytes(),
        );
    }

    let client_identity = client_identity_from_request(request);
    let required = match auth_state.is_captcha_required(&client_identity) {
        Ok(required) => required,
        Err(error) => {
            return json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
    };
    if !required {
        return json_response(
            200,
            json!({
                "ok": true,
                "data": {
                    "required": false,
                    "token": "",
                    "imageSvg": "",
                    "expiresInSeconds": 0
                }
            })
            .to_string()
            .into_bytes(),
        );
    }

    match auth_state.create_captcha_challenge(&client_identity, &settings) {
        Ok(challenge) => json_response(
            200,
            json!({
                "ok": true,
                "data": {
                    "required": true,
                    "token": challenge.token,
                    "imageSvg": challenge.image_svg,
                    "expiresInSeconds": challenge.expires_in_seconds
                }
            })
            .to_string()
            .into_bytes(),
        ),
        Err(error) => json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    }
}

async fn handle_login(
    runtime: AppRuntime,
    auth_state: WebAuthState,
    request: HttpRequest,
) -> HttpResponse {
    let payload = match serde_json::from_slice::<LoginRequest>(&request.body) {
        Ok(payload) => payload,
        Err(error) => {
            return json_response(
                400,
                json!({ "ok": false, "error": format!("登录请求 JSON 无效：{error}") })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    let settings = match runtime.settings() {
        Ok(settings) => settings,
        Err(error) => {
            return json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    if !auth_configured(&settings) {
        return json_response(
            403,
            json!({
                "ok": false,
                "error": "尚未配置 Web 管理员账号和密码，请先回到桌面端的全局设置中部署。"
            })
            .to_string()
            .into_bytes(),
        );
    }

    if let Err(error) = auth_state.check_login_allowed(&payload.username) {
        return json_response(
            429,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        );
    }

    let client_identity = client_identity_from_request(&request);
    match auth_state.is_captcha_required(&client_identity) {
        Ok(true) => {
            if let Err(error) = auth_state.verify_captcha(
                &client_identity,
                payload.captcha_token.as_deref(),
                payload.captcha_answer.as_deref(),
            ) {
                return json_response(
                    400,
                    json!({
                        "ok": false,
                        "error": error,
                        "captchaRequired": true
                    })
                    .to_string()
                    .into_bytes(),
                );
            }
        }
        Ok(false) => {}
        Err(error) => {
            return json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
    }

    let username_matches = payload.username.trim() == settings.web_admin_username.trim();
    let password_matches = username_matches
        && web_auth::verify_web_admin_password(&payload.password, &settings.web_admin_password)
            .unwrap_or(false);
    if !username_matches || !password_matches {
        if let Err(error) = auth_state.record_login_failure(&payload.username) {
            return json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
        if let Err(error) = auth_state.require_captcha(&client_identity) {
            return json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
        return json_response(
            401,
            json!({
                "ok": false,
                "error": "管理员账号或密码不正确，后续一小时内登录需要输入验证码",
                "captchaRequired": true
            })
            .to_string()
            .into_bytes(),
        );
    }

    if let Err(error) = auth_state.record_login_success(&payload.username) {
        return json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        );
    }

    match auth_state.create_session(auth_key(&settings)) {
        Ok(token) => {
            let mut response = json_response(
                200,
                json!({ "ok": true, "data": { "token": token } })
                    .to_string()
                    .into_bytes(),
            );
            response.header("Set-Cookie", auth_cookie(&token));
            response
        }
        Err(error) => json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    }
}

async fn stream_events(app: AppHandle) -> HttpResponse {
    let Some(bus) = app.try_state::<SyncEventBus>() else {
        return json_response(
            503,
            json!({ "ok": false, "error": "同步事件总线尚未初始化" })
                .to_string()
                .into_bytes(),
        );
    };

    let mut receiver = bus.subscribe();
    let (mut writer, body) = StreamingBody::new();

    tauri::async_runtime::spawn(async move {
        if writer
            .write_all(b": ASA Server Manager sync stream\n\n")
            .await
            .is_err()
        {
            return;
        }

        loop {
            let event = match receiver.recv().await {
                Ok(event) => event,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };
            let data = match serde_json::to_string(&event.payload) {
                Ok(data) => data,
                Err(_) => continue,
            };
            let frame = format!(
                "id: {}\nevent: {}\ndata: {}\n\n",
                event.id,
                event.name,
                data.replace('\n', "\\n"),
            );
            if writer.write_all(frame.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    let mut response = HttpResponse::stream(200, "OK", body);
    response.header("Content-Type", "text/event-stream; charset=utf-8");
    response.header("Cache-Control", "no-cache");
    response.header("X-Accel-Buffering", "no");
    response
}

async fn handle_invoke(app: AppHandle, runtime: AppRuntime, request: HttpRequest) -> HttpResponse {
    let payload = match serde_json::from_slice::<InvokeRequest>(&request.body) {
        Ok(payload) => payload,
        Err(error) => {
            return json_response(
                400,
                json!({ "ok": false, "error": format!("Web API 请求 JSON 无效：{error}") })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    match commands::handle_web_invoke(app, runtime, payload.command, payload.args).await {
        Ok(data) => json_response(
            200,
            json!({ "ok": true, "data": data }).to_string().into_bytes(),
        ),
        Err(error) => json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    }
}

fn require_auth(
    runtime: &AppRuntime,
    auth_state: &WebAuthState,
    request: &HttpRequest,
    allow_query_token: bool,
) -> Result<(), HttpResponse> {
    let settings = runtime.settings().map_err(|error| {
        json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        )
    })?;

    if !auth_configured(&settings) {
        return Err(json_response(
            403,
            json!({
                "ok": false,
                "error": "Web 管理员账号和密码尚未配置，请先在桌面端全局设置中部署。"
            })
            .to_string()
            .into_bytes(),
        ));
    }

    let Some(token) = auth_token_from_request(request, allow_query_token) else {
        return Err(json_response(
            401,
            json!({ "ok": false, "error": "Web 操作需要先登录" })
                .to_string()
                .into_bytes(),
        ));
    };

    let valid = auth_state
        .has_session(&token, &auth_key(&settings))
        .map_err(|error| {
            json_response(
                500,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            )
        })?;

    if valid {
        Ok(())
    } else {
        Err(json_response(
            401,
            json!({ "ok": false, "error": "Web 登录已失效，请重新登录" })
                .to_string()
                .into_bytes(),
        ))
    }
}

fn auth_configured(settings: &crate::models::GlobalSettings) -> bool {
    !settings.web_admin_username.trim().is_empty() && !settings.web_admin_password.is_empty()
}

fn auth_key(settings: &crate::models::GlobalSettings) -> String {
    format!(
        "{}\u{0}{}",
        settings.web_admin_username.trim(),
        settings.web_admin_password
    )
}

fn auth_token_from_request(request: &HttpRequest, allow_query_token: bool) -> Option<String> {
    request
        .headers
        .get("authorization")
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| cookie_token_from_request(request, WEB_AUTH_COOKIE_NAME))
        .or_else(|| {
            if allow_query_token {
                query_param(&request.path, "token")
            } else {
                None
            }
        })
}

fn cookie_token_from_request(request: &HttpRequest, name: &str) -> Option<String> {
    let cookie = request.headers.get("cookie")?;
    for item in cookie.split(';') {
        let Some((key, value)) = item.trim().split_once('=') else {
            continue;
        };
        if key == name {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn auth_cookie(token: &str) -> String {
    format!("{WEB_AUTH_COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Strict")
}

fn expired_auth_cookie() -> String {
    format!("{WEB_AUTH_COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict")
}

fn login_throttle_key(username: &str) -> String {
    username.trim().to_ascii_lowercase()
}

fn client_identity_from_request(request: &HttpRequest) -> String {
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

fn generate_captcha_answer(settings: &GlobalSettings) -> Result<String, String> {
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

fn normalize_captcha_answer(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

fn prune_expired_captcha_challenges(challenges: &mut HashMap<String, CaptchaChallengeState>) {
    let now = Instant::now();
    challenges.retain(|_, challenge| challenge.expires_at > now);
}

fn render_captcha_svg(answer: &str, font_size: u32, noise_points: u32) -> Result<String, String> {
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

fn query_param(raw_path: &str, name: &str) -> Option<String> {
    let query = raw_path.split_once('?')?.1;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(key).as_deref() == Some(name) {
            return percent_decode(value);
        }
    }
    None
}

fn generate_session_token() -> Result<String, String> {
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

fn inject_web_runtime_bootstrap(content: &[u8]) -> Vec<u8> {
    let Ok(html) = std::str::from_utf8(content) else {
        let mut output = Vec::with_capacity(WEB_RUNTIME_BOOTSTRAP_SCRIPT.len() + content.len());
        output.extend_from_slice(WEB_RUNTIME_BOOTSTRAP_SCRIPT.as_bytes());
        output.extend_from_slice(content);
        return output;
    };

    if html.contains(WEB_RUNTIME_BOOTSTRAP_SCRIPT) {
        return content.to_vec();
    }

    let insert_at = html
        .find("<head>")
        .map(|index| index + "<head>".len())
        .or_else(|| html.find("<script"));

    if let Some(index) = insert_at {
        let mut output = String::with_capacity(html.len() + WEB_RUNTIME_BOOTSTRAP_SCRIPT.len());
        output.push_str(&html[..index]);
        output.push_str(WEB_RUNTIME_BOOTSTRAP_SCRIPT);
        output.push_str(&html[index..]);
        output.into_bytes()
    } else {
        let mut output = Vec::with_capacity(WEB_RUNTIME_BOOTSTRAP_SCRIPT.len() + content.len());
        output.extend_from_slice(WEB_RUNTIME_BOOTSTRAP_SCRIPT.as_bytes());
        output.extend_from_slice(content);
        output
    }
}

fn serve_asset(raw_path: &str, head_only: bool) -> HttpResponse {
    let Some(path) = normalize_asset_path(raw_path) else {
        return text_response(400, "非法资源路径");
    };

    let requested = if path.is_empty() {
        "index.html".to_string()
    } else {
        path
    };

    let asset = find_asset(&requested).or_else(|| {
        if requested.contains('.') {
            None
        } else {
            find_asset("index.html")
        }
    });

    match asset {
        Some(asset) => {
            let body = if head_only {
                Vec::new()
            } else if asset.path == "index.html" {
                inject_web_runtime_bootstrap(asset.content)
            } else {
                asset.content.to_vec()
            };
            let mut response = HttpResponse::new(200, "OK", body);
            response.header("Content-Type", asset.content_type);
            response.header(
                "Cache-Control",
                if asset.path == "index.html" {
                    "no-cache"
                } else {
                    "public, max-age=31536000, immutable"
                },
            );
            response
        }
        None if WEB_ASSETS.is_empty() => text_response(
            503,
            "Web 静态资源尚未嵌入，请先执行 npm run build 后重新启动应用。",
        ),
        None => text_response(404, "资源不存在"),
    }
}

fn find_asset(path: &str) -> Option<&'static WebAsset> {
    WEB_ASSETS.iter().find(|asset| asset.path == path)
}

fn normalize_asset_path(raw_path: &str) -> Option<String> {
    let trimmed = raw_path.trim_start_matches('/');
    let decoded = percent_decode(trimmed)?;
    if decoded
        .split('/')
        .any(|part| part == ".." || part.contains('\\') || part.contains(':'))
    {
        return None;
    }
    Some(decoded)
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let high = hex_value(bytes[index + 1])?;
            let low = hex_value(bytes[index + 2])?;
            output.push(high * 16 + low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).ok()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

async fn read_request(
    stream: &mut TcpStream,
    client_addr: SocketAddr,
) -> Result<HttpRequest, String> {
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 4096];

    loop {
        let read = stream
            .read(&mut temp)
            .await
            .map_err(|error| format!("读取 HTTP 请求失败：{error}"))?;
        if read == 0 {
            return Err("HTTP 请求为空".to_string());
        }
        buffer.extend_from_slice(&temp[..read]);
        if buffer.len() > MAX_REQUEST_BYTES {
            return Err("HTTP 请求过大".to_string());
        }

        if let Some(header_end) = find_header_end(&buffer) {
            let header_text = std::str::from_utf8(&buffer[..header_end])
                .map_err(|_| "HTTP 请求头不是有效 UTF-8".to_string())?;
            let (method, path, headers) = parse_headers(header_text)?;
            let content_length = headers
                .get("content-length")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            let body_start = header_end + 4;
            let expected_len = body_start + content_length;

            while buffer.len() < expected_len {
                let read = stream
                    .read(&mut temp)
                    .await
                    .map_err(|error| format!("读取 HTTP 请求体失败：{error}"))?;
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&temp[..read]);
                if buffer.len() > MAX_REQUEST_BYTES {
                    return Err("HTTP 请求过大".to_string());
                }
            }

            let body = if buffer.len() >= expected_len {
                buffer[body_start..expected_len].to_vec()
            } else {
                return Err("HTTP 请求体不完整".to_string());
            };

            return Ok(HttpRequest {
                method,
                path,
                headers,
                body,
                client_addr,
            });
        }
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_headers(header_text: &str) -> Result<(String, String, HashMap<String, String>), String> {
    let mut lines = header_text.lines();
    let request_line = lines.next().ok_or_else(|| "缺少 HTTP 请求行".to_string())?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| "缺少 HTTP 方法".to_string())?
        .to_uppercase();
    let path = request_parts
        .next()
        .ok_or_else(|| "缺少 HTTP 路径".to_string())?
        .to_string();

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    Ok((method, path, headers))
}

struct HttpResponse {
    status: u16,
    reason: &'static str,
    headers: Vec<(String, String)>,
    body: HttpBody,
}

enum HttpBody {
    Full(Vec<u8>),
    Stream(mpsc::Receiver<Vec<u8>>),
}

struct StreamingBodyWriter {
    sender: mpsc::Sender<Vec<u8>>,
}

struct StreamingBody {
    receiver: mpsc::Receiver<Vec<u8>>,
}

impl StreamingBody {
    fn new() -> (StreamingBodyWriter, Self) {
        let (sender, receiver) = mpsc::channel(32);
        (StreamingBodyWriter { sender }, Self { receiver })
    }
}

impl StreamingBodyWriter {
    async fn write_all(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.sender.send(bytes.to_vec()).await.map_err(|_| ())
    }
}

impl HttpResponse {
    fn new(status: u16, reason: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            reason,
            headers: Vec::new(),
            body: HttpBody::Full(body),
        }
    }

    fn stream(status: u16, reason: &'static str, body: StreamingBody) -> Self {
        Self {
            status,
            reason,
            headers: Vec::new(),
            body: HttpBody::Stream(body.receiver),
        }
    }

    fn empty(status: u16, reason: &'static str) -> Self {
        Self::new(status, reason, Vec::new())
    }

    fn header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.push((name.into(), value.into()));
    }
}

fn json_response(status: u16, body: Vec<u8>) -> HttpResponse {
    let reason = reason_phrase(status);
    let mut response = HttpResponse::new(status, reason, body);
    response.header("Content-Type", "application/json; charset=utf-8");
    response
}

fn text_response(status: u16, message: &str) -> HttpResponse {
    let reason = reason_phrase(status);
    let mut response = HttpResponse::new(status, reason, message.as_bytes().to_vec());
    response.header("Content-Type", "text/plain; charset=utf-8");
    response
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

async fn write_response(stream: &mut TcpStream, mut response: HttpResponse) -> Result<(), String> {
    let body = std::mem::replace(&mut response.body, HttpBody::Full(Vec::new()));
    match &body {
        HttpBody::Full(bytes) => {
            response.header("Content-Length", bytes.len().to_string());
            response.header("Connection", "close");
        }
        HttpBody::Stream(_) => {
            response.header("Transfer-Encoding", "chunked");
            response.header("Connection", "keep-alive");
        }
    }
    response.header("X-Content-Type-Options", "nosniff");

    let mut head = format!("HTTP/1.1 {} {}\r\n", response.status, response.reason);
    for (name, value) in response.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");

    stream
        .write_all(head.as_bytes())
        .await
        .map_err(|error| format!("写入 HTTP 响应头失败：{error}"))?;

    match body {
        HttpBody::Full(bytes) => {
            stream
                .write_all(&bytes)
                .await
                .map_err(|error| format!("写入 HTTP 响应体失败：{error}"))?;
        }
        HttpBody::Stream(mut receiver) => {
            while let Some(chunk) = receiver.recv().await {
                if chunk.is_empty() {
                    continue;
                }
                let header = format!("{:X}\r\n", chunk.len());
                stream
                    .write_all(header.as_bytes())
                    .await
                    .map_err(|error| format!("写入 HTTP 流式响应头失败：{error}"))?;
                stream
                    .write_all(&chunk)
                    .await
                    .map_err(|error| format!("写入 HTTP 流式响应体失败：{error}"))?;
                stream
                    .write_all(b"\r\n")
                    .await
                    .map_err(|error| format!("写入 HTTP 流式响应分隔符失败：{error}"))?;
            }
            let _ = stream.write_all(b"0\r\n\r\n").await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(path: &str, authorization: Option<&str>) -> HttpRequest {
        let mut headers = HashMap::new();
        if let Some(authorization) = authorization {
            headers.insert("authorization".to_string(), authorization.to_string());
        }
        HttpRequest {
            method: "GET".to_string(),
            path: path.to_string(),
            headers,
            body: Vec::new(),
            client_addr: SocketAddr::from(([127, 0, 0, 1], 18080)),
        }
    }

    fn cookie_request(path: &str, cookie: &str) -> HttpRequest {
        let mut request = request(path, None);
        request
            .headers
            .insert("cookie".to_string(), cookie.to_string());
        request
    }

    #[test]
    fn query_token_only_allowed_for_event_stream() {
        let request = request("/api/invoke?token=query-token", None);
        assert_eq!(auth_token_from_request(&request, false), None);
        assert_eq!(
            auth_token_from_request(&request, true).as_deref(),
            Some("query-token")
        );
    }

    #[test]
    fn authorization_header_takes_precedence_over_query_token() {
        let request = request("/api/events?token=query-token", Some("Bearer header-token"));
        assert_eq!(
            auth_token_from_request(&request, true).as_deref(),
            Some("header-token")
        );
    }

    #[test]
    fn http_only_cookie_token_is_accepted_before_query_token() {
        let request = cookie_request(
            "/api/events?token=query-token",
            "other=value; asa-web-auth-token=cookie-token",
        );
        assert_eq!(
            auth_token_from_request(&request, true).as_deref(),
            Some("cookie-token")
        );
    }

    #[test]
    fn login_failures_lock_account_temporarily_and_success_resets_counter() {
        let auth_state = WebAuthState::default();
        for _ in 0..LOGIN_MAX_FAILED_ATTEMPTS {
            auth_state
                .record_login_failure("Admin")
                .expect("记录登录失败");
        }

        assert!(auth_state.check_login_allowed("admin").is_err());

        auth_state
            .record_login_success("ADMIN")
            .expect("登录成功后清理失败计数");
        assert!(auth_state.check_login_allowed("admin").is_ok());
    }

    #[test]
    fn first_failed_login_requires_captcha_for_client_identity() {
        let auth_state = WebAuthState::default();
        let client_identity = "203.0.113.10";

        assert!(
            !auth_state
                .is_captcha_required(client_identity)
                .expect("读取初始验证码状态")
        );
        auth_state
            .require_captcha(client_identity)
            .expect("标记需要验证码");

        assert!(
            auth_state
                .is_captcha_required(client_identity)
                .expect("读取验证码状态")
        );
    }

    #[test]
    fn captcha_challenge_can_only_be_used_once() {
        let auth_state = WebAuthState::default();
        let settings = GlobalSettings::default();
        let client_identity = "127.0.0.1";

        let challenge = auth_state
            .create_captcha_challenge(client_identity, &settings)
            .expect("创建验证码");
        let answer = {
            let challenges = auth_state
                .captcha_challenges
                .lock()
                .expect("读取验证码题库");
            challenges
                .get(&challenge.token)
                .expect("题库中存在验证码")
                .answer
                .clone()
        };

        auth_state
            .verify_captcha(client_identity, Some(&challenge.token), Some(&answer))
            .expect("第一次验证码校验成功");
        assert!(
            auth_state
                .verify_captcha(client_identity, Some(&challenge.token), Some(&answer))
                .is_err()
        );
    }
}
