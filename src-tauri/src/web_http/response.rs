use tokio::{io::AsyncWriteExt, net::TcpStream, sync::mpsc};

const WEB_CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'; form-action 'self'";

pub(crate) struct HttpResponse {
    status: u16,
    reason: &'static str,
    headers: Vec<(String, String)>,
    body: HttpBody,
}

enum HttpBody {
    Full(Vec<u8>),
    Stream(mpsc::Receiver<Vec<u8>>),
}

pub(crate) struct StreamingBodyWriter {
    sender: mpsc::Sender<Vec<u8>>,
}

pub(crate) struct StreamingBody {
    receiver: mpsc::Receiver<Vec<u8>>,
}

impl StreamingBody {
    pub(crate) fn new() -> (StreamingBodyWriter, Self) {
        let (sender, receiver) = mpsc::channel(32);
        (StreamingBodyWriter { sender }, Self { receiver })
    }
}

impl StreamingBodyWriter {
    pub(crate) async fn write_all(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.sender.send(bytes.to_vec()).await.map_err(|_| ())
    }
}

impl HttpResponse {
    pub(crate) fn new(status: u16, reason: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            reason,
            headers: Vec::new(),
            body: HttpBody::Full(body),
        }
    }

    pub(crate) fn stream(status: u16, reason: &'static str, body: StreamingBody) -> Self {
        Self {
            status,
            reason,
            headers: Vec::new(),
            body: HttpBody::Stream(body.receiver),
        }
    }

    pub(crate) fn empty(status: u16, reason: &'static str) -> Self {
        Self::new(status, reason, Vec::new())
    }

    pub(crate) fn header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.push((name.into(), value.into()));
    }
}

pub(crate) fn json_response(status: u16, body: Vec<u8>) -> HttpResponse {
    let reason = reason_phrase(status);
    let mut response = HttpResponse::new(status, reason, body);
    response.header("Content-Type", "application/json; charset=utf-8");
    response
}

pub(crate) fn text_response(status: u16, message: &str) -> HttpResponse {
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

pub(crate) async fn write_response(
    stream: &mut TcpStream,
    mut response: HttpResponse,
) -> Result<(), String> {
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
    response.header("X-Frame-Options", "DENY");
    response.header("Referrer-Policy", "no-referrer");
    response.header(
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=(), payment=()",
    );
    response.header("Content-Security-Policy", WEB_CONTENT_SECURITY_POLICY);

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
