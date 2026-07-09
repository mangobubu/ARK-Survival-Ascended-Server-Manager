# 更新日志

## v0.3.3

- 修复 OpenResty for Windows 启动时使用相对 Lua 路径，导致 `asa_security` 安全网关脚本无法加载、域名反向代理无法访问的问题。
- 将 OpenResty 生成配置中的 `lua_package_path` 与 IP 白名单 CIDR 文件路径统一写入标准化绝对路径，避免工作目录差异造成脚本或白名单文件读取失败。
- 强化 OpenResty 启动状态检测逻辑：启动前清理旧 PID 文件，启动后读取 PID 并确认进程真实存活，不再因空 PID 文件误判反代已启动。
- 优化 OpenResty 启动失败诊断信息，失败时会带出 PID 文件状态与最近错误日志，便于快速定位配置或运行时问题。
- 增加同配置重保存时的进程存活校验，OpenResty 被外部退出后会自动重新拉起，减少用户手动排障成本。
- 补充反向代理配置渲染回归测试，并完成 `cargo test` 与 `npm run build` 验证。
- 同步项目版本号至 `0.3.3`，并补充 Release 工作流可读取的版本更新日志。

## v0.3.2

- 修复启用域名反向代理 HTTPS 时，OpenResty 将证书相对路径误解析到 `web-reverse-proxy/conf/certs` 导致配置校验失败的问题。
- 调整 OpenResty 配置生成逻辑，SSL 证书链与私钥统一写入带引号的标准化绝对路径，兼容 Windows 用户目录与应用数据目录。
- 补充 HTTPS 证书路径渲染回归测试，确保后续不会再次生成 `certs/...` 或 `conf/certs/...` 形式的错误证书引用。
- 同步项目版本号至 `0.3.2`，并补充 Release 工作流可读取的版本更新日志。

## v0.3.1

- 修复腾讯云 DNSPod 在 ACME DNS-01 验证中遇到已存在 `_acme-challenge` TXT 记录时无法继续申请 SSL 证书的问题。
- 新增 DNSPod `DescribeRecordList` 与 `ModifyTXTRecord` 幂等处理流程，记录已存在时会自动查询并更新为本次 ACME 验证 token。
- 优化 DNS TXT 验证记录清理策略：新建记录会删除，复用记录会保留，被临时修改的已有记录会在验证结束后还原。
- 补充 DNSPod 查询、修改、记录已存在错误识别与 ACME 清理策略相关单元测试，降低后续回归风险。
- 同步项目版本号至 `0.3.1`，并让 Release 工作流读取当前 tag 对应的更新日志写入 GitHub Release 页面。
