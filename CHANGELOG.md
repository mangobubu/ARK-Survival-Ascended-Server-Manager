# 更新日志

## v0.3.8

- 修复 ASA 服务端“采集数量倍率”保存并应用重启后仍可能表现为官方 1x 基础数量的问题，在继续写入 `GameUserSettings.ini` 的 `HarvestAmountMultiplier` 基础上，同步把采集倍率追加到启动 URL，确保本次启动明确使用管理器配置值。
- 同步为 `HarvestHealthMultiplier` 增加启动 URL 兜底，保持采集数量倍率与采集物耐久倍率的运行时配置链路一致。
- 优化“采集数量倍率”界面提示，明确该字段对应 `HarvestAmountMultiplier`，并提示 ASA 官方 1x 若显示 `+2`，设置 5x 理论应显示 `+10`。
- 补充采集倍率启动参数顺序测试，确保新增倍率参数位于 `ServerAdminPassword` 之前，不破坏 ASA 管理员密码必须作为尾部 URL 选项的兼容处理。
- 补充配置渲染测试覆盖 `HarvestAmountMultiplier=5` 与 `HarvestHealthMultiplier=1.5` 的 `GameUserSettings.ini` 写入结果，降低后续倍率配置回归风险。
- 完成 `cargo test` 与 `npm run build` 验证，确认后端配置生成、启动参数构建和前端构建无回归。
- 同步项目版本号至 `0.3.8`，确保前端包版本、Tauri 配置、Cargo 包信息、锁文件和发布标签版本一致。

## v0.3.7

- 新增 ASA 服务端玩家进退服日志识别，将 `joined this ARK`、`joined the server`、`Join succeeded`、`left this ARK`、`logged out` 等服务端日志事件转写到集群“应用日志”页签。
- 新增基于 RCON `ListPlayers` 的在线玩家名单快照差异检测，不再只依赖 ASA 原始游戏日志文本，玩家加入或退出会在下一次状态轮询后稳定写入应用日志。
- 应用日志现在会按现有格式显示 `[时间] [实例名] 玩家名 加入了服务器 / 退出了服务器`，便于在集群视角快速追踪玩家进入与登出。
- 抽离 `server_player_events` 解析模块并补充 RCON 玩家列表解析、在线玩家差异快照、中英文玩家名、引号玩家名、Join succeeded 与无关日志过滤单元测试，避免把普通 ASA 服务端日志误判为玩家事件。
- 保持服务端窗口日志与游戏日志文件原始输出不变，仅额外生成面向管理员阅读的应用日志，降低改动范围并避免破坏现有实例日志面板。
- 完成 `cargo clippy --manifest-path src-tauri\Cargo.toml --all-targets -- -D warnings`、`cargo test --manifest-path src-tauri\Cargo.toml`、`npm run build`、`npm run check:config-metadata` 与本地 Vite 预览浏览器验证，确认构建、后端测试和 Web 登录页加载无回归。
- 同步项目版本号至 `0.3.7`，确保前端页脚、Tauri 配置、Cargo 包信息、锁文件和发布标签版本一致。

## v0.3.6

- 修复 Web 端安装/更新 ASA 服务端时桌面端未同步显示下载/更新进度的问题，新增桌面端与 Web 端共用的 2 秒跨端同步兜底刷新机制。
- 修复 Web 端保存配置后桌面端集群日志不实时更新的问题，将配置保存、备份恢复、启动停止、Web 服务状态等用户可见日志统一改为实时事件发布。
- 收口剩余业务路径中的 `runtime.add_log(...)` 直写调用，仅保留 `command_events::emit_instance_log` 作为统一日志写入与广播入口，避免后续新增功能再次漏发实时日志事件。
- 优化前端日志追加逻辑，按日志 `id` 去重，避免实时事件与兜底刷新同时命中时出现重复日志。
- 完成 `npm run build`、`cargo check`、`cargo test` 与 Tauri 开发版启动验证，确认桌面端和 Web 端同步链路无回归。
- 同步项目版本号至 `0.3.6`，确保前端页脚、Tauri 配置、Cargo 包信息、锁文件和发布标签版本一致。

## v0.3.5

- 新增 Web 端运行主机 ASA 服务端文件夹选择器，点击添加实例中的目录按钮即可浏览服务器存储目录内的主机文件夹。
- 新增只读后端目录浏览命令 `list_host_directories`，返回子目录、上级目录、ASA 配置文件和 `ArkAscendedServer.exe` 检测结果。
- 保留桌面端 Tauri 原生目录选择器逻辑，Web 端选择目录后复用现有配置导入流程自动回填路径、端口、管理员密码、地图和 MOD 信息。
- 强化 Web 目录浏览路径边界，只允许访问服务器存储目录内的文件夹，并补充越界访问、最近存在目录回退和 ASA 目录识别单元测试。
- 完成 `npm run build`、`cargo test --manifest-path src-tauri\Cargo.toml` 和浏览器 Web 交互验证，确认 Web 端不再提示只能手动输入路径。
- 同步项目版本号至 `0.3.5`，确保前端页脚、Tauri 配置、Cargo 包信息和发布标签版本一致。

## v0.3.4

- 修复应用左下角页脚长期硬编码显示 `v0.1.0 Rust Backend`，导致版本号不随发布版本变化的问题。
- 改为通过 Vite 构建期从 `package.json` 注入前端版本常量，页脚统一展示当前应用版本，避免后续版本展示再次漂移。
- 同步 `package.json`、`package-lock.json`、Tauri 配置、Cargo 包信息与锁文件版本至 `0.3.4`。
- 补充前端 `__APP_VERSION__` 类型声明，保持 TypeScript 严格类型检查通过。
- 完成 `npm run build`、`npm run check:config-metadata`、`cargo test` 验证，并启动本地 Vite 页面通过浏览器确认页面加载与构建产物版本文案。

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
