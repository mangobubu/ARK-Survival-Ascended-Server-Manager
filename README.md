# 方舟进化飞升服务器管理器

基于 Tauri 2、Rust、React 19 与 Ant Design 6.5 的桌面端 ASA 服务器管理器。前端负责交互界面，实例、配置、安装更新、进程控制、日志、备份、导入导出等核心能力统一由 Rust 后端模块承载。

## 平台支持

- 当前项目仅面向 **Windows x64** 架构发布。
- 原因：ARK: Survival Ascended Dedicated Server 当前运行与管理场景以 Windows 服务端环境为核心，本管理器也围绕 Windows 下的 ASA 服务端目录、进程与 SteamCMD 管理能力设计。
- GitHub Releases 只会提供 Windows x64 构建产物，不提供 Linux、macOS 或 ARM 架构包。

## 技术版本

- Tauri CLI / Rust `2.11.4`，JavaScript API `2.11.1`
- React / React DOM `19.2.7`
- Ant Design `6.5.0`
- Vite `8.x`
- TypeScript `6.x`

## 配置分组

- **基础设置**：服务器名称、端口、RCON、集群、常用倍率、PvE/PvP 与自动维护。
- **高级设置**：世界节奏、生态、繁殖、建筑、部落、白名单与跨服传输控制。
- **性能设置**：进程资源调度、网络 Tick、RCON 缓冲、存档策略和启动参数预览。
- **MOD 设置**：CurseForge MOD ID、加载顺序、启用状态、更新策略与 `ActiveMods` 预览。
- **日志参数**：`-servergamelog`、部落日志 RCON 输出、管理员审计、轮转、保留和日志筛选。

字段提示区分了 ARK 的 `GameUserSettings.ini`、`Game.ini`、启动参数，以及管理器自身的资源监控/轮转策略。全局设置、实例、MOD、配置与日志均由 Rust 后端持久化保存，实例配置会写入真实 ARK 配置文件。

## Rust 后端模块

- **状态持久化**：全局设置、实例列表、配置、MOD 和日志统一写入应用数据目录。
- **实例管理**：创建实例、校验端口、维护状态、安装路径、RCON 端口和运行信息。
- **配置渲染**：根据界面配置生成 `GameUserSettings.ini`、`Game.ini` 和 ASA 启动参数。
- **服务端安装/更新**：通过 SteamCMD 匿名登录执行 ASA Dedicated Server `2430930` 的 `app_update validate`。
- **进程控制**：启动 ASA 服务端可执行文件，捕获 stdout/stderr，停止时优先通过 RCON 保存世界。
- **日志与状态事件**：Rust 后台推送 `asa:log-line` 和 `asa:instance-status` 事件，主窗口实时刷新。
- **备份与导入导出**：压缩 `ShooterGame/Saved`，支持实例/集群 JSON 导出与导入。

## SteamCMD 初始化

- 应用启动时会检查全局设置中的 SteamCMD 目录。
- 可选择已有的 `steamcmd.exe`，或选择上级目录并由管理器下载 Valve 官方安装包。
- 下载、解压和首次初始化均由 Rust 后台执行；Windows 下不会显示 SteamCMD 控制台窗口。
- 下载界面显示百分比、实时速度、已下载大小和总大小，失败时不会覆盖已有非空目录。

## 存储目录初始化

- 主窗口启动后会检查全局设置中的服务器存储目录和备份存储目录。
- 目录不存在时会自动递归创建；路径无效或创建失败时会在界面提示具体错误。
- 全局设置不再依赖 WebView `localStorage`，设置窗口保存后会写入 Rust 后端状态并同步到主窗口。

## 验证命令

```bash
npm install
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri -- build --no-bundle
npm run tauri -- dev
```

## 发布规则

- GitHub Release 工作流位于 `.github/workflows/release.yml`。
- 只有推送形如 `v0.1.0`、`v1.2.3` 的 `v*.*.*` 版本标签时，才会触发 Release 构建。
- Release 工作流固定使用 `windows-latest`，并显式指定 `x86_64-pc-windows-msvc` 目标，只生成 Windows x64 发布产物。
- 示例：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 参数参考

- [ARK Official Community Wiki · Server configuration](https://ark.wiki.gg/wiki/Server_configuration)
- [Tauri releases](https://github.com/tauri-apps/tauri/releases)
- [React versions](https://react.dev/versions)
- [Ant Design changelog](https://ant.design/components/changelog/)
