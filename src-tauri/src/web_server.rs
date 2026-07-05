
use crate::{app_state::AppRuntime, commands};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::HashMap, net::SocketAddr};
use tauri::AppHandle;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
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
    body: Vec<u8>,
}

#[derive(Deserialize)]
struct InvokeRequest {
    command: String,
    #[serde(default)]
    args: Value,
}

pub fn start(app: AppHandle, runtime: AppRuntime, port: u16) {
    tauri::async_runtime::spawn(async move {
        let address = SocketAddr::from(([127, 0, 0, 1], port));
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
                            tauri::async_runtime::spawn(async move {
                                let _ = handle_connection(app, runtime, stream).await;
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
    mut stream: TcpStream,
) -> Result<(), String> {
    let response = match read_request(&mut stream).await {
        Ok(request) => route_request(app, runtime, request).await,
        Err(error) => json_response(
            400,
            json!({ "ok": false, "error": error }).to_string().into_bytes(),
        ),
    };

    write_response(&mut stream, response).await
}

async fn route_request(app: AppHandle, runtime: AppRuntime, request: HttpRequest) -> HttpResponse {
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

    if request.method == "POST" && request_path == "/api/invoke" {
        return handle_invoke(app, runtime, request).await;
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
        Ok(data) => json_response(200, json!({ "ok": true, "data": data }).to_string().into_bytes()),
        Err(error) => json_response(
            500,
            json!({ "ok": false, "error": error }).to_string().into_bytes(),
        ),
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

    let asset = find_asset(&requested)
        .or_else(|| {
            if requested.contains('.') {
                None
            } else {
                find_asset("index.html")
            }
        });

    match asset {
        Some(asset) => {
            let body = if head_only { Vec::new() } else { asset.content.to_vec() };
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
    let request_line = lines
        .next()
        .ok_or_else(|| "缺少 HTTP 请求行".to_string())?;
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
    body: Vec<u8>,
}

impl HttpResponse {
    fn new(status: u16, reason: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            reason,
            headers: Vec::new(),
            body,
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
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

async fn write_response(stream: &mut TcpStream, mut response: HttpResponse) -> Result<(), String> {
    response.header("Content-Length", response.body.len().to_string());
    response.header("Connection", "close");
    response.header("Access-Control-Allow-Origin", "*");
    response.header("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
    response.header("Access-Control-Allow-Headers", "content-type");
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
    stream
        .write_all(&response.body)
        .await
        .map_err(|error| format!("写入 HTTP 响应体失败：{error}"))?;
    Ok(())
}
