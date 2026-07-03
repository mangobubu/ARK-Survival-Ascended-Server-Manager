import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  AppstoreAddOutlined,
  CheckCircleOutlined,
  CloudDownloadOutlined,
  CloudServerOutlined,
  DatabaseOutlined,
  DeleteOutlined,
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
import { Button, Checkbox, Dropdown, Empty, message, Modal, Progress, Space, Table, Tabs, Tag, Tooltip, Typography } from 'antd'
import type { ColumnsType } from 'antd/es/table'
import type { MenuProps } from 'antd'
import ConfigPanel from './ConfigPanel'
import { defaultConfig } from './data'
import {
  applyInstanceConfig,
  checkModUpdates,
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
const APPLICATION_LOG_TAB_KEY = 'application'
const SERVER_CONSOLE_LOG_KIND = 'console'
const SERVER_FILE_LOG_KIND = 'file'
type ServerLogKind = NonNullable<LogLine['serverLogKind']>

function getServerLogKind(line: LogLine): ServerLogKind {
  return line.serverLogKind ?? SERVER_CONSOLE_LOG_KIND
}

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
  if (status === 'stopping') return { color: 'processing', text: '◍ 停止中' }
  if (status === 'starting') return { color: 'processing', text: '◌ 启动中' }
  if (status === 'updating') return { color: 'processing', text: '↻ 更新中' }
  if (status === 'backingUp') return { color: 'processing', text: '▣ 备份中' }
  if (status === 'error') return { color: 'error', text: '⊗ 异常' }
  return { color: 'default', text: '⊖ 已停止' }
}

function canDeleteInstance(status: ServerStatus) {
  return status === 'stopped' || status === 'error'
}

function enforceRequiredRconConfig(config: ServerConfig): ServerConfig {
  return {
    ...config,
    rconEnabled: true,
    adminPassword: config.adminPassword.trim(),
  }
}

function isActiveJobProgress(progress?: JobProgress) {
  return Boolean(progress && !['completed', 'cancelled', 'failed'].includes(progress.phase))
}

function formatJobBytes(value: number | null | undefined) {
  if (!Number.isFinite(value) || !value || value <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = value
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex += 1
  }
  const digits = size >= 100 || unitIndex === 0 ? 0 : size >= 10 ? 1 : 2
  return `${size.toFixed(digits)} ${units[unitIndex]}`
}

function phaseText(phase: string) {
  const labels: Record<string, string> = {
    preparing: '准备中',
    running: '下载/更新',
    downloading: '下载/更新',
    verifying: '校验中',
    preallocating: '预分配',
    committing: '写入中',
  }
  return labels[phase] ?? phase
}

function parseSteamCmdProgressLog(message: string) {
  const match = message.match(/progress\s*:\s*([\d.]+)\s*\(\s*([\d,]+)\s*\/\s*([\d,]+)\s*\)/i)
  if (!match) return null
  const percent = Number.parseFloat(match[1])
  const downloadedBytes = Number.parseInt(match[2].replaceAll(',', ''), 10)
  const totalBytes = Number.parseInt(match[3].replaceAll(',', ''), 10)
  if (!Number.isFinite(percent) || !Number.isFinite(downloadedBytes)) return null

  const lowerMessage = message.toLowerCase()
  const phase = lowerMessage.includes('download')
    ? 'downloading'
    : lowerMessage.includes('validat') || lowerMessage.includes('verify')
      ? 'verifying'
      : lowerMessage.includes('prealloc')
        ? 'preallocating'
        : lowerMessage.includes('commit')
          ? 'committing'
          : 'running'

  return {
    phase,
    percent,
    downloadedBytes,
    totalBytes: totalBytes > 0 ? totalBytes : null,
  }
}

function InstanceJobProgress({ progress }: { progress?: JobProgress }) {
  if (!progress) return null
  const totalKnown = Number.isFinite(progress.totalBytes) && (progress.totalBytes ?? 0) > 0
  const downloadedBytes = progress.downloadedBytes ?? 0
  const bytesPerSecond = progress.bytesPerSecond ?? 0
  const hasTransferInfo = downloadedBytes > 0 || totalKnown
  const percent = progress.percent != null ? `${Math.max(0, Math.min(100, progress.percent)).toFixed(1)}%` : '--'
  const speedText = bytesPerSecond > 0 ? `${formatJobBytes(bytesPerSecond)}/s` : hasTransferInfo ? '0 B/s' : '--'
  const downloadedText = downloadedBytes > 0 ? formatJobBytes(downloadedBytes) : '--'
  const totalText = totalKnown ? formatJobBytes(progress.totalBytes) : '--'

  return (
    <div className="instance-job-progress" onClick={(event) => event.stopPropagation()}>
      <div className="instance-job-progress__line">
        <span className="instance-job-progress__phase">{phaseText(progress.phase)}</span>
        <span>进度 <b>{percent}</b></span>
        <span>速度 <b>{speedText}</b></span>
        <span>已下载 <b>{downloadedText}</b></span>
        <span>总大小 <b>{totalText}</b></span>
      </div>
    </div>
  )
}

function LogTabLabel({ title, count }: { title: string; count: number }) {
  return (
    <span className="log-tab-label" title={title}>
      <span>{title}</span>
      <small>{count}</small>
    </span>
  )
}

function LogConsole({
  title,
  lines,
  showInstance,
  emptyText,
  consoleRef,
  clearTitle,
  onClear,
}: {
  title?: string
  lines: LogLine[]
  showInstance: boolean
  emptyText: string
  consoleRef?: React.RefObject<HTMLDivElement | null>
  clearTitle: string
  onClear: () => void | Promise<void>
}) {
  const clearButton = (
    <Tooltip title={clearTitle}>
      <Button
        className="log-console__clear"
        type="text"
        size="small"
        icon={<DeleteOutlined />}
        aria-label={clearTitle}
        disabled={lines.length === 0}
        onClick={(event) => {
          event.stopPropagation()
          void onClear()
        }}
      />
    </Tooltip>
  )

  return (
    <div className={`log-console-panel${title ? ' log-console-panel--titled' : ''}`}>
      {title ? (
        <div className="log-console-panel__title">
          <span>{title}</span>
          <small>{lines.length}</small>
          {clearButton}
        </div>
      ) : clearButton}
      <div className={`log-console${showInstance ? '' : ' log-console--server'}`} ref={consoleRef}>
        {lines.length === 0 ? <div className="empty-log">{emptyText}</div> : lines.map((line) => (
          <div className={`log-line log-line--${line.level}`} key={line.id}>
            <span>[{line.time}]</span>{showInstance && <b>[{line.instance}]</b>}<span>{line.message}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function InstanceLogPanes({
  consoleLines,
  fileLines,
  consoleRef,
  fileRef,
  onClearConsole,
  onClearFile,
}: {
  consoleLines: LogLine[]
  fileLines: LogLine[]
  consoleRef?: React.RefObject<HTMLDivElement | null>
  fileRef?: React.RefObject<HTMLDivElement | null>
  onClearConsole: () => void | Promise<void>
  onClearFile: () => void | Promise<void>
}) {
  return (
    <div className="instance-log-panes">
      <LogConsole
        title="服务端窗口日志"
        lines={consoleLines}
        showInstance={false}
        emptyText="暂无服务端窗口日志"
        consoleRef={consoleRef}
        clearTitle="清除服务端窗口日志"
        onClear={onClearConsole}
      />
      <LogConsole
        title="游戏日志文件"
        lines={fileLines}
        showInstance={false}
        emptyText="暂无游戏日志文件"
        consoleRef={fileRef}
        clearTitle="清除游戏日志文件"
        onClear={onClearFile}
      />
    </div>
  )
}

function mergeJobProgress(previous: JobProgress | undefined, next: JobProgress): JobProgress {
  const previousHasBytes = (previous?.downloadedBytes ?? 0) > 0 || (previous?.totalBytes ?? 0) > 0
  const nextHasBytes = (next.downloadedBytes ?? 0) > 0 || (next.totalBytes ?? 0) > 0

  if (previous && previousHasBytes && !nextHasBytes) {
    return {
      ...previous,
      jobId: next.jobId,
      phase: next.phase,
      message: next.message,
      detail: next.detail ?? previous.detail,
    }
  }

  return {
    ...next,
    percent: next.percent ?? previous?.percent ?? null,
    downloadedBytes: next.downloadedBytes > 0 ? next.downloadedBytes : previous?.downloadedBytes ?? 0,
    totalBytes: next.totalBytes && next.totalBytes > 0 ? next.totalBytes : previous?.totalBytes ?? null,
    bytesPerSecond: nextHasBytes ? Math.max(0, next.bytesPerSecond) : previous?.bytesPerSecond ?? 0,
  }
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
  const [autoScrollLogs, setAutoScrollLogs] = useState(true)
  const [activeLogTab, setActiveLogTab] = useState(APPLICATION_LOG_TAB_KEY)
  const applicationLogConsoleRef = useRef<HTMLDivElement>(null)
  const serverConsoleLogConsoleRef = useRef<HTMLDivElement>(null)
  const serverFileLogConsoleRef = useRef<HTMLDivElement>(null)
  const instancesRef = useRef<ServerInstance[]>([])
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
    try {
      await clearScopedLogs({ source: 'application' })
      setLogs((current) => current.filter((line) => line.source !== 'application'))
      messageApi.success('已清除应用日志')
    } catch (error) {
      messageApi.error(`清除应用日志失败：${String(error)}`)
    }
  }, [messageApi])
  const handleClearServerLogs = useCallback(async (instanceName: string, serverLogKind: ServerLogKind) => {
    const kindText = serverLogKind === SERVER_FILE_LOG_KIND ? '游戏日志文件' : '服务端窗口日志'
    try {
      await clearScopedLogs({ source: 'server', instance: instanceName, serverLogKind })
      setLogs((current) => current.filter((line) => (
        line.source !== 'server'
        || line.instance !== instanceName
        || getServerLogKind(line) !== serverLogKind
      )))
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
    setInstances((current) => current.some((item) => item.id === next.id)
      ? current.map((item) => item.id === next.id ? next : item)
      : [...current, next])
  }, [])

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

    void listen<LogLine>('asa:log-line', (event) => {
      if (!disposed) {
        appendLogLine(event.payload)
        updateJobProgressFromLog(event.payload)
      }
    }).then((unlisten) => unlisteners.push(unlisten))

    void listen<ServerInstance>('asa:instance-status', (event) => {
      if (!disposed) replaceInstance(event.payload)
    }).then((unlisten) => unlisteners.push(unlisten))

    void listen<JobProgress>('asa:job-progress', (event) => {
      if (disposed || !event.payload.instanceId) return
      setJobProgress((current) => ({
        ...current,
        [event.payload.instanceId as string]: mergeJobProgress(current[event.payload.instanceId as string], event.payload),
      }))
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
  }, [appendLogLine, messageApi, refreshLogs, replaceInstance, updateJobProgressFromLog])

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
      await saveInstanceConfig(selected.id, nextConfig, mods)
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
      content: selected.status === 'running' ? '运行中的实例会先保存世界并重启。' : '配置会写入真实 ARK 配置文件。',
      okText: selected.status === 'running' ? '保存并重启' : '保存并应用',
      cancelText: '取消',
      onOk: async () => {
        const updated = await applyInstanceConfig(selected.id, nextConfig, mods)
        replaceInstance(updated)
        setConfig(nextConfig)
        setDirty(false)
        await refreshLogs()
        messageApi.success('配置已应用')
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
      const existing = await WebviewWindow.getByLabel(ADD_INSTANCE_WINDOW_LABEL)
      if (existing) {
        await existing.setFocus()
        return
      }

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
          <Tooltip title="停止"><Button size="small" type="text" icon={<StopFilled />} danger={item.status === 'running'} disabled={item.status === 'stopped' || item.status === 'stopping'} onClick={(event) => { event.stopPropagation(); stopInstance(item) }} /></Tooltip>
          <Dropdown
            menu={{
              items: [
                { key: 'install', label: '安装/更新', icon: <CloudDownloadOutlined /> },
                { key: 'backup', label: '创建备份', icon: <DatabaseOutlined /> },
                { key: 'folder', label: '打开目录', icon: <FolderOpenOutlined /> },
                { key: 'edit', label: '编辑实例', icon: <EditOutlined /> },
                { key: 'delete', label: '删除实例', icon: <DeleteOutlined />, danger: true, disabled: !canDeleteInstance(item.status) },
              ],
              onClick: ({ key }) => {
                if (key === 'install') void installInstance(item)
                if (key === 'backup') void createBackup(item.id).then((backup) => messageApi.success(`备份已创建：${backup.path}`)).catch((error) => messageApi.error(`创建备份失败：${String(error)}`))
                if (key === 'folder') void openInstanceDirectory(item.id).catch((error) => messageApi.error(`打开目录失败：${String(error)}`))
                if (key === 'edit') {
                  setSelectedId(item.id)
                  setSelectedRows([item.id])
                }
                if (key === 'delete') deleteInstanceRecord(item)
              },
            }}
            trigger={['click']}
          >
            <Button size="small" type="text" icon={<EllipsisOutlined />} onClick={(event) => event.stopPropagation()} />
          </Dropdown>
        </Space.Compact>
      ),
    },
  ], [deleteInstanceRecord, messageApi, selectedId])

  const activeProgressIds = useMemo(
    () => instances
      .filter((item) => item.status === 'updating' && isActiveJobProgress(jobProgress[item.id]))
      .map((item) => item.id),
    [instances, jobProgress],
  )

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
                expandable={{
                  expandedRowKeys: activeProgressIds,
                  expandedRowRender: (item) => <InstanceJobProgress progress={jobProgress[item.id]} />,
                  rowExpandable: (item) => isActiveJobProgress(jobProgress[item.id]),
                  showExpandColumn: false,
                }}
                rowSelection={{ selectedRowKeys: selectedRows, onChange: setSelectedRows, columnWidth: 36 }}
                onRow={(item) => ({ onClick: () => { setSelectedId(item.id); setSelectedRows([item.id]) } })}
                rowClassName={(item) => item.id === selectedId ? 'selected-instance-row' : ''}
              />
            </section>

            <section className="surface cluster-log-card">
              <div className="surface__title">
                <span><LineChartOutlined /> 集群日志 / 实例状态</span>
                <Space><Checkbox checked={autoScrollLogs} onChange={(event) => setAutoScrollLogs(event.target.checked)}>自动滚动</Checkbox><Button size="small" onClick={() => void handleClearAllLogs()}>清除所有日志</Button></Space>
              </div>
              <Tabs
                className="log-tabs"
                activeKey={activeLogTab}
                onChange={setActiveLogTab}
                items={logTabItems}
                size="small"
              />
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
            <Button danger icon={<DeleteOutlined />} disabled={!selected || !canDeleteInstance(selected.status)} onClick={() => selected && deleteInstanceRecord(selected)}>删除所选实例</Button>
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
