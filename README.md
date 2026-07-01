# 方舟进化飞升服务器管理器 · 交互原型

基于 Tauri 2、React 19 与 Ant Design 6.5 的桌面端交互原型。界面参考需求截图，重点实现实例列表、集群状态、实例配置编辑、MOD 管理与日志参数配置。

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

字段提示区分了 ARK 的 `GameUserSettings.ini`、`Game.ini`、启动参数，以及管理器自身的资源监控/轮转策略。当前版本是纯交互原型，保存操作写入浏览器 WebView 的 `localStorage`，尚未读写真实服务端文件或启动 ARK 进程。

## 验证命令

```bash
npm install
npm run build
npm run tauri build -- --no-bundle
```

按需求不自动运行应用；需要人工预览时再执行 `npm run tauri dev`。

## 参数参考

- [ARK Official Community Wiki · Server configuration](https://ark.wiki.gg/wiki/Server_configuration)
- [Tauri releases](https://github.com/tauri-apps/tauri/releases)
- [React versions](https://react.dev/versions)
- [Ant Design changelog](https://ant.design/components/changelog/)
