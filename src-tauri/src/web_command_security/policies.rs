use serde::Serialize;

use super::risk::WebCommandRisk;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebCommandPolicy {
    pub(crate) command: &'static str,
    pub(crate) label: &'static str,
    pub(crate) risk: WebCommandRisk,
}

const WEB_COMMAND_POLICIES: &[WebCommandPolicy] = &[
    policy("app_version", "读取应用版本", WebCommandRisk::Read),
    policy(
        "get_asa_config_metadata",
        "读取 ASA 配置元数据",
        WebCommandRisk::Read,
    ),
    policy("get_settings", "读取全局设置", WebCommandRisk::Read),
    policy(
        "list_web_security_bans",
        "读取 Web 安全封禁列表",
        WebCommandRisk::Read,
    ),
    policy(
        "get_web_acme_certificate_status",
        "读取 Web ACME 证书状态",
        WebCommandRisk::Read,
    ),
    policy("list_instances", "读取实例列表", WebCommandRisk::Read),
    policy("check_instance_port", "检查实例端口", WebCommandRisk::Read),
    policy(
        "read_server_directory_config",
        "读取服务端目录配置",
        WebCommandRisk::Read,
    ),
    policy(
        "list_host_directories",
        "浏览运行主机服务器目录",
        WebCommandRisk::Read,
    ),
    policy("get_instance_config", "读取实例配置", WebCommandRisk::Read),
    policy(
        "get_instance_mods",
        "读取实例 MOD 列表",
        WebCommandRisk::Read,
    ),
    policy("check_mod_updates", "检查 MOD 更新", WebCommandRisk::Read),
    policy(
        "search_curseforge_mods",
        "搜索 CurseForge 官方 MOD",
        WebCommandRisk::Read,
    ),
    policy(
        "refresh_instance_status",
        "刷新实例状态",
        WebCommandRisk::Read,
    ),
    policy("query_logs", "查询日志", WebCommandRisk::Read),
    policy("list_backups", "读取备份列表", WebCommandRisk::Read),
    policy(
        "list_instance_files",
        "浏览实例目录文件",
        WebCommandRisk::Read,
    ),
    policy(
        "ensure_storage_directories",
        "检查并创建存储目录",
        WebCommandRisk::Write,
    ),
    policy("check_steamcmd", "检查 SteamCMD", WebCommandRisk::Write),
    policy(
        "clear_startup_auto_update_skip_flags",
        "清除启动自动更新跳过标记",
        WebCommandRisk::Write,
    ),
    policy("create_backup", "创建备份", WebCommandRisk::Write),
    policy(
        "export_instance_config_for_download",
        "下载所选实例配置",
        WebCommandRisk::Write,
    ),
    policy(
        "export_cluster_for_download",
        "下载整个集群配置",
        WebCommandRisk::Write,
    ),
    policy(
        "create_instance_file_entry",
        "在实例目录中新建文件或文件夹",
        WebCommandRisk::Write,
    ),
    policy(
        "rename_instance_file_entry",
        "重命名实例目录项",
        WebCommandRisk::Write,
    ),
    policy(
        "copy_instance_file_entry",
        "复制实例目录项",
        WebCommandRisk::Write,
    ),
    policy(
        "install_steamcmd",
        "安装或初始化 SteamCMD",
        WebCommandRisk::High,
    ),
    policy("save_instance_config", "保存实例配置", WebCommandRisk::High),
    policy(
        "update_instance_mods",
        "更新实例 MOD 列表",
        WebCommandRisk::High,
    ),
    policy(
        "install_or_update_instance",
        "安装或更新 ASA 服务端",
        WebCommandRisk::High,
    ),
    policy("start_instance", "启动实例", WebCommandRisk::High),
    policy("stop_instance", "停止实例", WebCommandRisk::High),
    policy("restart_instance", "重启实例", WebCommandRisk::High),
    policy("clear_logs", "清空全部管理器日志", WebCommandRisk::High),
    policy(
        "clear_scoped_logs",
        "清空指定范围日志",
        WebCommandRisk::High,
    ),
    policy(
        "unban_web_security_ip",
        "解除 Web 安全封禁",
        WebCommandRisk::High,
    ),
    policy("save_settings", "保存全局设置", WebCommandRisk::High),
    policy("create_instance", "创建实例", WebCommandRisk::High),
    policy(
        "apply_instance_config",
        "应用配置并重启实例",
        WebCommandRisk::High,
    ),
    policy(
        "execute_rcon_command",
        "执行 RCON 命令",
        WebCommandRisk::High,
    ),
    policy("restore_backup", "恢复备份", WebCommandRisk::High),
    policy(
        "import_instance_config_upload",
        "上传并导入实例配置",
        WebCommandRisk::High,
    ),
    policy(
        "delete_instance_file_entry",
        "删除实例目录中的文件或文件夹",
        WebCommandRisk::High,
    ),
    policy("delete_instance", "删除实例", WebCommandRisk::High),
];

const fn policy(
    command: &'static str,
    label: &'static str,
    risk: WebCommandRisk,
) -> WebCommandPolicy {
    WebCommandPolicy {
        command,
        label,
        risk,
    }
}

pub(crate) fn web_command_policies() -> &'static [WebCommandPolicy] {
    WEB_COMMAND_POLICIES
}

pub(crate) fn web_command_policy(command: &str) -> Result<WebCommandRisk, String> {
    WEB_COMMAND_POLICIES
        .iter()
        .find(|policy| policy.command == command)
        .map(|policy| policy.risk)
        .ok_or_else(|| format!("未知 Web API 命令：{command}"))
}
