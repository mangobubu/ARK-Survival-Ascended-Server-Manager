use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WebCommandRisk {
    Read,
    Write,
    High,
}

impl WebCommandRisk {
    pub fn validate_confirmed(self, command: &str, confirmed: bool) -> Result<(), String> {
        if self != WebCommandRisk::High {
            return Ok(());
        }
        if confirmed {
            Ok(())
        } else {
            Err(format!("Web 高风险命令 {command} 缺少服务端确认令牌"))
        }
    }
}

impl Serialize for WebCommandRisk {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::High => "high",
        })
    }
}
