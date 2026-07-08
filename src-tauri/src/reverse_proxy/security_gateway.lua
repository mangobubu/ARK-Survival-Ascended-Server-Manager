
local _M = {}

local config = {
    domain = "",
    public_port = 0,
    ip_whitelist_cidr_path = "conf/asa-ip-whitelist-cidrs.txt",
    login_failure_ban_threshold = 5,
    login_failure_ban_seconds = 1800,
    rate_limit_per_minute = 180,
    api_rate_limit_per_minute = 120,
    login_rate_limit_per_minute = 10,
    path_risk_ban_threshold = 3,
    path_risk_ban_seconds = 3600,
    body_risk_ban_seconds = 3600,
    redis_enabled = false,
    redis_host = "127.0.0.1",
    redis_port = 6379,
    redis_timeout_ms = 80,
    redis_whitelist_key = "asa:web:ip:whitelist",
    redis_blacklist_key = "asa:web:ip:blacklist",
}

local abnormal_ua_patterns = {
    "sqlmap",
    "nikto",
    "nmap",
    "masscan",
    "zgrab",
    "acunetix",
    "nessus",
    "openvas",
    "dirbuster",
    "gobuster",
    "wpscan",
    "curl",
    "wget",
    "python%-requests",
    "go%-http%-client",
    "java/",
    "powershell",
    "winhttp",
}

local dangerous_body_patterns = {
    "__proto__",
    "constructor%.prototype",
    "constructor%[prototype%]",
    "%$where",
    "<script",
    "javascript:",
    "%${jndi:",
    "%.%.%/",
    "%.%.\\",
    "%%2e%%2e",
    "union%s+select",
    "drop%s+table",
    "insert%s+into",
    "delete%s+from",
    "cmd%.exe",
    "powershell",
    "certutil",
    "bitsadmin",
}

local suspicious_path_patterns = {
    "%.env",
    "wp%-login%.php",
    "phpmyadmin",
    "server%-status",
    "%.git",
    "%.svn",
    "%.bak$",
    "%.sql$",
    "%.zip$",
    "%.tar",
    "%.7z$",
    "%%2e%%2e",
    "%.%.%/",
    "%.%.\\",
    "/admin",
    "/actuator",
    "/boaform",
    "/cgi%-bin",
}

function _M.configure(next_config)
    if type(next_config) ~= "table" then
        return
    end
    for key, value in pairs(next_config) do
        config[key] = value
    end
end

local function json_escape(value)
    value = tostring(value or "")
    value = value:gsub("\\", "\\\\")
    value = value:gsub('"', '\\"')
    value = value:gsub("\r", "\\r")
    value = value:gsub("\n", "\\n")
    return value
end

local function deny(status, message)
    ngx.status = status
    ngx.header["Content-Type"] = "application/json; charset=utf-8"
    ngx.say('{"ok":false,"error":"' .. json_escape(message) .. '"}')
    return ngx.exit(status)
end

local function json_response(status, body)
    ngx.status = status
    ngx.header["Content-Type"] = "application/json; charset=utf-8"
    ngx.say(body)
    return ngx.exit(status)
end

local function client_ip()
    return ngx.var.remote_addr or "0.0.0.0"
end

local function now_bucket(seconds)
    return math.floor(ngx.now() / seconds)
end

local function incr(dict, key, ttl)
    local value, err = dict:incr(key, 1, 0, ttl)
    if value then
        return value
    end
    dict:set(key, 1, ttl)
    return 1
end

local function ban_source(reason)
    reason = tostring(reason or "")
    if reason:find("登录") then
        return "login"
    end
    if reason:find("User%-Agent") or reason:find("User-Agent") then
        return "ua"
    end
    if reason:find("请求体") then
        return "body"
    end
    if reason:find("路径") then
        return "path"
    end
    if reason:find("频繁") then
        return "rate"
    end
    return "security"
end

local function encode_ban_record(reason, seconds)
    local safe_reason = tostring(reason or "blocked")
    return table.concat({
        safe_reason,
        tostring(math.floor(ngx.now() * 1000)),
        tostring(tonumber(seconds) or 0),
        ban_source(safe_reason),
    }, "\t")
end

local function shared_dict_ttl(dict, key)
    local ok, ttl = pcall(function()
        return dict:ttl(key)
    end)
    if ok and type(ttl) == "number" then
        return ttl
    end
    return 0
end

local function decode_ban_record(value, remaining_seconds)
    local text = tostring(value or "")
    local reason, banned_at_ms, duration_seconds, source = text:match("^([^\t]*)\t([^\t]*)\t([^\t]*)\t([^\t]*)$")
    if not reason then
        reason = text ~= "" and text or "blocked"
        banned_at_ms = "0"
        duration_seconds = "0"
        source = ban_source(reason)
    end
    return {
        reason = reason,
        banned_at_ms = tonumber(banned_at_ms) or 0,
        duration_seconds = tonumber(duration_seconds) or 0,
        source = source or ban_source(reason),
        remaining_seconds = math.max(0, math.floor(tonumber(remaining_seconds) or 0)),
    }
end

local function ban_ip(ip, seconds, reason)
    ngx.shared.asa_ip_bans:set("ban:" .. ip, encode_ban_record(reason, seconds), seconds)
end

local function ip_to_number(ip)
    local a, b, c, d = ip:match("^(%d+)%.(%d+)%.(%d+)%.(%d+)$")
    a, b, c, d = tonumber(a), tonumber(b), tonumber(c), tonumber(d)
    if not a or not b or not c or not d then
        return nil
    end
    if a > 255 or b > 255 or c > 255 or d > 255 then
        return nil
    end
    return ((a * 256 + b) * 256 + c) * 256 + d
end

local function mask(bits)
    if bits <= 0 then
        return 0
    end
    return (0xffffffff - (2 ^ (32 - bits) - 1))
end

local function cidr_match(ip_num, cidr)
    local network, bits = cidr:match("^%s*(%d+%.%d+%.%d+%.%d+)%s*/%s*(%d+)%s*$")
    bits = tonumber(bits)
    if not network or not bits or bits < 0 or bits > 32 then
        return false
    end
    local network_num = ip_to_number(network)
    if not network_num then
        return false
    end
    local m = mask(bits)
    return (ip_num - (ip_num % (2 ^ (32 - bits)))) == (network_num - (network_num % (2 ^ (32 - bits))))
end

local function is_private_or_loopback(ip)
    local ip_num = ip_to_number(ip)
    if not ip_num then
        return false
    end
    return cidr_match(ip_num, "10.0.0.0/8")
        or cidr_match(ip_num, "127.0.0.0/8")
        or cidr_match(ip_num, "172.16.0.0/12")
        or cidr_match(ip_num, "192.168.0.0/16")
        or cidr_match(ip_num, "100.64.0.0/10")
end

local function load_ip_whitelist_cidrs()
    local cache = ngx.shared.asa_cidr_cache
    local cached = cache:get("ip_whitelist_cidrs")
    if cached then
        local cidrs = {}
        for cidr in cached:gmatch("[^\n]+") do
            cidrs[#cidrs + 1] = cidr
        end
        return cidrs
    end

    local file = io.open(config.ip_whitelist_cidr_path, "r")
    local cidrs = {}
    if file then
        for line in file:lines() do
            local cidr = line:gsub("#.*$", ""):match("^%s*(.-)%s*$")
            if cidr ~= "" then
                cidrs[#cidrs + 1] = cidr
            end
        end
        file:close()
    end

    cache:set("ip_whitelist_cidrs", table.concat(cidrs, "\n"), 300)
    return cidrs
end

local function is_ip_whitelisted(ip)
    if is_private_or_loopback(ip) then
        return true
    end
    local ip_num = ip_to_number(ip)
    if not ip_num then
        return false
    end
    for _, cidr in ipairs(load_ip_whitelist_cidrs()) do
        if cidr_match(ip_num, cidr) then
            return true
        end
    end
    return false
end

local function redis_ip_policy(ip)
    if not config.redis_enabled then
        return nil
    end
    local ok, redis = pcall(require, "resty.redis")
    if not ok then
        return nil
    end
    local red = redis:new()
    red:set_timeout(config.redis_timeout_ms)
    local connected = red:connect(config.redis_host, config.redis_port)
    if not connected then
        return nil
    end
    local whitelist = red:sismember(config.redis_whitelist_key, ip)
    if whitelist == 1 then
        red:set_keepalive(10000, 16)
        return "allow"
    end
    local blacklist = red:sismember(config.redis_blacklist_key, ip)
    red:set_keepalive(10000, 16)
    if blacklist == 1 then
        return "deny"
    end
    return nil
end

local function has_abnormal_ua()
    local ua = (ngx.var.http_user_agent or ""):lower()
    if ua == "" then
        return true, "空 User-Agent"
    end
    for _, pattern in ipairs(abnormal_ua_patterns) do
        if ua:find(pattern) then
            return true, "异常 User-Agent"
        end
    end
    return false, nil
end

local function is_suspicious_path(uri)
    local lower_uri = (uri or ""):lower()
    for _, pattern in ipairs(suspicious_path_patterns) do
        if lower_uri:find(pattern) then
            return true
        end
    end
    return false
end

local function request_body_has_danger()
    local method = ngx.req.get_method()
    if method ~= "POST" and method ~= "PUT" and method ~= "PATCH" then
        return false
    end
    local content_type = (ngx.var.content_type or ""):lower()
    if content_type ~= "" and not content_type:find("json") and not content_type:find("form") and not content_type:find("text") then
        return false
    end
    ngx.req.read_body()
    local body = ngx.req.get_body_data()
    if not body then
        return false
    end
    local lower_body = body:lower()
    for _, pattern in ipairs(dangerous_body_patterns) do
        if lower_body:find(pattern) then
            return true
        end
    end
    return false
end

local function enforce_rate(ip, uri)
    local bucket = now_bucket(60)
    local counters = ngx.shared.asa_rate_counters
    local total = incr(counters, "total:" .. ip .. ":" .. bucket, 70)
    if total > config.rate_limit_per_minute then
        return false, "访问过于频繁"
    end
    if uri:find("^/api/") then
        local api = incr(counters, "api:" .. ip .. ":" .. bucket, 70)
        if api > config.api_rate_limit_per_minute then
            return false, "API 访问过于频繁"
        end
    end
    if uri == "/api/auth/login" then
        local login = incr(counters, "login:" .. ip .. ":" .. bucket, 70)
        if login > config.login_rate_limit_per_minute then
            ban_ip(ip, config.login_failure_ban_seconds, "登录接口访问过于频繁")
            return false, "登录接口访问过于频繁，IP 已临时封禁"
        end
    end
    return true, nil
end

local function enforce_path_risk(ip, uri)
    if not is_suspicious_path(uri) then
        return true, nil
    end
    local risks = incr(ngx.shared.asa_path_risks, "path:" .. ip, 600)
    if risks >= config.path_risk_ban_threshold then
        ban_ip(ip, config.path_risk_ban_seconds, "异常路径探测")
        return false, "异常路径探测过多，IP 已临时封禁"
    end
    return false, "异常路径已拦截"
end

local function ban_record_json(ip, record)
    return '{'
        .. '"ip":"' .. json_escape(ip) .. '",'
        .. '"reason":"' .. json_escape(record.reason) .. '",'
        .. '"source":"' .. json_escape(record.source) .. '",'
        .. '"bannedAtMs":' .. tostring(record.banned_at_ms) .. ','
        .. '"remainingSeconds":' .. tostring(record.remaining_seconds)
        .. '}'
end

function _M.admin_bans()
    local bans = ngx.shared.asa_ip_bans
    local ok, keys = pcall(function()
        return bans:get_keys(0)
    end)
    if not ok or type(keys) ~= "table" then
        keys = {}
    end
    table.sort(keys)

    local records = {}
    for _, key in ipairs(keys) do
        local ip = key:match("^ban:(.+)$")
        if ip then
            local value = bans:get(key)
            if value then
                local ttl = shared_dict_ttl(bans, key)
                local record = decode_ban_record(value, ttl)
                records[#records + 1] = ban_record_json(ip, record)
            end
        end
    end

    return json_response(200, '{"ok":true,"data":[' .. table.concat(records, ",") .. ']}')
end

function _M.admin_unban()
    local ip = ngx.var.arg_ip or ""
    if not ip_to_number(ip) then
        return json_response(400, '{"ok":false,"error":"手动解封 IP 必须是合法 IPv4 地址"}')
    end

    local key = "ban:" .. ip
    local bans = ngx.shared.asa_ip_bans
    local existed = bans:get(key) ~= nil
    bans:delete(key)
    ngx.shared.asa_login_failures:delete("login:" .. ip)
    ngx.shared.asa_path_risks:delete("path:" .. ip)

    return json_response(
        200,
        '{"ok":true,"data":{"ip":"' .. json_escape(ip) .. '","existed":' .. tostring(existed) .. '}}'
    )
end

function _M.access()
    local ip = client_ip()
    local uri = ngx.var.uri or "/"
    local policy = redis_ip_policy(ip)
    if policy == "allow" then
        return
    end
    if policy == "deny" then
        return deny(403, "IP 已命中 Redis 黑名单")
    end

    local ban_key = "ban:" .. ip
    local ban_value = ngx.shared.asa_ip_bans:get(ban_key)
    if ban_value then
        local ban_record = decode_ban_record(ban_value, shared_dict_ttl(ngx.shared.asa_ip_bans, ban_key))
        return deny(403, "IP 已被临时封禁：" .. ban_record.reason)
    end

    if not is_ip_whitelisted(ip) then
        return deny(451, "当前 IP 不在 Web 访问白名单")
    end

    local ua_blocked, ua_reason = has_abnormal_ua()
    if ua_blocked then
        ban_ip(ip, 600, ua_reason)
        return deny(403, ua_reason)
    end

    local ok, rate_reason = enforce_rate(ip, uri)
    if not ok then
        return deny(429, rate_reason)
    end

    local safe_path, path_reason = enforce_path_risk(ip, uri)
    if not safe_path then
        return deny(403, path_reason)
    end

    if request_body_has_danger() then
        ban_ip(ip, config.body_risk_ban_seconds, "请求体包含危险字段")
        return deny(403, "请求体包含危险字段，IP 已临时封禁")
    end
end

function _M.log()
    if (ngx.var.uri or "") ~= "/api/auth/login" then
        return
    end
    local ip = client_ip()
    local status = tonumber(ngx.status) or 0
    local failures = ngx.shared.asa_login_failures
    if status == 401 then
        local count = incr(failures, "login:" .. ip, 600)
        if count >= config.login_failure_ban_threshold then
            failures:delete("login:" .. ip)
            ban_ip(ip, config.login_failure_ban_seconds, "登录失败次数过多")
        end
    elseif status >= 200 and status < 300 then
        failures:delete("login:" .. ip)
    end
end

return _M
