use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use crate::{
    models::{WebSecurityBanRecord, WebSecurityUnbanResult},
    reverse_proxy_admin::{extract_admin_data, parse_admin_http_response},
};
use serde_json::Value;

use super::super::ReverseProxyConfig;

impl ReverseProxyConfig {
    pub(in crate::reverse_proxy) fn list_security_bans(
        &self,
    ) -> Result<Vec<WebSecurityBanRecord>, String> {
        let payload = self.admin_json_request("GET", "/__asa_security/bans")?;
        let data = extract_admin_data(payload)?;
        serde_json::from_value::<Vec<WebSecurityBanRecord>>(data)
            .map_err(|error| format!("解析 OpenResty 封禁记录失败：{error}"))
    }

    pub(in crate::reverse_proxy) fn unban_security_ip(
        &self,
        ip: &str,
    ) -> Result<WebSecurityUnbanResult, String> {
        let payload = self.admin_json_request("POST", &format!("/__asa_security/unban?ip={ip}"))?;
        let data = extract_admin_data(payload)?;
        serde_json::from_value::<WebSecurityUnbanResult>(data)
            .map_err(|error| format!("解析 OpenResty 解封结果失败：{error}"))
    }

    fn admin_json_request(&self, method: &str, path: &str) -> Result<Value, String> {
        let address = SocketAddr::from(([127, 0, 0, 1], self.public_port));
        let mut stream =
            TcpStream::connect_timeout(&address, Duration::from_secs(2)).map_err(|error| {
                format!(
                    "无法连接 OpenResty 本机管理端口 127.0.0.1:{}：{error}",
                    self.public_port
                )
            })?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| format!("设置 OpenResty 管理请求读取超时失败：{error}"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| format!("设置 OpenResty 管理请求写入超时失败：{error}"))?;

        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {domain}:{port}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
            domain = self.domain,
            port = self.public_port,
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| format!("发送 OpenResty 管理请求失败：{error}"))?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|error| format!("读取 OpenResty 管理响应失败：{error}"))?;
        parse_admin_http_response(&response)
    }
}
