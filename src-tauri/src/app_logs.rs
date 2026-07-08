use crate::{
    app_state::{ManagerData, current_time_text, now_millis},
    models::{LogLine, LogSource, ServerLogKind, ServerStatus},
};

pub(crate) const MAX_LOG_LINES: usize = 1_500;

pub(crate) fn recover_stale_update_statuses(data: &mut ManagerData) -> bool {
    let mut recovered_instances = Vec::new();

    for instance in &mut data.instances {
        if instance.status != ServerStatus::Updating {
            continue;
        }

        instance.status = ServerStatus::Stopped;
        instance.pid = None;
        instance.players = 0;
        instance.last_error = None;
        instance.last_stopped_at = Some(current_timestamp_text());
        instance.skip_auto_update_on_start_once = true;
        recovered_instances.push(instance.name.clone());
    }

    let changed = !recovered_instances.is_empty();
    for instance_name in recovered_instances {
        push_log_line(
            data,
            LogSource::Application,
            None,
            &instance_name,
            "warn",
            "检测到上次安装/更新在管理器关闭前中断，已自动恢复为已停止；如需继续安装/校验，请重新执行安装/更新。",
        );
    }

    changed
}

pub(crate) fn push_log_line(
    data: &mut ManagerData,
    source: LogSource,
    server_log_kind: Option<ServerLogKind>,
    instance: &str,
    level: &str,
    message: &str,
) -> LogLine {
    let timestamp_id = now_millis();
    let id = data
        .logs
        .last()
        .map(|line| line.id.saturating_add(1))
        .filter(|next_id| *next_id > timestamp_id)
        .unwrap_or(timestamp_id);
    let line = LogLine {
        id,
        time: current_time_text(),
        source,
        server_log_kind,
        instance: instance.to_string(),
        level: level.to_string(),
        message: message.to_string(),
    };
    data.logs.push(line.clone());
    if data.logs.len() > MAX_LOG_LINES {
        let overflow = data.logs.len() - MAX_LOG_LINES;
        data.logs.drain(0..overflow);
    }
    line
}

fn current_timestamp_text() -> String {
    crate::app_state::current_timestamp_text()
}
