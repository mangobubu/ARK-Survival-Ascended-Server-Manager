import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  AppstoreAddOutlined,
  CheckCircleOutlined,
  CloudDownloadOutlined,
  CloudServerOutlined,
  DatabaseOutlined,
  EditOutlined,
  EllipsisOutlined,
  ExclamationCircleFilled,
  ExportOutlined,
  FolderOpenOutlined,
  ImportOutlined,
  LineChartOutlined,
  PlayCircleFilled,
  PlusOutlined,
  ReloadOutlined,
  SaveOutlined,
  SettingOutlined,
  StopFilled,
  TeamOutlined,
} from '@ant-design/icons'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { Button, Checkbox, Dropdown, Empty, message, Modal, Progress, Space, Table, Tag, Tooltip, Typography } from 'antd'
import type { ColumnsType } from 'antd/es/table'
import type { MenuProps } from 'antd'
import ConfigPanel from './ConfigPanel'
import { defaultConfig } from './data'
import {
  applyInstanceConfig,
  checkModUpdates,
  clearLogs,
  createBackup,
  exportCluster,
  exportInstanceConfig,
  getInstanceConfig,
  getInstanceMods,
  importInstanceConfig,
  installOrUpdateInstance,
  listInstances,
  openInstanceDirectory,
  queryLogs,
  refreshInstanceStatus,
  saveInstanceConfig,
  startInstance as startInstanceCommand,
  stopInstance as stopInstanceCommand,
} from './backendApi'
import { loadGlobalSettings, loadGlobalSettingsFromBackend, subscribeGlobalSettings } from './globalSettings'
import SteamCmdSetup from './SteamCmdSetup'
import type {
  GlobalSettings,
  InstanceCreatedEvent,
  JobProgress,
  LogLine,
  ModItem,
  ServerConfig,
  ServerInstance,
  ServerStatus,
} from './types'
import { ADD_INSTANCE_CREATED_EVENT, ADD_INSTANCE_WINDOW_LABEL } from './windowEvents'

const { Text } = Typography

const mapGlyphs: Record<string, string> = {
  'The Island': '◆',
  'Scorched Earth': '✣',
  Aberration: '◇',
  'The Center': '✦',
  Extinction: '⬡',
  Astraeos: '✧',
  Ragnarok: 'ᛉ',
  Valguero: '△',
  'Lost Colony': '◈',
}

function Logo() {
  return (
    <div className="brand">
      <img className="brand__emblem" src="/app-icon.png" alt="ASA 服务器管理器" />
      <div><div className="brand__title">方舟进化飞升服务器管理器</div><div className="brand__subtitle">ARK SURVIVAL ASCENDED SERVER MANAGER</div></div>
    </div>
  )
}

function StatCard({ icon, label, value, suffix, tone = 'blue' }: { icon: React.ReactNode; label: string; value: string | number; suffix: string; tone?: 'blue' | 'green' }) {
  return (
    <div className={`stat-card stat-card--${tone}`}>
      <div className="stat-card__icon">{icon}</div>
      <div><div className="stat-card__label">{label}</div><div className="stat-card__value">{value} <small>{suffix}</small></div></div>
    </div>
  )
}

function statusMeta(status: ServerStatus) {
  if (status === 'running') return { color: 'success', text: '⊙ 运行中' }
  if (status === 'starting') return { color: 'processing', text: '◌ 启动中' }
  if (status === 'updating') return { color: 'processing', text: '↻ 更新中' }
  if (status === 'backingUp') return { color: 'processing', text: '▣ 备份中' }
  if (status === 'error') return { color: 'error', text: '⊗ 异常' }
  return { color: 'default', text: '⊖ 已停止' }
}

export default function App() {
  const [instances, setInstances] = useState<ServerInstance[]>([])
  const [selectedId, setSelectedId] = useState('')
  const [selectedRows, setSelectedRows] = useState<React.Key[]>([])
  const [config, setConfig] = useState<ServerConfig>(defaultConfig)
  const [mods, setMods] = useState<ModItem[]>([])
  const [logs, setLogs] = useState<LogLine[]>([])
  const [dirty, setDirty] = useState(false)
  const [globalSettings, setGlobalSettings] = useState<GlobalSettings>(loadGlobalSettings)
  const [jobProgress, setJobProgress] = useState<Record<string, JobProgress>>({})
  const [checkingMods, setCheckingMods] = useState(false)
  const [messageApi, contextHolder] = message.useMessage()
  const [modal, modalContext] = Modal.useModal()

  const selected = instances.find((item) => item.id === selectedId)
  const running = instances.filter((item) => item.status === 'running').length
  const totalPlayers = instances.reduce((sum, item) => sum + item.players, 0)
  const playerCapacity = instances.reduce((sum, item) => sum + item.maxPlayers, 0)

  const replaceInstance = useCallback((next: ServerInstance) => {
    setInstances((current) => current.some((item) => item.id === next.id)
      ? current.map((item) => item.id === next.id ? next : item)
      : [...current, next])
  }, [])

  const appendLogLine = useCallback((line: LogLine) => {
    setLogs((current) => [...current, line].slice(-500))
  }, [])

  const loadInstances = useCallback(async () => {
    const loaded = await listInstances()
    setInstances(loaded)
    setSelectedId((current) => {
      if (current && loaded.some((item) => item.id === current)) return current
      return loaded[0]?.id ?? ''
    })
    setSelectedRows((current) => current.filter((id) => loaded.some((item) => item.id === id)))
    return loaded
  }, [])

  const loadInstanceDetails = useCallback(async (instanceId: string) => {
    if (!instanceId) return
    const [loadedConfig, loadedMods] = await Promise.all([
      getInstanceConfig(instanceId),
      getInstanceMods(instanceId),
    ])
    setConfig({ ...defaultConfig, ...loadedConfig })
    setMods(loadedMods)
    setDirty(false)
  }, [])

  const refreshLogs = useCallback(async () => {
    setLogs(await queryLogs(500))
  }, [])

  useEffect(() => {
    const unsubscribe = subscribeGlobalSettings(setGlobalSettings)
    void Promise.all([
      loadGlobalSettingsFromBackend(),
      loadInstances(),
      refreshLogs(),
    ]).then(([settings]) => setGlobalSettings(settings)).catch((error) => {
      messageApi.error(`初始化管理器状态失败：${String(error)}`)
    })
    return unsubscribe
  }, [loadInstances, messageApi, refreshLogs])

  useEffect(() => {
    if (selectedId) void loadInstanceDetails(selectedId).catch((error) => {
      messageApi.error(`加载实例配置失败：${String(error)}`)
    })
  }, [loadInstanceDetails, messageApi, selectedId])

  useEffect(() => {
    let disposed = false
    const unlisteners: Array<() => void> = []

    void listen<LogLine>('asa:log-line', (event) => {
      if (!disposed) appendLogLine(event.payload)
    }).then((unlisten) => unlisteners.push(unlisten))

    void listen<ServerInstance>('asa:instance-status', (event) => {
      if (!disposed) replaceInstance(event.payload)
    }).then((unlisten) => unlisteners.push(unlisten))

    void listen<InstanceCreatedEvent>(ADD_INSTANCE_CREATED_EVENT, (event) => {
      if (disposed) return
      replaceInstance(event.payload.instance)
      setSelectedId(event.payload.instance.id)
      setSelectedRows([event.payload.instance.id])
      void refreshLogs()
      messageApi.success(`${event.payload.instance.name} 已添加`)
      if (event.payload.autoInstall) {
        void installInstance(event.payload.instance)
      }
    }).then((unlisten) => unlisteners.push(unlisten))

    return () => {
      disposed = true
      unlisteners.forEach((unlisten) => unlisten())
    }
  }, [appendLogLine, messageApi, refreshLogs, replaceInstance])

  const installInstance = async (item: ServerInstance) => {
    try {
      const updated = await installOrUpdateInstance(item.id, (progress) => {
        setJobProgress((current) => ({ ...current, [item.id]: progress }))
      })
      replaceInstance(updated)
      messageApi.success(`${item.name} 安装/更新完成`)
      await refreshLogs()
    } catch (error) {
      messageApi.error(`${item.name} 安装/更新失败：${String(error)}`)
    }
  }

  const startInstance = async (item: ServerInstance) => {
    try {
      const updated = await startInstanceCommand(item.id)
      replaceInstance(updated)
      messageApi.success(`${item.name} 已启动`)
      await refreshLogs()
    } catch (error) {
      messageApi.error(`${item.name} 启动失败：${String(error)}`)
    }
  }

  const stopInstance = (item: ServerInstance) => {
    if (item.status === 'stopped') {
      messageApi.info(`${item.name} 已停止`)
      return
    }
    modal.confirm({
      title: `停止 ${item.name}？`,
      icon: <ExclamationCircleFilled />,
      content: '管理器将优先通过 RCON 保存世界，再停止服务端进程。',
      okText: '保存并停止',
      cancelText: '取消',
      okButtonProps: { danger: true },
      onOk: async () => {
        const updated = await stopInstanceCommand(item.id)
        replaceInstance(updated)
        await refreshLogs()
        messageApi.success(`${item.name} 已停止`)
      },
    })
  }

  const saveConfig = async () => {
    if (!selected) return
    try {
      await saveInstanceConfig(selected.id, config, mods)
      setDirty(false)
      await refreshLogs()
      messageApi.success('实例配置已保存并写入文件')
    } catch (error) {
      messageApi.error(`保存实例配置失败：${String(error)}`)
    }
  }

  const applyConfig = () => {
    if (!selected) return
    modal.confirm({
      title: `保存并应用 ${selected.name}？`,
      icon: <ReloadOutlined className="confirm-blue-icon" />,
      content: selected.status === 'running' ? '运行中的实例会先保存世界并重启。' : '配置会写入真实 ARK 配置文件。',
      okText: selected.status === 'running' ? '保存并重启' : '保存并应用',
      cancelText: '取消',
      onOk: async () => {
        const updated = await applyInstanceConfig(selected.id, config, mods)
        replaceInstance(updated)
        setDirty(false)
        await refreshLogs()
        messageApi.success('配置已应用')
      },
    })
  }

  const updateConfig = <K extends keyof ServerConfig>(key: K, value: ServerConfig[K]) => {
    setConfig((current) => ({ ...current, [key]: value }))
    setDirty(true)
  }

  const handleModsChange = (next: ModItem[]) => {
    setMods(next)
    setDirty(true)
  }

  const handleCheckModUpdates = async () => {
    setCheckingMods(true)
    try {
      const checked = await checkModUpdates(mods)
      setMods(checked)
      setDirty(true)
      messageApi.success('MOD 列表已完成本地校验')
    } catch (error) {
      messageApi.error(`MOD 检查失败：${String(error)}`)
    } finally {
      setCheckingMods(false)
    }
  }

  const openAddInstanceWindow = async () => {
    try {
      const existing = await WebviewWindow.getByLabel(ADD_INSTANCE_WINDOW_LABEL)
      if (existing) {
        await existing.setFocus()
        return
      }

      const nextIndex = instances.length + 1
      const maxGamePort = instances.length ? Math.max(...instances.map((item) => item.gamePort)) : 7857
      const maxQueryPort = instances.length ? Math.max(...instances.map((item) => item.queryPort)) : 27095
      const maxRconPort = instances.length ? Math.max(...instances.map((item) => item.rconPort)) : 32339
      const params = new URLSearchParams({
        window: 'add-instance',
        index: String(nextIndex),
        gamePort: String(maxGamePort + 10),
        queryPort: String(maxQueryPort + 10),
        rconPort: String(maxRconPort + 1),
        serverRoot: globalSettings.serverStoragePath,
      })

      const webview = new WebviewWindow(ADD_INSTANCE_WINDOW_LABEL, {
        url: `/index.html?${params.toString()}`,
        title: '添加服务器实例',
        width: 760,
        height: 690,
        minWidth: 720,
        minHeight: 640,
        maxWidth: 760,
        maxHeight: 690,
        center: true,
        resizable: false,
        maximizable: false,
        parent: 'main',
        backgroundColor: '#020a13',
      })

      webview.once('tauri://error', (event) => {
        console.error('创建添加实例窗口失败', event)
        void WebviewWindow.getByLabel(ADD_INSTANCE_WINDOW_LABEL).then((window) => window?.setFocus())
      })
    } catch (error) {
      messageApi.error(`无法打开添加实例窗口：${String(error)}`)
    }
  }

  const openSettingsWindow = () => {
    const webview = new WebviewWindow('settings', {
      url: '/index.html?window=settings',
      title: '全局设置 (Global Settings)',
      width: 860,
      height: 660,
      minWidth: 720,
      minHeight: 560,
      center: true,
      resizable: true,
    })

    webview.once('tauri://error', (event) => {
      console.error('创建设置窗口失败', event)
      void WebviewWindow.getByLabel('settings').then((window) => window?.setFocus())
    })
  }

  const runForSelected = async (action: (item: ServerInstance) => Promise<void> | void) => {
    const selectedItems = instances.filter((item) => selectedRows.includes(item.id))
    for (const item of selectedItems) await action(item)
  }

  const createSelectedBackup = async () => {
    if (!selected) return
    try {
      const backup = await createBackup(selected.id)
      await refreshLogs()
      messageApi.success(`备份已创建：${backup.path}`)
    } catch (error) {
      messageApi.error(`创建备份失败：${String(error)}`)
    }
  }

  const exportSelected = async () => {
    try {
      const result = await exportInstanceConfig(selectedRows.map(String))
      messageApi.success(`已导出 ${result.exportedInstances} 个实例：${result.path}`)
    } catch (error) {
      messageApi.error(`导出实例失败：${String(error)}`)
    }
  }

  const exportAll = async () => {
    try {
      const result = await exportCluster()
      messageApi.success(`已导出整个集群：${result.path}`)
    } catch (error) {
      messageApi.error(`导出集群失败：${String(error)}`)
    }
  }

  const importConfig = async () => {
    try {
      const selectedPath = await open({
        title: '选择 ASA 实例导出文件',
        multiple: false,
        filters: [{ name: 'ASA 导出文件', extensions: ['json'] }],
      })
      if (!selectedPath) return
      const result = await importInstanceConfig(selectedPath)
      await loadInstances()
      messageApi.success(`已导入 ${result.importedInstances} 个实例，跳过 ${result.skippedInstances} 个重复实例`)
    } catch (error) {
      messageApi.error(`导入实例失败：${String(error)}`)
    }
  }

  const refreshSelectedStatus = async () => {
    try {
      if (selected) replaceInstance(await refreshInstanceStatus(selected.id))
      await loadInstances()
      await refreshLogs()
      messageApi.success('实例状态已刷新')
    } catch (error) {
      messageApi.error(`刷新状态失败：${String(error)}`)
    }
  }

  const batchMenu: MenuProps = {
    items: [
      { key: 'install', label: '安装/更新所选', icon: <CloudDownloadOutlined /> },
      { key: 'backup', label: '备份所选', icon: <DatabaseOutlined /> },
      { key: 'refresh', label: '刷新所选状态', icon: <ReloadOutlined /> },
    ],
    onClick: ({ key }) => {
      if (key === 'install') void runForSelected((item) => installInstance(item))
      if (key === 'backup') void runForSelected(async (item) => {
        await createBackup(item.id)
        await refreshLogs()
      }).then(() => messageApi.success('所选实例备份完成')).catch((error) => messageApi.error(`批量备份失败：${String(error)}`))
      if (key === 'refresh') void refreshSelectedStatus()
    },
  }

  const columns: ColumnsType<ServerInstance> = useMemo(() => [
    {
      title: '实例名称',
      dataIndex: 'name',
      width: 112,
      render: (name: string, item) => <div className="instance-name"><span className="instance-node"><span /><span /><span /></span><strong>{name}</strong>{item.id === selectedId && <span className="selected-dot" />}</div>,
    },
    { title: '地图', dataIndex: 'map', width: 118, render: (map: string) => <span className="map-name"><b>{mapGlyphs[map] ?? '◆'}</b>{map}</span> },
    { title: '模式', dataIndex: 'mode', width: 48 },
    {
      title: '状态',
      dataIndex: 'status',
      width: 86,
      render: (status: ServerStatus) => {
        const meta = statusMeta(status)
        return <Tag color={meta.color}>{meta.text}</Tag>
      },
    },
    { title: '端口 / 查询', width: 96, render: (_, item) => <span className="mono-text">{item.gamePort} / {item.queryPort}</span> },
    {
      title: '玩家数 / 上限',
      width: 108,
      render: (_, item) => <div className="player-cell"><span>{item.players} / {item.maxPlayers}</span><Progress percent={item.maxPlayers > 0 ? item.players / item.maxPlayers * 100 : 0} showInfo={false} size="small" strokeColor="#16cc79" railColor="#152838" /></div>,
    },
    {
      title: '操作',
      width: 108,
      render: (_, item) => (
        <Space.Compact>
          <Tooltip title="启动"><Button size="small" type="text" icon={<PlayCircleFilled />} disabled={item.status !== 'stopped' && item.status !== 'error'} onClick={(event) => { event.stopPropagation(); void startInstance(item) }} /></Tooltip>
          <Tooltip title="停止"><Button size="small" type="text" icon={<StopFilled />} danger={item.status === 'running'} disabled={item.status === 'stopped'} onClick={(event) => { event.stopPropagation(); stopInstance(item) }} /></Tooltip>
          <Dropdown
            menu={{
              items: [
                { key: 'install', label: '安装/更新', icon: <CloudDownloadOutlined /> },
                { key: 'backup', label: '创建备份', icon: <DatabaseOutlined /> },
                { key: 'folder', label: '打开目录', icon: <FolderOpenOutlined /> },
                { key: 'edit', label: '编辑实例', icon: <EditOutlined /> },
              ],
              onClick: ({ key }) => {
                if (key === 'install') void installInstance(item)
                if (key === 'backup') void createBackup(item.id).then((backup) => messageApi.success(`备份已创建：${backup.path}`)).catch((error) => messageApi.error(`创建备份失败：${String(error)}`))
                if (key === 'folder') void openInstanceDirectory(item.id).catch((error) => messageApi.error(`打开目录失败：${String(error)}`))
                if (key === 'edit') {
                  setSelectedId(item.id)
                  setSelectedRows([item.id])
                }
              },
            }}
            trigger={['click']}
          >
            <Button size="small" type="text" icon={<EllipsisOutlined />} onClick={(event) => event.stopPropagation()} />
          </Dropdown>
        </Space.Compact>
      ),
    },
  ], [messageApi, selectedId])

  const selectedProgress = selected ? jobProgress[selected.id] : undefined

  return (
    <div className="app-shell">
      {contextHolder}{modalContext}
      <header className="topbar">
        <Logo />
        <div className="topbar__actions">
          <Button icon={<PlayCircleFilled />} disabled={selectedRows.length === 0} onClick={() => void runForSelected((item) => startInstance(item))}>启动所选</Button>
          <Button danger icon={<StopFilled />} disabled={selectedRows.length === 0} onClick={() => void runForSelected((item) => stopInstance(item))}>停止所选</Button>
          <Button icon={<SaveOutlined />} disabled={!selected} onClick={() => void saveConfig()}>保存配置</Button>
          <Button className="apply-button" icon={<ReloadOutlined />} disabled={!selected} onClick={applyConfig}>应用并重启</Button>
          <Button icon={<SettingOutlined />} aria-label="全局设置" onClick={openSettingsWindow} />
        </div>
      </header>

      <main className="workspace">
        <SteamCmdSetup settings={globalSettings} onSettingsChange={setGlobalSettings} />
        {selectedProgress && selectedProgress.phase !== 'completed' && (
          <Progress percent={selectedProgress.percent ?? 0} status="active" strokeColor="#13b8ff" />
        )}
        <section className="stats-grid">
          <StatCard icon={<CloudServerOutlined />} label="总服务器数" value={instances.length} suffix="个实例" />
          <StatCard icon={<CheckCircleOutlined />} label="运行中" value={`${running} / ${instances.length}`} suffix="个实例" tone="green" />
          <StatCard icon={<DatabaseOutlined />} label="地图数量" value={new Set(instances.map((item) => item.map)).size} suffix="张地图" />
          <StatCard icon={<TeamOutlined />} label="总玩家数" value={`${totalPlayers} / ${playerCapacity}`} suffix="" />
        </section>

        <div className="main-grid">
          <div className="left-column">
            <section className="surface instance-list-card">
              <div className="surface__title">
                <span>服务器实例列表</span>
                <Space size={8}>
                  <Button size="small" icon={<PlusOutlined />} onClick={() => void openAddInstanceWindow()}>添加实例</Button>
                  <Dropdown menu={batchMenu} disabled={selectedRows.length === 0}>
                    <Button size="small" icon={<AppstoreAddOutlined />}>批量操作</Button>
                  </Dropdown>
                  <Button size="small" icon={<ReloadOutlined />} onClick={() => void refreshSelectedStatus()}>刷新列表</Button>
                </Space>
              </div>
              <Table
                rowKey="id"
                columns={columns}
                dataSource={instances}
                locale={{ emptyText: <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="尚未创建服务器实例" /> }}
                pagination={false}
                size="small"
                tableLayout="fixed"
                scroll={{ y: 360 }}
                rowSelection={{ selectedRowKeys: selectedRows, onChange: setSelectedRows, columnWidth: 36 }}
                onRow={(item) => ({ onClick: () => { setSelectedId(item.id); setSelectedRows([item.id]) } })}
                rowClassName={(item) => item.id === selectedId ? 'selected-instance-row' : ''}
              />
            </section>

            <section className="surface cluster-log-card">
              <div className="surface__title">
                <span><LineChartOutlined /> 集群日志 / 实例状态</span>
                <Space><Checkbox defaultChecked>自动滚动</Checkbox><Button size="small" onClick={() => void clearLogs().then(() => setLogs([])).catch((error) => messageApi.error(`清空日志失败：${String(error)}`))}>清空日志</Button></Space>
              </div>
              <div className="log-console">
                {logs.length === 0 ? <div className="empty-log">暂无日志</div> : logs.map((line) => (
                  <div className={`log-line log-line--${line.level}`} key={line.id}>
                    <span>[{line.time}]</span><b>[{line.instance}]</b><span>{line.message}</span>
                  </div>
                ))}
              </div>
            </section>
          </div>

          {selected ? (
            <ConfigPanel
              instance={selected}
              config={config}
              mods={mods}
              dirty={dirty}
              onConfigChange={updateConfig}
              onModsChange={handleModsChange}
              onSave={() => void saveConfig()}
              onApply={applyConfig}
              onCheckModUpdates={() => void handleCheckModUpdates()}
              checkingMods={checkingMods}
            />
          ) : (
            <section className="surface config-panel">
              <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="创建或导入服务器实例后即可编辑真实配置" />
            </section>
          )}
        </div>

        <section className="surface quick-actions">
          <div className="quick-actions__title">快捷操作</div>
          <div className="quick-actions__buttons">
            <Button icon={<ImportOutlined />} onClick={() => void importConfig()}>导入实例配置</Button>
            <Button className="green-button" icon={<ExportOutlined />} disabled={selectedRows.length === 0} onClick={() => void exportSelected()}>导出所选实例</Button>
            <Button className="gold-button" icon={<DatabaseOutlined />} disabled={instances.length === 0} onClick={() => void exportAll()}>导出整个集群</Button>
            <Button icon={<DatabaseOutlined />} disabled={!selected} onClick={() => void createSelectedBackup()}>创建所选备份</Button>
            <Button icon={<FolderOpenOutlined />} disabled={!selected} onClick={() => selected && void openInstanceDirectory(selected.id).catch((error) => messageApi.error(`打开目录失败：${String(error)}`))}>打开实例目录</Button>
          </div>
        </section>
      </main>

      <footer className="app-footer">
        <Text type="secondary">v0.1.0 Rust Backend</Text>
        <div><span>▣ 上次保存：{dirty ? '存在未保存修改' : '已同步'}</span><span>▧ 配置目录：{selected ? `${selected.installPath}\\ShooterGame\\Saved\\Config\\WindowsServer` : globalSettings.serverStoragePath}</span></div>
      </footer>
    </div>
  )
}
