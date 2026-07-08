use std::{collections::HashMap, net::SocketAddr};

use tokio::{io::AsyncReadExt, net::TcpStream};

const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct HttpRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
    pub(crate) client_addr: SocketAddr,
}

pub(crate) async fn read_request(
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
