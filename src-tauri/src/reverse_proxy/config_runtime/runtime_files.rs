use std::fs;

use crate::reverse_proxy_ip_whitelist::render_ip_whitelist_cidr_file;

use super::super::ReverseProxyConfig;

impl ReverseProxyConfig {
    pub(in crate::reverse_proxy) fn prepare_runtime_files(&self) -> Result<(), String> {
        self.create_runtime_directories()?;
        self.write_openresty_config()?;
        self.write_security_lua()?;
        self.write_ip_whitelist_cidrs()?;
        Ok(())
    }

    fn create_runtime_directories(&self) -> Result<(), String> {
        for relative_dir in ["conf", "logs", "temp", "lualib"] {
            let path = self.proxy_root_path.join(relative_dir);
            fs::create_dir_all(&path).map_err(|error| {
                format!("无法创建 Web 反向代理运行目录 {}：{error}", path.display())
            })?;
        }
        Ok(())
    }

    fn write_openresty_config(&self) -> Result<(), String> {
        let config_path = self.config_path();
        fs::write(&config_path, self.render_config()).map_err(|error| {
            format!(
                "无法写入 Web 反向代理 OpenResty 配置 {}：{error}",
                config_path.display()
            )
        })
    }

    fn write_security_lua(&self) -> Result<(), String> {
        let lua_path = self.security_lua_path();
        fs::write(&lua_path, self.render_security_lua()).map_err(|error| {
            format!(
                "无法写入 Web 反向代理 Lua 安全脚本 {}：{error}",
                lua_path.display()
            )
        })
    }

    fn write_ip_whitelist_cidrs(&self) -> Result<(), String> {
        let cidr_path = self.ip_whitelist_cidr_path();
        let cidr_content = render_ip_whitelist_cidr_file(&self.ip_whitelist)?;
        fs::write(&cidr_path, cidr_content).map_err(|error| {
            format!(
                "无法写入 Web 访问 IP 白名单 CIDR 文件 {}：{error}",
                cidr_path.display()
            )
        })
    }
}
