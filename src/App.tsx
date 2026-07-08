import { Suspense, lazy, type Key, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  CheckCircleOutlined,
  CloudDownloadOutlined,
  CloudServerOutlined,
  DatabaseOutlined,
  ExclamationCircleFilled,
  FolderOpenOutlined,
  PlayCircleFilled,
  ReloadOutlined,
  SaveOutlined,
  SettingOutlined,
  StopFilled,
  TeamOutlined,
} from '@ant-design/icons'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { open } from '@tauri-apps/plugin-dialog'
import { Button, Empty, message, Modal, Space, Typography } from 'antd'
import type { MenuProps } from 'antd'
import { defaultConfig } from './data'
import {
  Logo,
  PanelLoading,
  StatCard,
  canDeleteInstance,
  enforceRequiredRconConfig,
} from './appShell'
import { ClusterLogCard, QuickActions } from './appDashboardPanels'
import {
  APPLICATION_LOG_TAB_KEY,
  InstanceLogPanes,
  LogConsole,
  LogTabLabel,
  SERVER_CONSOLE_LOG_KIND,
  SERVER_FILE_LOG_KIND,
  type ServerLogKind,
  getServerLogKind,
  matchesLogClearScope,
} from './appLogPanels'
import {
  mergeJobProgress,
  parseSteamCmdProgressLog,
} from './jobProgressView'
import InstanceTable from './InstanceTable'
import AppChildDialog, { type WebDialogState } from './AppChildDialog'
import {
  applyInstanceConfig,
  checkModUpdates,
  checkSteamCmd,
  clearStartupAutoUpdateSkipFlags,
  clearLogs,
  clearScopedLogs,
  createBackup,
  deleteInstance as deleteInstanceCommand,
  exportCluster,
  exportInstanceConfig,
  getInstanceConfig,
  getInstanceMods,
  importInstanceConfig,
  installOrUpdateInstance,
  listInstances,
  openDirectoryPath,
  openInstanceDirectory,
  queryLogs,
  refreshInstanceStatus,
  saveInstanceConfig,
  startInstance as startInstanceCommand,
  stopInstance as stopInstanceCommand,
} from './backendApi'
import { loadGlobalSettings, loadGlobalSettingsFromBackend, subscribeGlobalSettings } from './globalSettings'
import { isTauriRuntime } from './runtime'
import SteamCmdSetup from './SteamCmdSetup'
import { currentWindowBackgroundColor } from './themePreference'
import {
  ADD_INSTANCE_CREATED_EVENT,
  INSTANCE_CONFIG_CHANGED_EVENT,
  INSTANCE_DELETED_EVENT,
  INSTANCE_STATUS_EVENT,
  INSTANCES_CHANGED_EVENT,
  JOB_PROGRESS_EVENT,
  LOG_LINE_EVENT,
  LOGS_CLEARED_EVENT,
  LOGS_RESET_EVENT,
  subscribeBackendEvent,
} from './syncEvents'
import type {
  GlobalSettings,
  JobProgress,
  LogClearScope,
  LogLine,
  ModItem,
  ServerConfig,
  ServerInstance,
} from './types'
import { ADD_INSTANCE_WINDOW_LABEL, MAIN_WINDOW_LABEL, RCON_WINDOW_LABEL_PREFIX } from './windowEvents'

const { Text } = Typography
const ConfigPanel = lazy(() => import('./ConfigPanel'))
const PLAYER_STATUS_POLL_INTERVAL_MS = 5_000

export default function App() {
  const [instances, setInstances] = useState<ServerInstance[]>([])
  const [selectedId, setSelectedId] = useState('')
  const [selectedRows, setSelectedRows] = useState<Key[]>([])
  const [config, setConfig] = useState<ServerConfig>(defaultConfig)
  const [mods, setMods] = useState<ModItem[]>([])
  const [logs, setLogs] = useState<LogLine[]>([])
  const [dirty, setDirty] = useState(false)
  const [globalSettings, setGlobalSettings] = useState<GlobalSettings>(loadGlobalSettings)
  const childWindowBackgroundColor = currentWindowBackgroundColor(globalSettings.theme)
  const [initialDataReady, setInitialDataReady] = useState(false)
  const [jobProgress, setJobProgress] = useState<Record<string, JobProgress>>({})
  const [checkingMods, setCheckingMods] = useState(false)
  const [autoScrollLogs, setAutoScrollLogs] = useState(true)
  const [activeLogTab, setActiveLogTab] = useState(APPLICATION_LOG_TAB_KEY)
  const [webDialog, setWebDialog] = useState<WebDialogState>(null)
  const applicationLogConsoleRef = useRef<HTMLDivElement>(null)
  const serverConsoleLogConsoleRef = useRef<HTMLDivElement>(null)
  const serverFileLogConsoleRef = useRef<HTMLDivElement>(null)
  const instancesRef = useRef<ServerInstance[]>([])
  const statusPollInFlightRef = useRef(false)
  const startupAutoUpdateRanRef = useRef(false)
  const steamCmdProgressSamplesRef = useRef<Record<string, { downloadedBytes: number; timestamp: number }>>({})
  const [messageApi, contextHolder] = message.useMessage()
  const [modal, modalContext] = Modal.useModal()

  const selected = instances.find((item) => item.id === selectedId)
  const running = instances.filter((item) => item.status === 'running').length
  const totalPlayers = instances.reduce((sum, item) => sum + item.players, 0)
  const playerCapacity = instances.reduce((sum, item) => sum + item.maxPlayers, 0)
  const applicationLogs = useMemo(() => logs.filter((line) => line.source !== 'server'), [logs])
  const serverLogsByInstanceName = useMemo(() => {
    const grouped = new Map<string, { console: LogLine[]; file: LogLine[] }>()
    instances.forEach((item) => grouped.set(item.name, { console: [], file: [] }))
    logs.forEach((line) => {
      if (line.source !== 'server') return
      grouped.get(line.instance)?.[getServerLogKind(line)].push(line)
    })
    return grouped
  }, [instances, logs])
  const activeLogLineCount = useMemo(() => {
    if (activeLogTab === APPLICATION_LOG_TAB_KEY) return applicationLogs.length
    const activeInstance = instances.find((item) => item.id === activeLogTab)
    if (!activeInstance) return 0
    const activeLogs = serverLogsByInstanceName.get(activeInstance.name)
    return (activeLogs?.console.length ?? 0) + (activeLogs?.file.length ?? 0)
  }, [activeLogTab, applicationLogs.length, instances, serverLogsByInstanceName])
  const handleClearApplicationLogs = useCallback(async () => {
    const scope: LogClearScope = { source: 'application' }
    try {
      await clearScopedLogs(scope)
      setLogs((current) => current.filter((line) => !matchesLogClearScope(line, scope)))
      messageApi.success('已清除应用日志')
    } catch (error) {
      messageApi.error(`清除应用日志失败：${String(error)}`)
    }
  }, [messageApi])
  const handleClearServerLogs = useCallback(async (instanceName: string, serverLogKind: ServerLogKind) => {
    const kindText = serverLogKind === SERVER_FILE_LOG_KIND ? '游戏日志文件' : '服务端窗口日志'
    const scope: LogClearScope = { source: 'server', instance: instanceName, serverLogKind }
    try {
      await clearScopedLogs(scope)
      setLogs((current) => current.filter((line) => !matchesLogClearScope(line, scope)))
      messageApi.success(`已清除 ${instanceName} 的${kindText}`)
    } catch (error) {
      messageApi.error(`清除${kindText}失败：${String(error)}`)
    }
  }, [messageApi])
  const handleClearAllLogs = useCallback(async () => {
    try {
      await clearLogs()
      setLogs([])
      messageApi.success('已清除所有日志')
    } catch (error) {
      messageApi.error(`清除所有日志失败：${String(error)}`)
    }
  }, [messageApi])
  const logTabItems = useMemo(() => [
    {
      key: APPLICATION_LOG_TAB_KEY,
      label: <LogTabLabel title="应用日志" count={applicationLogs.length} />,
      children: (
        <LogConsole
          lines={applicationLogs}
          showInstance
          emptyText="暂无应用日志"
          consoleRef={activeLogTab === APPLICATION_LOG_TAB_KEY ? applicationLogConsoleRef : undefined}
          clearTitle="清除应用日志"
          onClear={handleClearApplicationLogs}
        />
      ),
    },
    ...instances.map((item) => {
      const instanceLogs = serverLogsByInstanceName.get(item.name) ?? { console: [], file: [] }
      const totalLogs = instanceLogs.console.length + instanceLogs.file.length
      return {
        key: item.id,
        label: <LogTabLabel title={item.name} count={totalLogs} />,
        children: (
          <InstanceLogPanes
            consoleLines={instanceLogs.console}
            fileLines={instanceLogs.file}
            consoleRef={activeLogTab === item.id ? serverConsoleLogConsoleRef : undefined}
            fileRef={activeLogTab === item.id ? serverFileLogConsoleRef : undefined}
            onClearConsole={() => handleClearServerLogs(item.name, SERVER_CONSOLE_LOG_KIND)}
            onClearFile={() => handleClearServerLogs(item.name, SERVER_FILE_LOG_KIND)}
          />
        ),
      }
    }),
  ], [activeLogTab, applicationLogs, handleClearApplicationLogs, handleClearServerLogs, instances, serverLogsByInstanceName])

  useEffect(() => {
    instancesRef.current = instances
  }, [instances])

  useEffect(() => {
    if (activeLogTab !== APPLICATION_LOG_TAB_KEY && !instances.some((item) => item.id === activeLogTab)) {
      setActiveLogTab(APPLICATION_LOG_TAB_KEY)
    }
  }, [activeLogTab, instances])

  const replaceInstance = useCallback((next: ServerInstance) => {
    setInstances((current) => {
      const updated = current.some((item) => item.id === next.id)
        ? current.map((item) => item.id === next.id ? next : item)
        : [...current, next]
      instancesRef.current = updated
      return updated
    })
  }, [])

  const pollLiveInstanceStatus = useCallback(async () => {
    if (statusPollInFlightRef.current) return

    const liveInstanceIds = instancesRef.current
      .filter((item) => item.status === 'running' || item.status === 'starting')
      .map((item) => item.id)
    if (liveInstanceIds.length === 0) return

    statusPollInFlightRef.current = true
    try {
      const results = await Promise.allSettled(liveInstanceIds.map((id) => refreshInstanceStatus(id)))
      results.forEach((result) => {
        if (result.status === 'fulfilled') replaceInstance(result.value)
      })
    } finally {
      statusPollInFlightRef.current = false
    }
  }, [replaceInstance])

  useEffect(() => {
    const timer = window.setInterval(() => {
      void pollLiveInstanceStatus()
    }, PLAYER_STATUS_POLL_INTERVAL_MS)
    return () => window.clearInterval(timer)
  }, [pollLiveInstanceStatus])

  const appendLogLine = useCallback((line: LogLine) => {
    setLogs((current) => [...current, line].slice(-500))
  }, [])

  const updateJobProgressFromLog = useCallback((line: LogLine) => {
    const parsed = parseSteamCmdProgressLog(line.message)
    if (!parsed) return

    const instance = instancesRef.current.find((item) => item.name === line.instance)
    if (!instance) return

    const now = Date.now()
    const previous = steamCmdProgressSamplesRef.current[instance.id]
    const elapsedSeconds = previous ? (now - previous.timestamp) / 1000 : 0
    const bytesPerSecond = previous && elapsedSeconds > 0 && parsed.downloadedBytes >= previous.downloadedBytes
      ? Math.round((parsed.downloadedBytes - previous.downloadedBytes) / elapsedSeconds)
      : 0

    steamCmdProgressSamplesRef.current[instance.id] = {
      downloadedBytes: parsed.downloadedBytes,
      timestamp: now,
    }

    setJobProgress((current) => {
      const next: JobProgress = {
        jobId: `steamcmd-log-${line.id}`,
        instanceId: instance.id,
        phase: parsed.phase,
        percent: parsed.percent,
        message: 'SteamCMD progress',
        detail: line.message,
        downloadedBytes: parsed.downloadedBytes,
        totalBytes: parsed.totalBytes,
        bytesPerSecond,
      }
      return {
        ...current,
        [instance.id]: mergeJobProgress(current[instance.id], next),
      }
    })
  }, [])

  const loadInstances = useCallback(async () => {
    const loaded = await listInstances()
    instancesRef.current = loaded
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
    setConfig(enforceRequiredRconConfig({ ...defaultConfig, ...loadedConfig }))
    setMods(loadedMods)
    setDirty(false)
  }, [])

  const refreshLogs = useCallback(async () => {
    const loadedLogs = await queryLogs(500)
    setLogs(loadedLogs)
    loadedLogs.forEach(updateJobProgressFromLog)
  }, [updateJobProgressFromLog])

  useEffect(() => {
    const unsubscribe = subscribeGlobalSettings(setGlobalSettings)
    void Promise.all([
      loadGlobalSettingsFromBackend(),
      loadInstances(),
      refreshLogs(),
    ]).then(([settings]) => setGlobalSettings(settings)).catch((error) => {
      messageApi.error(`初始化管理器状态失败：${String(error)}`)
    }).finally(() => {
      setInitialDataReady(true)
    })
    return unsubscribe
  }, [loadInstances, messageApi, refreshLogs])

  useEffect(() => {
    if (selectedId) void loadInstanceDetails(selectedId).catch((error) => {
      messageApi.error(`加载实例配置失败：${String(error)}`)
    })
  }, [loadInstanceDetails, messageApi, selectedId])

  useEffect(() => {
    if (!autoScrollLogs) return
    const consoleElements = activeLogTab === APPLICATION_LOG_TAB_KEY
      ? [applicationLogConsoleRef.current]
      : [serverConsoleLogConsoleRef.current, serverFileLogConsoleRef.current]
    consoleElements.forEach((consoleElement) => {
      if (consoleElement) {
        consoleElement.scrollTop = consoleElement.scrollHeight
      }
    })
  }, [activeLogLineCount, activeLogTab, autoScrollLogs])

  useEffect(() => {
    let disposed = false
    const unlisteners: Array<() => void> = []

    unlisteners.push(subscribeBackendEvent(LOG_LINE_EVENT, (line) => {
      if (!disposed) {
        appendLogLine(line)
        updateJobProgressFromLog(line)
      }
    }))

    unlisteners.push(subscribeBackendEvent(LOGS_CLEARED_EVENT, (scope) => {
      if (!disposed) {
        setLogs((current) => current.filter((line) => !matchesLogClearScope(line, scope)))
      }
    }))

    unlisteners.push(subscribeBackendEvent(LOGS_RESET_EVENT, () => {
      if (!disposed) setLogs([])
    }))

    unlisteners.push(subscribeBackendEvent(INSTANCE_STATUS_EVENT, (instance) => {
      if (!disposed) replaceInstance(instance)
    }))

    unlisteners.push(subscribeBackendEvent(JOB_PROGRESS_EVENT, (progress) => {
      if (disposed || !progress.instanceId) return
      setJobProgress((current) => ({
        ...current,
        [progress.instanceId as string]: mergeJobProgress(current[progress.instanceId as string], progress),
      }))
    }))

    unlisteners.push(subscribeBackendEvent(ADD_INSTANCE_CREATED_EVENT, (eventPayload) => {
      if (disposed) return
      replaceInstance(eventPayload.instance)
      setSelectedId(eventPayload.instance.id)
      setSelectedRows([eventPayload.instance.id])
      void loadInstanceDetails(eventPayload.instance.id).catch((error) => {
        messageApi.error(`加载新增实例配置失败：${String(error)}`)
      })
      void refreshLogs()
      messageApi.success(`${eventPayload.instance.name} 已添加`)
      if (eventPayload.autoInstall) {
        void installInstance(eventPayload.instance)
      }
    }))

    unlisteners.push(subscribeBackendEvent(INSTANCE_CONFIG_CHANGED_EVENT, (payload) => {
      if (disposed) return
      replaceInstance(payload.instance)
      if (payload.instanceId !== selectedId) return
      if (dirty) {
        messageApi.info('检测到另一端已更新当前实例配置；已保留你本地尚未保存的编辑')
        return
      }
      setConfig(enforceRequiredRconConfig({ ...defaultConfig, ...payload.config }))
      setMods(payload.mods)
      setDirty(false)
    }))

    unlisteners.push(subscribeBackendEvent(INSTANCE_DELETED_EVENT, (removed) => {
      if (disposed) return
      setInstances((current) => {
        const updated = current.filter((item) => item.id !== removed.id)
        instancesRef.current = updated
        return updated
      })
      setSelectedRows((current) => current.filter((id) => id !== removed.id))
      setJobProgress((current) => {
        const next = { ...current }
        delete next[removed.id]
        return next
      })
      if (selectedId === removed.id) {
        setSelectedId('')
        setConfig(defaultConfig)
        setMods([])
        setDirty(false)
      }
    }))

    unlisteners.push(subscribeBackendEvent(INSTANCES_CHANGED_EVENT, () => {
      if (!disposed) void loadInstances().catch((error) => console.error('同步实例列表失败', error))
    }))

    return () => {
      disposed = true
      unlisteners.forEach((unlisten) => unlisten())
    }
  }, [appendLogLine, dirty, loadInstanceDetails, loadInstances, messageApi, refreshLogs, replaceInstance, selectedId, updateJobProgressFromLog])

  useEffect(() => {
    if (isTauriRuntime()) return undefined

    const timer = window.setInterval(() => {
      void Promise.all([loadInstances(), refreshLogs()]).catch((error) => {
        console.error('刷新 Web 版状态失败', error)
      })
    }, 3_000)

    return () => window.clearInterval(timer)
  }, [loadInstances, refreshLogs])

  const installInstance = async (item: ServerInstance) => {
    try {
      const updated = await installOrUpdateInstance(item.id, (progress) => {
        setJobProgress((current) => ({
          ...current,
          [item.id]: mergeJobProgress(current[item.id], progress),
        }))
      })
      replaceInstance(updated)
      messageApi.success(`${item.name} 安装/更新完成`)
      await refreshLogs()
    } catch (error) {
      const errorText = String(error)
      if (errorText.includes('取消')) {
        setJobProgress((current) => ({
          ...current,
          [item.id]: {
            jobId: `cancelled-${Date.now()}`,
            instanceId: item.id,
            phase: 'cancelled',
            percent: 0,
            message: '安装/更新已取消',
            detail: null,
            downloadedBytes: 0,
            totalBytes: null,
            bytesPerSecond: 0,
          },
        }))
        messageApi.info(`${item.name} 安装/更新已取消`)
      } else {
        setJobProgress((current) => ({
          ...current,
          [item.id]: {
            jobId: `failed-${Date.now()}`,
            instanceId: item.id,
            phase: 'failed',
            percent: 0,
            message: errorText,
            detail: errorText,
            downloadedBytes: 0,
            totalBytes: null,
            bytesPerSecond: 0,
          },
        }))
        messageApi.error(`${item.name} 安装/更新失败：${errorText}`)
      }
      await refreshLogs()
    }
  }

  useEffect(() => {
    if (!initialDataReady || startupAutoUpdateRanRef.current) return
    startupAutoUpdateRanRef.current = true
    if (!globalSettings.autoUpdateOnStart) return

    void (async () => {
      const steamCmd = await checkSteamCmd(globalSettings.steamCmdPath)
      if (!steamCmd.valid) {
        messageApi.warning('启动时检查更新已跳过：SteamCMD 尚未配置或不可用')
        return
      }

      const candidates: ServerInstance[] = []
      const skippedRecovered: ServerInstance[] = []
      for (const item of instancesRef.current) {
        if (!['stopped', 'error'].includes(item.status)) continue
        if (item.skipAutoUpdateOnStartOnce) {
          skippedRecovered.push(item)
          continue
        }
        try {
          const instanceConfig = await getInstanceConfig(item.id)
          if (instanceConfig.autoUpdateServer ?? true) {
            candidates.push(item)
          }
        } catch (error) {
          messageApi.warning(`${item.name} 启动时检查更新配置读取失败：${String(error)}`)
        }
      }

      if (skippedRecovered.length > 0) {
        const names = skippedRecovered.map((item) => item.name).join('、')
        messageApi.warning(`已跳过 ${names} 的启动自动更新：该实例是从上次中断的安装/更新状态恢复而来，请确认后手动执行安装/更新`)
        try {
          const updated = await clearStartupAutoUpdateSkipFlags(skippedRecovered.map((item) => item.id))
          updated.forEach(replaceInstance)
        } catch (error) {
          messageApi.warning(`清除启动自动更新跳过标记失败：${String(error)}`)
        }
      }

      if (candidates.length === 0) return
      messageApi.info(`启动时检查更新：将依次检查/更新 ${candidates.length} 个启用自动更新的实例`)
      for (const item of candidates) {
        await installInstance(item)
      }
    })().catch((error) => {
      messageApi.error(`启动时检查更新失败：${String(error)}`)
    })
  }, [globalSettings.autoUpdateOnStart, globalSettings.steamCmdPath, initialDataReady, messageApi, replaceInstance])

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
    if (item.status === 'stopping') {
      messageApi.info(`${item.name} 正在后台停止`)
      return
    }
    const isUpdating = item.status === 'updating'
    modal.confirm({
      title: isUpdating ? `取消 ${item.name} 的安装/更新？` : `停止 ${item.name}？`,
      icon: <ExclamationCircleFilled />,
      content: isUpdating
        ? '管理器将结束当前 SteamCMD 更新进程树，并把实例状态恢复为已停止。'
        : '管理器将优先通过 RCON 保存世界，再停止服务端进程。',
      okText: isUpdating ? '取消更新' : '保存并停止',
      cancelText: '取消',
      okButtonProps: { danger: true },
      onOk: async () => {
        try {
          const updated = await stopInstanceCommand(item.id)
          replaceInstance(updated)
          if (isUpdating) {
            setJobProgress((current) => ({
              ...current,
              [item.id]: {
                jobId: `cancelled-${Date.now()}`,
                instanceId: item.id,
                phase: 'cancelled',
                percent: 0,
                message: '安装/更新取消请求已发送',
                detail: null,
                downloadedBytes: 0,
                totalBytes: null,
                bytesPerSecond: 0,
              },
            }))
          }
          await refreshLogs()
          messageApi.success(isUpdating ? `${item.name} 安装/更新取消请求已发送` : `${item.name} 停止请求已发送，后台继续处理`)
        } catch (error) {
          messageApi.error(`${item.name} 停止请求失败：${String(error)}`)
        }
      },
    })
  }

  const saveConfig = async () => {
    if (!selected) return
    const nextConfig = enforceRequiredRconConfig(config)
    if (!nextConfig.adminPassword) {
      setConfig(nextConfig)
      messageApi.warning('请先设置管理员密码，RCON 必须保持启用')
      return
    }
    try {
      const updated = await saveInstanceConfig(selected.id, nextConfig, mods)
      replaceInstance(updated)
      setConfig(nextConfig)
      setDirty(false)
      await refreshLogs()
      messageApi.success('实例配置已保存并写入文件')
    } catch (error) {
      messageApi.error(`保存实例配置失败：${String(error)}`)
    }
  }

  const applyConfig = () => {
    if (!selected) return
    const nextConfig = enforceRequiredRconConfig(config)
    if (!nextConfig.adminPassword) {
      setConfig(nextConfig)
      messageApi.warning('请先设置管理员密码，RCON 必须保持启用')
      return
    }
    modal.confirm({
      title: `保存并应用 ${selected.name}？`,
      icon: <ReloadOutlined className="confirm-blue-icon" />,
      content: selected.status === 'running' ? '运行中的实例会先保存世界并重启。' : '配置会写入真实 ARK 配置文件，然后启动实例。',
      okText: '保存并重启',
      cancelText: '取消',
      onOk: async () => {
        try {
          const updated = await applyInstanceConfig(selected.id, nextConfig, mods)
          replaceInstance(updated)
          setConfig(nextConfig)
          setDirty(false)
          await refreshLogs()
          messageApi.success('配置已应用并已请求重启')
        } catch (error) {
          messageApi.error(`应用并重启失败：${String(error)}`)
          throw error
        }
      },
    })
  }

  const updateConfig = <K extends keyof ServerConfig>(key: K, value: ServerConfig[K]) => {
    setConfig((current) => enforceRequiredRconConfig({ ...current, [key]: key === 'rconEnabled' ? true : value }))
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
      const nextIndex = instances.length + 1
      const lastInstance = instances.at(-1)
      const params = new URLSearchParams({
        window: 'add-instance',
        index: String(nextIndex),
        gamePort: String((lastInstance?.gamePort ?? 7857) + 10),
        queryPort: String((lastInstance?.queryPort ?? 27095) + 10),
        rconPort: String((lastInstance?.rconPort ?? 32330) + 10),
        serverRoot: globalSettings.serverStoragePath,
      })

      if (!isTauriRuntime()) {
        setWebDialog({ type: 'add-instance', params })
        return
      }

      const existing = await WebviewWindow.getByLabel(ADD_INSTANCE_WINDOW_LABEL)
      if (existing) {
        await existing.setFocus()
        return
      }

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
        parent: MAIN_WINDOW_LABEL,
        backgroundColor: childWindowBackgroundColor,
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
    if (!isTauriRuntime()) {
      setWebDialog({ type: 'settings' })
      return
    }

    const webview = new WebviewWindow('settings', {
      url: '/index.html?window=settings',
      title: '全局设置 (Global Settings)',
      width: 860,
      height: 660,
      minWidth: 720,
      minHeight: 560,
      center: true,
      resizable: true,
      parent: MAIN_WINDOW_LABEL,
      backgroundColor: childWindowBackgroundColor,
    })

    webview.once('tauri://error', (event) => {
      console.error('创建设置窗口失败', event)
      void WebviewWindow.getByLabel('settings').then((window) => window?.setFocus())
    })
  }

  const openRconWindow = useCallback(async (item: ServerInstance) => {
    if (!isTauriRuntime()) {
      setWebDialog({ type: 'rcon', instance: item })
      return
    }

    const label = `${RCON_WINDOW_LABEL_PREFIX}-${item.id}`
    try {
      const existing = await WebviewWindow.getByLabel(label)
      if (existing) {
        await existing.setFocus()
        return
      }

      const params = new URLSearchParams({
        window: 'rcon',
        instanceId: item.id,
        name: item.name,
        rconPort: String(item.rconPort),
      })

      const webview = new WebviewWindow(label, {
        url: `/index.html?${params.toString()}`,
        title: `${item.name} RCON管理`,
        width: 1080,
        height: 720,
        minWidth: 900,
        minHeight: 620,
        center: true,
        resizable: true,
        parent: MAIN_WINDOW_LABEL,
        backgroundColor: childWindowBackgroundColor,
      })

      webview.once('tauri://error', (event) => {
        console.error('创建 RCON 管理窗口失败', event)
        void WebviewWindow.getByLabel(label).then((window) => window?.setFocus())
      })
    } catch (error) {
      messageApi.error(`无法打开 RCON 管理窗口：${String(error)}`)
    }
  }, [childWindowBackgroundColor, messageApi])

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
      if (!isTauriRuntime()) {
        messageApi.info('Web 版无法打开系统文件选择器，请在桌面端导入实例配置')
        return
      }
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

  const askOpenDeletedInstanceDirectory = useCallback((item: ServerInstance) => {
    modal.confirm({
      title: '是否打开保留的实例文件夹？',
      icon: <FolderOpenOutlined className="confirm-blue-icon" />,
      content: (
        <Space direction="vertical" size={8}>
          <Text>实例记录已删除，服务器文件仍保留在原目录。</Text>
          <Text code copyable={{ text: item.installPath }}>{item.installPath}</Text>
        </Space>
      ),
      okText: '打开文件夹',
      cancelText: '稍后手动处理',
      onOk: async () => {
        try {
          await openDirectoryPath(item.installPath)
        } catch (error) {
          messageApi.error(`打开文件夹失败：${String(error)}`)
        }
      },
    })
  }, [messageApi, modal])

  const deleteInstanceRecord = useCallback((item: ServerInstance) => {
    if (!canDeleteInstance(item.status)) {
      messageApi.warning(`${item.name} 当前不是已停止状态，请先停止实例后再删除`)
      return
    }

    modal.confirm({
      title: `删除 ${item.name} 的实例记录？`,
      icon: <ExclamationCircleFilled />,
      content: (
        <Space direction="vertical" size={8}>
          <Text>此操作只会从管理器列表中删除实例记录，不会删除实例目录、存档、配置或备份。</Text>
          <Text>如果需要彻底删除服务器文件，请在删除实例记录后手动删除以下文件夹：</Text>
          <Text code copyable={{ text: item.installPath }}>{item.installPath}</Text>
        </Space>
      ),
      okText: '删除记录',
      cancelText: '取消',
      okButtonProps: { danger: true },
      onOk: async () => {
        try {
          const removed = await deleteInstanceCommand(item.id)
          await loadInstances()
          setSelectedRows((current) => current.filter((id) => id !== removed.id))
          setJobProgress((current) => {
            const next = { ...current }
            delete next[removed.id]
            return next
          })
          if (selectedId === removed.id) {
            setConfig(defaultConfig)
            setMods([])
            setDirty(false)
          }
          await refreshLogs()
          messageApi.success(`${removed.name} 已从管理器删除，服务器文件未删除`)
          askOpenDeletedInstanceDirectory(removed)
        } catch (error) {
          messageApi.error(`删除实例失败：${String(error)}`)
        }
      },
    })
  }, [askOpenDeletedInstanceDirectory, loadInstances, messageApi, modal, refreshLogs, selectedId])

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

  const selectInstanceInTable = useCallback((id: string) => {
    setSelectedId(id)
    setSelectedRows([id])
  }, [])

  const createInstanceBackupFromTable = useCallback((item: ServerInstance) => {
    void createBackup(item.id)
      .then((backup) => messageApi.success(`备份已创建：${backup.path}`))
      .catch((error) => messageApi.error(`创建备份失败：${String(error)}`))
  }, [messageApi])

  const openInstanceDirectoryFromTable = useCallback((item: ServerInstance) => {
    void openInstanceDirectory(item.id)
      .catch((error) => messageApi.error(`打开目录失败：${String(error)}`))
  }, [messageApi])

  const closeWebDialog = () => setWebDialog(null)

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
        <SteamCmdSetup
          settings={globalSettings}
          settingsReady={initialDataReady}
          onSettingsChange={setGlobalSettings}
        />
        <section className="stats-grid">
          <StatCard icon={<CloudServerOutlined />} label="总服务器数" value={instances.length} suffix="个实例" />
          <StatCard icon={<CheckCircleOutlined />} label="运行中" value={`${running} / ${instances.length}`} suffix="个实例" tone="green" />
          <StatCard icon={<DatabaseOutlined />} label="地图数量" value={new Set(instances.map((item) => item.map)).size} suffix="张地图" />
          <StatCard icon={<TeamOutlined />} label="总玩家数" value={`${totalPlayers} / ${playerCapacity}`} suffix="" />
        </section>

        <div className="main-grid">
          <div className="left-column">
            <InstanceTable
              instances={instances}
              selectedId={selectedId}
              selectedRows={selectedRows}
              jobProgress={jobProgress}
              batchMenu={batchMenu}
              onAddInstance={() => void openAddInstanceWindow()}
              onRefreshStatus={() => void refreshSelectedStatus()}
              onSelectedRowsChange={setSelectedRows}
              onSelectInstance={selectInstanceInTable}
              onStartInstance={(item) => void startInstance(item)}
              onStopInstance={stopInstance}
              onInstallInstance={(item) => void installInstance(item)}
              onCreateBackup={createInstanceBackupFromTable}
              onOpenDirectory={openInstanceDirectoryFromTable}
              onOpenRcon={(item) => void openRconWindow(item)}
              onDeleteInstance={deleteInstanceRecord}
            />

            <ClusterLogCard
              activeLogTab={activeLogTab}
              autoScrollLogs={autoScrollLogs}
              items={logTabItems}
              onActiveLogTabChange={setActiveLogTab}
              onAutoScrollLogsChange={setAutoScrollLogs}
              onClearAllLogs={() => void handleClearAllLogs()}
            />
          </div>

          {selected ? (
            <Suspense fallback={<PanelLoading />}>
              <ConfigPanel
                instance={selected}
                config={config}
                mods={mods}
                dirty={dirty}
                language={globalSettings.language}
                onConfigChange={updateConfig}
                onModsChange={handleModsChange}
                onSave={() => void saveConfig()}
                onApply={applyConfig}
                onCheckModUpdates={() => void handleCheckModUpdates()}
                checkingMods={checkingMods}
              />
            </Suspense>
          ) : (
            <section className="surface config-panel">
              <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="创建或导入服务器实例后即可编辑真实配置" />
            </section>
          )}
        </div>

        <QuickActions
          instancesCount={instances.length}
          selected={selected}
          selectedRowsCount={selectedRows.length}
          onImportConfig={() => void importConfig()}
          onExportSelected={() => void exportSelected()}
          onExportAll={() => void exportAll()}
          onCreateSelectedBackup={() => void createSelectedBackup()}
          onOpenSelectedDirectory={() => selected && void openInstanceDirectory(selected.id).catch((error) => messageApi.error(`打开目录失败：${String(error)}`))}
          onDeleteSelected={() => selected && deleteInstanceRecord(selected)}
        />
      </main>

      <AppChildDialog dialog={webDialog} onClose={closeWebDialog} />

      <footer className="app-footer">
        <Text type="secondary">v0.1.0 Rust Backend</Text>
        <div><span>▣ 上次保存：{dirty ? '存在未保存修改' : '已同步'}</span><span>▧ 配置目录：{selected ? `${selected.installPath}\\ShooterGame\\Saved\\Config\\WindowsServer` : globalSettings.serverStoragePath}</span></div>
      </footer>
    </div>
  )
}
