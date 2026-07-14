use super::*;
use std::collections::HashSet;

#[test]
fn 高风险命令必须带服务端确认结果() {
    let risk = web_command_policy("delete_instance").expect("识别命令风险");
    assert!(risk.validate_confirmed("delete_instance", false).is_err());
    assert!(risk.validate_confirmed("delete_instance", true).is_ok());
}

#[test]
fn 影响实例状态和审计状态的命令归类为高风险() {
    for command in [
        "install_steamcmd",
        "save_instance_config",
        "update_instance_mods",
        "install_or_update_instance",
        "start_instance",
        "stop_instance",
        "restart_instance",
        "clear_logs",
        "clear_scoped_logs",
        "unban_web_security_ip",
        "import_instance_config_upload",
        "delete_instance_file_entry",
    ] {
        assert!(
            matches!(web_command_policy(command), Ok(WebCommandRisk::High)),
            "{command} 应归类为高风险 Web 命令"
        );
    }
}

#[test]
fn 命令安全元数据不重复且包含中文标签() {
    let mut commands = HashSet::new();
    for policy in web_command_policies() {
        assert!(
            commands.insert(policy.command),
            "{} 重复注册",
            policy.command
        );
        assert!(
            !policy.label.trim().is_empty(),
            "{} 缺少展示标签",
            policy.command
        );
    }
    assert!(commands.contains("delete_instance"));
    assert!(commands.contains("save_settings"));
}

#[test]
fn 命令安全元数据与_web_invoke_分发分支保持一致() {
    let policy_commands = web_command_policies()
        .iter()
        .map(|policy| policy.command.to_string())
        .collect::<HashSet<_>>();
    let dispatch_commands =
        extract_web_invoke_dispatch_commands(include_str!("../commands_web.rs"));

    let missing_policies = dispatch_commands
        .difference(&policy_commands)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing_policies.is_empty(),
        "以下 Web 分发命令缺少安全元数据：{}",
        missing_policies.join(", ")
    );

    let missing_dispatch = policy_commands
        .difference(&dispatch_commands)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing_dispatch.is_empty(),
        "以下安全元数据命令缺少 Web 分发分支：{}",
        missing_dispatch.join(", ")
    );
}

fn extract_web_invoke_dispatch_commands(source: &str) -> HashSet<String> {
    let function_start = source
        .find("pub(crate) async fn handle_web_invoke")
        .expect("commands_web.rs 应包含 handle_web_invoke");
    let source = &source[function_start..];
    let match_start = source
        .find("match command.as_str()")
        .expect("handle_web_invoke 应按命令名分发");
    let source = &source[match_start..];
    let dispatch_body = match_block_source(source);

    dispatch_body
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line.strip_prefix('"')?;
            let (command, after_command) = rest.split_once('"')?;
            after_command
                .trim_start()
                .starts_with("=>")
                .then(|| command.to_string())
        })
        .collect()
}

fn match_block_source(source: &str) -> &str {
    let block_start = source.find('{').expect("match 分发表应包含代码块");
    let mut depth = 0_u32;
    for (offset, ch) in source[block_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[block_start..block_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("match 分发表代码块未正确闭合");
}
