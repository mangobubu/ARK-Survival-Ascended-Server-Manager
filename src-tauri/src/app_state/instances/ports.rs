use crate::{
    app_state::AppRuntime,
    instance_runtime_config::{
        instance_uses_port, suggest_next_instance_port, system_port_unavailable_reason,
        validate_port_kind,
    },
    models::PortCheckResult,
};

impl AppRuntime {
    pub fn check_instance_port(
        &self,
        port: u16,
        port_kind: &str,
    ) -> Result<PortCheckResult, String> {
        validate_port_kind(port_kind)?;
        if port < 1024 {
            return Ok(PortCheckResult {
                port,
                available: false,
                exists: false,
                suggested_port: None,
                reason: Some("端口必须在 1024-65535 范围内".to_string()),
            });
        }

        let data = self.lock()?;
        let exists = instance_uses_port(&data.instances, port);
        let suggested_port = if exists {
            suggest_next_instance_port(&data.instances, port_kind)?
        } else {
            None
        };

        if exists {
            return Ok(PortCheckResult {
                port,
                available: false,
                exists,
                suggested_port,
                reason: Some(format!("端口 {port} 已被其他实例占用")),
            });
        }

        let reason = system_port_unavailable_reason(port);
        Ok(PortCheckResult {
            port,
            available: reason.is_none(),
            exists,
            suggested_port,
            reason,
        })
    }
}
