use std::io::Cursor;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{Duration, timeout},
};

const SERVERDATA_AUTH: i32 = 3;
const SERVERDATA_AUTH_RESPONSE: i32 = 2;
const SERVERDATA_EXECCOMMAND: i32 = 2;
const SERVERDATA_RESPONSE_VALUE: i32 = 0;

pub async fn execute(
    host: &str,
    port: u16,
    password: &str,
    command: &str,
) -> Result<String, String> {
    if password.trim().is_empty() {
        return Err("RCON 密码不能为空".to_string());
    }

    let address = format!("{host}:{port}");
    let mut stream = timeout(Duration::from_secs(6), TcpStream::connect(&address))
        .await
        .map_err(|_| format!("连接 RCON 超时：{address}"))?
        .map_err(|error| format!("无法连接 RCON {address}：{error}"))?;

    send_packet(&mut stream, 1, SERVERDATA_AUTH, password).await?;
    let auth = read_packet(&mut stream).await?;
    if auth.request_id == -1 || auth.packet_type != SERVERDATA_AUTH_RESPONSE {
        return Err("RCON 认证失败，请检查管理员密码".to_string());
    }

    send_packet(&mut stream, 2, SERVERDATA_EXECCOMMAND, command).await?;
    let response = read_packet(&mut stream).await?;
    if response.packet_type != SERVERDATA_RESPONSE_VALUE
        && response.packet_type != SERVERDATA_EXECCOMMAND
    {
        return Err("RCON 返回了未知响应类型".to_string());
    }
    Ok(response.body)
}

struct RconPacket {
    request_id: i32,
    packet_type: i32,
    body: String,
}

async fn send_packet(
    stream: &mut TcpStream,
    request_id: i32,
    packet_type: i32,
    body: &str,
) -> Result<(), String> {
    let body_bytes = body.as_bytes();
    let packet_size = 4 + 4 + body_bytes.len() + 2;
    let mut buffer = Vec::with_capacity(packet_size + 4);
    buffer.extend_from_slice(&(packet_size as i32).to_le_bytes());
    buffer.extend_from_slice(&request_id.to_le_bytes());
    buffer.extend_from_slice(&packet_type.to_le_bytes());
    buffer.extend_from_slice(body_bytes);
    buffer.extend_from_slice(&[0, 0]);

    timeout(Duration::from_secs(6), stream.write_all(&buffer))
        .await
        .map_err(|_| "发送 RCON 命令超时".to_string())?
        .map_err(|error| format!("发送 RCON 命令失败：{error}"))
}

async fn read_packet(stream: &mut TcpStream) -> Result<RconPacket, String> {
    let mut size_bytes = [0_u8; 4];
    timeout(Duration::from_secs(6), stream.read_exact(&mut size_bytes))
        .await
        .map_err(|_| "读取 RCON 响应超时".to_string())?
        .map_err(|error| format!("读取 RCON 响应失败：{error}"))?;

    let size = i32::from_le_bytes(size_bytes);
    if !(10..=4096).contains(&size) {
        return Err(format!("RCON 响应长度异常：{size}"));
    }

    let mut payload = vec![0_u8; size as usize];
    timeout(Duration::from_secs(6), stream.read_exact(&mut payload))
        .await
        .map_err(|_| "读取 RCON 响应内容超时".to_string())?
        .map_err(|error| format!("读取 RCON 响应内容失败：{error}"))?;

    let mut cursor = Cursor::new(payload);
    let mut request_id_bytes = [0_u8; 4];
    let mut packet_type_bytes = [0_u8; 4];
    std::io::Read::read_exact(&mut cursor, &mut request_id_bytes)
        .map_err(|error| format!("解析 RCON 请求 ID 失败：{error}"))?;
    std::io::Read::read_exact(&mut cursor, &mut packet_type_bytes)
        .map_err(|error| format!("解析 RCON 响应类型失败：{error}"))?;
    let mut body = Vec::new();
    std::io::Read::read_to_end(&mut cursor, &mut body)
        .map_err(|error| format!("解析 RCON 响应正文失败：{error}"))?;
    while body.last() == Some(&0) {
        body.pop();
    }

    Ok(RconPacket {
        request_id: i32::from_le_bytes(request_id_bytes),
        packet_type: i32::from_le_bytes(packet_type_bytes),
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn 空密码拒绝执行() {
        let error = execute("127.0.0.1", 1, "", "saveworld")
            .await
            .expect_err("应拒绝空密码");
        assert!(error.contains("不能为空"));
    }
}
