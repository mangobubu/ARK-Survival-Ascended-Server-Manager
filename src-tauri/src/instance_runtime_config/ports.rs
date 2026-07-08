use crate::models::ServerInstance;
use std::net::{TcpListener, UdpSocket};

pub(crate) fn ensure_port_available(
    instances: &[ServerInstance],
    port: u16,
    label: &str,
) -> Result<(), String> {
    if instance_uses_port(instances, port) {
        return Err(format!("{label} {port} 已被其他实例占用"));
    }
    if let Some(reason) = system_port_unavailable_reason(port) {
        return Err(format!("{label} {port} 不可用：{reason}"));
    }
    Ok(())
}

pub(crate) fn instance_uses_port(instances: &[ServerInstance], port: u16) -> bool {
    instances
        .iter()
        .any(|item| item.game_port == port || item.query_port == port || item.rcon_port == port)
}

pub(crate) fn validate_port_kind(port_kind: &str) -> Result<(), String> {
    match port_kind {
        "gamePort" | "queryPort" | "rconPort" => Ok(()),
        _ => Err(format!("未知端口类型：{port_kind}")),
    }
}

fn port_for_kind(instance: &ServerInstance, port_kind: &str) -> Result<u16, String> {
    match port_kind {
        "gamePort" => Ok(instance.game_port),
        "queryPort" => Ok(instance.query_port),
        "rconPort" => Ok(instance.rcon_port),
        _ => Err(format!("未知端口类型：{port_kind}")),
    }
}

pub(crate) fn suggest_next_instance_port(
    instances: &[ServerInstance],
    port_kind: &str,
) -> Result<Option<u16>, String> {
    let Some(last_instance) = instances.last() else {
        return Ok(None);
    };
    let mut candidate = u32::from(port_for_kind(last_instance, port_kind)?) + 10;
    while candidate <= u32::from(u16::MAX) {
        let port = candidate as u16;
        if !instance_uses_port(instances, port) {
            return Ok(Some(port));
        }
        candidate += 10;
    }
    Ok(None)
}

pub(crate) fn system_port_unavailable_reason(port: u16) -> Option<String> {
    let tcp_listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => listener,
        Err(error) => return Some(format!("TCP 绑定失败：{error}")),
    };
    drop(tcp_listener);

    let udp_socket = match UdpSocket::bind(("0.0.0.0", port)) {
        Ok(socket) => socket,
        Err(error) => return Some(format!("UDP 绑定失败：{error}")),
    };
    drop(udp_socket);

    None
}
