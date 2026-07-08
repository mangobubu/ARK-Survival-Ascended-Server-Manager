use super::AppRuntime;
use crate::{
    app_logs::{MAX_LOG_LINES, push_log_line},
    models::{LogLine, LogSource, ServerLogKind},
};

impl AppRuntime {
    pub fn add_log(&self, instance: &str, level: &str, message: &str) -> Result<LogLine, String> {
        self.add_log_with_source(LogSource::Application, None, instance, level, message)
    }

    pub fn add_server_log_with_kind(
        &self,
        instance: &str,
        level: &str,
        message: &str,
        server_log_kind: ServerLogKind,
    ) -> Result<LogLine, String> {
        self.add_log_with_source(
            LogSource::Server,
            Some(server_log_kind),
            instance,
            level,
            message,
        )
    }

    fn add_log_with_source(
        &self,
        source: LogSource,
        server_log_kind: Option<ServerLogKind>,
        instance: &str,
        level: &str,
        message: &str,
    ) -> Result<LogLine, String> {
        let line = {
            let mut data = self.lock()?;
            push_log_line(&mut data, source, server_log_kind, instance, level, message)
        };
        self.persist()?;
        Ok(line)
    }

    pub fn query_logs(&self, limit: Option<usize>) -> Result<Vec<LogLine>, String> {
        let data = self.lock()?;
        let limit = limit.unwrap_or(MAX_LOG_LINES);
        let start = data.logs.len().saturating_sub(limit);
        Ok(data.logs[start..].to_vec())
    }

    pub fn clear_logs(&self) -> Result<(), String> {
        {
            let mut data = self.lock()?;
            data.logs.clear();
        }
        self.persist()
    }

    pub fn clear_logs_by_scope(
        &self,
        source: LogSource,
        instance: Option<&str>,
        server_log_kind: Option<ServerLogKind>,
    ) -> Result<(), String> {
        {
            let mut data = self.lock()?;
            data.logs.retain(|line| {
                let matches_instance = instance.is_none_or(|target| line.instance == target);
                let matches_kind = server_log_kind.as_ref().is_none_or(|target| {
                    line.server_log_kind
                        .as_ref()
                        .unwrap_or(&ServerLogKind::Console)
                        == target
                });
                !(line.source == source && matches_instance && matches_kind)
            });
        }
        self.persist()
    }
}
