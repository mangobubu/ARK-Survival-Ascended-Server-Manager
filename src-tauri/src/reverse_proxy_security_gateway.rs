pub(crate) fn render_security_lua() -> String {
    SECURITY_LUA_SCRIPT.to_string()
}

const SECURITY_LUA_SCRIPT: &str = include_str!("reverse_proxy/security_gateway.lua");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 生成_lua_安全脚本包含动态封禁和风险检测() {
        let lua = render_security_lua();

        assert!(lua.contains("login_failure_ban_threshold"));
        assert!(lua.contains("admin_bans"));
        assert!(lua.contains("admin_unban"));
        assert!(lua.contains("abnormal_ua_patterns"));
        assert!(lua.contains("dangerous_body_patterns"));
        assert!(lua.contains("当前 IP 不在 Web 访问白名单"));
        assert!(lua.contains("redis_whitelist_key"));
        assert!(lua.contains("redis_blacklist_key"));
    }
}
