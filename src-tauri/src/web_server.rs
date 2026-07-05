use crate::{app_state::AppRuntime, commands, sync_events::SyncEventBus};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fmt::Write as _,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Manager};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;

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
}

#[derive(Clone, Default)]
struct WebAuthState {
    sessions: Arc<Mutex<HashMap<String, String>>>,
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
}

pub fn start(app: AppHandle, runtime: AppRuntime, port: u16) {
    tauri::async_runtime::spawn(async move {
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let auth_state = WebAuthState::default();
        match TcpListener::bind(address).await {
            Ok(listener) => {
                let _ = runtime.add_log(
                    "Web服务",
                    "success",
                    &format!("Web 版本已启动：http://127.0.0.1:{port}"),
                );
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            let app = app.clone();
                            let runtime = runtime.clone();
                            let auth_state = auth_state.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = handle_connection(app, runtime, auth_state, stream).await;
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
            Err(error) => {
                let _ = runtime.add_log(
                    "Web服务",
                    "error",
                    &format!("Web 版本启动失败，端口 {port} 不可用：{error}"),
                );
            }
        }
    });
}

async fn handle_connection(
    app: AppHandle,
    runtime: AppRuntime,
    auth_state: WebAuthState,
    mut stream: TcpStream,
) -> Result<(), String> {
    let response = match read_request(&mut stream).await {
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
        return handle_auth_status(runtime);
    }

    if request.method == "POST" && request_path == "/api/auth/login" {
        return handle_login(runtime, auth_state, request).await;
    }

    if request.method == "POST" && request_path == "/api/auth/logout" {
        if let Some(token) = auth_token_from_request(&request) {
            if let Err(error) = auth_state.remove_session(&token) {
                return json_response(
                    500,
                    json!({ "ok": false, "error": error })
                        .to_string()
                        .into_bytes(),
                );
            }
        }
        return json_response(200, json!({ "ok": true }).to_string().into_bytes());
    }

    if request.method == "POST" && request_path == "/api/invoke" {
        if let Err(response) = require_auth(&runtime, &auth_state, &request) {
            return response;
        }
        return handle_invoke(app, runtime, request).await;
    }

    if request.method == "GET" && request_path == "/api/events" {
        if let Err(response) = require_auth(&runtime, &auth_state, &request) {
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

fn handle_auth_status(runtime: AppRuntime) -> HttpResponse {
    match runtime.settings() {
        Ok(settings) => json_response(
            200,
            json!({ "ok": true, "data": { "configured": auth_configured(&settings) } })
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

    let username_matches = payload.username.trim() == settings.web_admin_username.trim();
    let password_matches = payload.password == settings.web_admin_password;
    if !username_matches || !password_matches {
        return json_response(
            401,
            json!({ "ok": false, "error": "管理员账号或密码不正确" })
                .to_string()
                .into_bytes(),
        );
    }

    match auth_state.create_session(auth_key(&settings)) {
        Ok(token) => json_response(
            200,
            json!({ "ok": true, "data": { "token": token } })
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

    let Some(token) = auth_token_from_request(request) else {
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

fn auth_token_from_request(request: &HttpRequest) -> Option<String> {
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
        .or_else(|| query_param(&request.path, "token"))
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

async fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
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
    response.header("Access-Control-Allow-Origin", "*");
    response.header("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
    response.header(
        "Access-Control-Allow-Headers",
        "content-type, authorization",
    );
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
