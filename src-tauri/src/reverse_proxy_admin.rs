use serde_json::Value;
use std::net::Ipv4Addr;

pub(crate) fn normalize_admin_ip(ip: &str) -> Result<String, String> {
    let trimmed = ip.trim();
    let ip = trimmed
        .parse::<Ipv4Addr>()
        .map_err(|_| "手动解封 IP 必须是合法 IPv4 地址".to_string())?;
    Ok(ip.to_string())
}

pub(crate) fn extract_admin_data(payload: Value) -> Result<Value, String> {
    if payload.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(payload.get("data").cloned().unwrap_or(Value::Null));
    }

    Err(payload
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("OpenResty 管理接口返回失败")
        .to_string())
}

pub(crate) fn parse_admin_http_response(response: &[u8]) -> Result<Value, String> {
    let header_end = find_http_header_end(response)
        .ok_or_else(|| "OpenResty 管理接口响应格式无效：缺少 HTTP 头".to_string())?;
    let header = String::from_utf8_lossy(&response[..header_end]);
    let status_line = header
        .lines()
        .next()
        .ok_or_else(|| "OpenResty 管理接口响应格式无效：缺少状态行".to_string())?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| format!("OpenResty 管理接口状态行无效：{status_line}"))?;

    let mut body = response[header_end + 4..].to_vec();
    if header
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        body = decode_chunked_body(&body)?;
    }

    if !(200..=299).contains(&status_code) {
        let detail: String = String::from_utf8_lossy(&body).chars().take(500).collect();
        return Err(format!(
            "OpenResty 管理接口返回 HTTP {status_code}：{detail}"
        ));
    }

    serde_json::from_slice::<Value>(&body)
        .map_err(|error| format!("解析 OpenResty 管理接口 JSON 失败：{error}"))
}

fn find_http_header_end(response: &[u8]) -> Option<usize> {
    response.windows(4).position(|window| window == b"\r\n\r\n")
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut cursor = 0;
    let mut decoded = Vec::new();

    loop {
        let Some(line_end) = find_crlf(body, cursor) else {
            return Err("OpenResty 管理接口分块响应格式无效：缺少块长度".to_string());
        };
        let size_text = String::from_utf8_lossy(&body[cursor..line_end]);
        let size_hex = size_text.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| format!("OpenResty 管理接口分块长度无效：{size_hex}"))?;
        cursor = line_end + 2;

        if size == 0 {
            break;
        }
        if body.len() < cursor + size + 2 {
            return Err("OpenResty 管理接口分块响应被截断".to_string());
        }
        decoded.extend_from_slice(&body[cursor..cursor + size]);
        cursor += size + 2;
    }

    Ok(decoded)
}

fn find_crlf(body: &[u8], start: usize) -> Option<usize> {
    body.get(start..)?
        .windows(2)
        .position(|window| window == b"\r\n")
        .map(|offset| start + offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 管理接口响应支持普通_json_和分块_json() {
        let plain =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"ok\":true,\"data\":[]}";
        let parsed = parse_admin_http_response(plain).unwrap();
        assert_eq!(parsed["ok"], true);

        let part_a = "{\"ok\":true,\"data\":{\"ip\":\"";
        let part_b = "203.0.113.10\"}}";
        let chunked = format!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{}\r\n{:x}\r\n{}\r\n0\r\n\r\n",
            part_a.len(),
            part_a,
            part_b.len(),
            part_b
        );
        let parsed = parse_admin_http_response(chunked.as_bytes()).unwrap();
        assert_eq!(parsed["data"]["ip"], "203.0.113.10");
    }

    #[test]
    fn 手动解封_ip_只接受_ipv4() {
        assert_eq!(normalize_admin_ip("203.0.113.10").unwrap(), "203.0.113.10");
        assert!(normalize_admin_ip("example.com").is_err());
        assert!(normalize_admin_ip("2001:db8::1").is_err());
    }
}
