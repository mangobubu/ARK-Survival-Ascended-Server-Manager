use crate::models::{GlobalSettings, ServerInstance};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FirewallRuleSummary {
    pub(crate) label: String,
    pub(crate) protocol: &'static str,
    pub(crate) port: u16,
}

#[derive(Clone, Debug)]
#[cfg_attr(test, allow(dead_code))]
struct FirewallRuleSpec {
    name: String,
    summary: FirewallRuleSummary,
}

pub(crate) fn ensure_instance_firewall_rules(
    instance: &ServerInstance,
) -> Result<Vec<FirewallRuleSummary>, String> {
    let specs = vec![
        FirewallRuleSpec::new(
            format!("ASA Server Manager - Instance {} - Game UDP", instance.id),
            format!("实例「{}」游戏端口", instance.name),
            "UDP",
            instance.game_port,
        ),
        FirewallRuleSpec::new(
            format!("ASA Server Manager - Instance {} - Query UDP", instance.id),
            format!("实例「{}」查询端口", instance.name),
            "UDP",
            instance.query_port,
        ),
        FirewallRuleSpec::new(
            format!("ASA Server Manager - Instance {} - RCON TCP", instance.id),
            format!("实例「{}」RCON 端口", instance.name),
            "TCP",
            instance.rcon_port,
        ),
    ];
    ensure_rule_specs(&specs)?;
    Ok(summarize_rules(&specs))
}

pub(crate) fn ensure_web_firewall_rules(
    settings: &GlobalSettings,
) -> Result<Vec<FirewallRuleSummary>, String> {
    let mut specs = Vec::new();
    if settings.web_management_enabled {
        specs.push(FirewallRuleSpec::new(
            "ASA Server Manager - Web Management TCP".to_string(),
            "Web 管理服务".to_string(),
            "TCP",
            settings.web_server_port,
        ));
    }
    if settings.web_management_enabled && settings.web_reverse_proxy_enabled {
        specs.push(FirewallRuleSpec::new(
            "ASA Server Manager - Web Reverse Proxy TCP".to_string(),
            "Web 域名反向代理".to_string(),
            "TCP",
            settings.web_reverse_proxy_port,
        ));
    }
    ensure_rule_specs(&specs)?;
    Ok(summarize_rules(&specs))
}

pub(crate) fn format_rule_summaries(rules: &[FirewallRuleSummary]) -> String {
    rules
        .iter()
        .map(|rule| format!("{} {} {}", rule.label, rule.protocol, rule.port))
        .collect::<Vec<_>>()
        .join("、")
}

#[cfg_attr(test, allow(dead_code))]
impl FirewallRuleSpec {
    fn new(name: String, label: String, protocol: &'static str, port: u16) -> FirewallRuleSpec {
        FirewallRuleSpec {
            name,
            summary: FirewallRuleSummary {
                label,
                protocol,
                port,
            },
        }
    }

    fn add_args(&self) -> Vec<String> {
        vec![
            "advfirewall".to_string(),
            "firewall".to_string(),
            "add".to_string(),
            "rule".to_string(),
            format!("name={}", self.name),
            "dir=in".to_string(),
            "action=allow".to_string(),
            format!("protocol={}", self.summary.protocol),
            format!("localport={}", self.summary.port),
            "profile=any".to_string(),
            "enable=yes".to_string(),
        ]
    }

    fn set_args(&self) -> Vec<String> {
        vec![
            "advfirewall".to_string(),
            "firewall".to_string(),
            "set".to_string(),
            "rule".to_string(),
            format!("name={}", self.name),
            "new".to_string(),
            "dir=in".to_string(),
            "action=allow".to_string(),
            format!("protocol={}", self.summary.protocol),
            format!("localport={}", self.summary.port),
            "profile=any".to_string(),
            "enable=yes".to_string(),
        ]
    }
}

fn summarize_rules(specs: &[FirewallRuleSpec]) -> Vec<FirewallRuleSummary> {
    specs.iter().map(|spec| spec.summary.clone()).collect()
}

#[cfg(any(not(windows), test))]
fn ensure_rule_specs(_specs: &[FirewallRuleSpec]) -> Result<(), String> {
    Ok(())
}

#[cfg(all(windows, not(test)))]
fn ensure_rule_specs(specs: &[FirewallRuleSpec]) -> Result<(), String> {
    use std::{
        fs,
        process::{Command, Output},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::reverse_proxy_runtime::{command_output_detail, hide_console_window};

    fn run_netsh(args: &[String]) -> Result<Output, String> {
        let mut command = Command::new("netsh.exe");
        command.args(args);
        hide_console_window(&mut command);
        command
            .output()
            .map_err(|error| format!("无法执行 netsh 配置 Windows 防火墙：{error}"))
    }

    fn rule_exists(name: &str) -> Result<bool, String> {
        let args = vec![
            "advfirewall".to_string(),
            "firewall".to_string(),
            "show".to_string(),
            "rule".to_string(),
            format!("name={name}"),
        ];
        let output = run_netsh(&args)?;
        Ok(output.status.success())
    }

    fn powershell_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }

    fn write_elevated_script(commands: &[Vec<String>]) -> Result<std::path::PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("系统时间异常，无法生成防火墙提权脚本名：{error}"))?
            .as_millis();
        let script_path = std::env::temp_dir().join(format!(
            "asa-server-manager-firewall-{}-{millis}.ps1",
            std::process::id()
        ));
        let mut script = String::from("$ErrorActionPreference = 'Stop'\n");
        for args in commands {
            let escaped_args = args
                .iter()
                .map(|arg| powershell_quote(arg))
                .collect::<Vec<_>>()
                .join(" ");
            script.push_str(&format!(
                "& netsh.exe {escaped_args}\nif ($LASTEXITCODE -ne 0) {{ exit $LASTEXITCODE }}\n"
            ));
        }
        script.push_str("exit 0\n");
        fs::write(&script_path, script).map_err(|error| {
            format!(
                "无法写入 Windows 防火墙提权脚本 {}：{error}",
                script_path.display()
            )
        })?;
        Ok(script_path)
    }

    fn run_elevated(commands: &[Vec<String>]) -> Result<(), String> {
        let script_path = write_elevated_script(commands)?;
        let script_text = script_path.to_string_lossy().to_string();
        let launcher = format!(
            "$ErrorActionPreference = 'Stop'; try {{ $process = Start-Process -FilePath 'powershell.exe' -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',{}) -Verb RunAs -Wait -PassThru; exit $process.ExitCode }} catch {{ Write-Error $_; exit 1223 }}",
            powershell_quote(&script_text)
        );
        let mut command = Command::new("powershell.exe");
        command
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(launcher);
        hide_console_window(&mut command);
        let output = command
            .output()
            .map_err(|error| format!("无法启动 Windows 防火墙提权申请：{error}"));
        let _ = fs::remove_file(&script_path);
        let output = output?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Windows 防火墙提权执行失败：{}",
                command_output_detail(&output)
            ))
        }
    }

    let mut elevated_commands = Vec::new();
    let mut failures = Vec::new();

    for spec in specs {
        let args = if rule_exists(&spec.name)? {
            spec.set_args()
        } else {
            spec.add_args()
        };
        let output = run_netsh(&args)?;
        if output.status.success() {
            continue;
        }
        failures.push(format!(
            "{} {} {}：{}",
            spec.summary.label,
            spec.summary.protocol,
            spec.summary.port,
            command_output_detail(&output)
        ));
        elevated_commands.push(args);
    }

    if elevated_commands.is_empty() {
        return Ok(());
    }

    run_elevated(&elevated_commands).map_err(|error| {
        format!(
            "自动配置 Windows 防火墙规则失败，已尝试申请管理员权限。需配置的规则：{}。{error}",
            failures.join("；")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ServerInstance, ServerStatus};

    fn test_instance() -> ServerInstance {
        ServerInstance {
            id: "asa-test".to_string(),
            name: "测试实例".to_string(),
            map: "TheIsland_WP".to_string(),
            map_code: "TheIsland_WP".to_string(),
            mode: "PvE".to_string(),
            status: ServerStatus::Stopped,
            game_port: 7777,
            query_port: 27015,
            players: 0,
            max_players: 70,
            install_path: "D:\\ASA\\Test".to_string(),
            rcon_port: 27020,
            cluster_id: String::new(),
            description: String::new(),
            pid: None,
            last_started_at: None,
            last_stopped_at: None,
            server_version: String::new(),
            version_state: "未安装".to_string(),
            last_error: None,
            skip_auto_update_on_start_once: false,
        }
    }

    #[test]
    fn 实例防火墙规则摘要包含_game_query_rcon_端口() {
        let summaries = ensure_instance_firewall_rules(&test_instance()).expect("生成规则摘要");

        assert_eq!(summaries.len(), 3);
        assert_eq!(summaries[0].protocol, "UDP");
        assert_eq!(summaries[0].port, 7777);
        assert_eq!(summaries[1].port, 27015);
        assert_eq!(summaries[2].protocol, "TCP");
        assert_eq!(summaries[2].port, 27020);
    }

    #[test]
    fn web_防火墙规则只在启用对应服务时生成() {
        let mut settings = GlobalSettings {
            web_management_enabled: true,
            web_server_port: 18080,
            web_reverse_proxy_enabled: true,
            web_reverse_proxy_port: 18081,
            ..GlobalSettings::default()
        };

        let summaries = ensure_web_firewall_rules(&settings).expect("生成 Web 规则摘要");
        assert_eq!(summaries.len(), 2);

        settings.web_reverse_proxy_enabled = false;
        let summaries = ensure_web_firewall_rules(&settings).expect("生成 Web 规则摘要");
        assert_eq!(summaries.len(), 1);

        settings.web_management_enabled = false;
        let summaries = ensure_web_firewall_rules(&settings).expect("生成 Web 规则摘要");
        assert!(summaries.is_empty());
    }
}
