import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  ClearOutlined,
  CodeOutlined,
  SendOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons'
import { Button, Empty, Input, message, Space, Tag, Typography } from 'antd'
import { executeRconCommand, listInstances } from './backendApi'
import { RCON_COMMANDS, normalizeRconInput, type RconCommandDoc } from './rconCommands'
import type { ServerInstance, ServerStatus } from './types'

const { Text } = Typography

type RconLogKind = 'system' | 'success' | 'command' | 'response' | 'error'
type RconConnectionStatus = 'checking' | 'ready' | 'failed'

interface RconLogLine {
  id: number
  time: string
  kind: RconLogKind
  message: string
}

interface RconWindowProps {
  instanceId?: string
  name?: string
  rconPort?: number
}

const categoryOrder: RconCommandDoc['category'][] = ['状态查询', '服务器公告', '运维控制', '玩家管理']

function currentTimeText() {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false })
}

function statusText(status?: ServerStatus) {
  const labels: Record<ServerStatus, string> = {
    running: '运行中',
    stopped: '已停止',
    stopping: '停止中',
    starting: '启动中',
    updating: '更新中',
    backingUp: '备份中',
    error: '异常',
  }
  return status ? labels[status] : '读取中'
}

function statusColor(status?: ServerStatus) {
  if (status === 'running') return 'success'
  if (status === 'error') return 'error'
  if (!status || status === 'starting' || status === 'stopping' || status === 'updating' || status === 'backingUp') return 'processing'
  return 'default'
}

function logKindLabel(kind: RconLogKind) {
  if (kind === 'success') return '成功'
  if (kind === 'command') return '命令'
  if (kind === 'response') return '响应'
  if (kind === 'error') return '错误'
  return '系统'
}

function commandToneColor(tone?: RconCommandDoc['tone']) {
  if (tone === 'danger') return 'red'
  if (tone === 'warning') return 'gold'
  return 'blue'
}

export default function RconWindow({ instanceId: propInstanceId, name: propName, rconPort: propRconPort }: RconWindowProps = {}) {
  const params = useMemo(() => new URLSearchParams(window.location.search), [])
  const instanceId = propInstanceId ?? params.get('instanceId') ?? ''
  const initialName = propName ?? params.get('name') ?? ''
  const initialRconPort = propRconPort ?? (Number(params.get('rconPort') ?? 0) || null)

  const [instance, setInstance] = useState<ServerInstance | null>(null)
  const [instanceName, setInstanceName] = useState(initialName)
  const [commandText, setCommandText] = useState('')
  const [commandFocused, setCommandFocused] = useState(false)
  const [sending, setSending] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState<RconConnectionStatus>('checking')
  const [activeSuggestionIndex, setActiveSuggestionIndex] = useState(0)
  const [logs, setLogs] = useState<RconLogLine[]>([])
  const logIdRef = useRef(1)
  const logEndRef = useRef<HTMLDivElement>(null)
  const connectionProbeStartedRef = useRef(false)
  const [messageApi, contextHolder] = message.useMessage()

  const appendLog = useCallback((kind: RconLogKind, message: string) => {
    setLogs((current) => [
      ...current,
      { id: logIdRef.current++, time: currentTimeText(), kind, message },
    ].slice(-200))
  }, [])

  useEffect(() => {
    document.title = `${initialName || '实例'} RCON管理`
    appendLog('system', 'RCON 管理窗口已打开。输入 / 可查看常用 ASA RCON 命令建议。')
    if (!instanceId) {
      setConnectionStatus('failed')
      appendLog('error', 'RCON连接失败：缺少实例 ID，命令输入已禁用。')
    }
  }, [appendLog, initialName, instanceId])

  const probeRconConnection = useCallback(async (target: ServerInstance | null) => {
    if (!instanceId || connectionProbeStartedRef.current) return
    connectionProbeStartedRef.current = true
    setConnectionStatus('checking')

    const targetName = target?.name ?? (initialName || '当前实例')
    const targetPort = target?.rconPort ?? initialRconPort ?? '-'
    appendLog('system', `正在检测 ${targetName} 的 RCON 连接：127.0.0.1:${targetPort}`)

    try {
      await executeRconCommand(instanceId, 'ListPlayers')
      setConnectionStatus('ready')
      appendLog('success', `RCON连接成功：${targetName} 127.0.0.1:${targetPort}，命令输入已启用。`)
    } catch (error) {
      setConnectionStatus('failed')
      appendLog('error', `RCON连接失败：${String(error)}，命令输入已禁用。`)
    }
  }, [appendLog, initialName, initialRconPort, instanceId])

  useEffect(() => {
    let alive = true
    if (!instanceId) return undefined

    void listInstances()
      .then((items) => {
        if (!alive) return
        const found = items.find((item) => item.id === instanceId) ?? null
        setInstance(found)
        if (found) {
          setInstanceName(found.name)
          document.title = `${found.name} RCON管理`
          appendLog('system', `已载入实例：${found.name}，RCON 地址 127.0.0.1:${found.rconPort}。`)
        } else {
          appendLog('error', `未在管理器列表中找到实例：${instanceId}`)
        }
        void probeRconConnection(found)
      })
      .catch((error) => {
        setConnectionStatus('failed')
        appendLog('error', `RCON连接失败：读取实例信息失败：${String(error)}，命令输入已禁用。`)
      })

    return () => {
      alive = false
    }
  }, [appendLog, instanceId, probeRconConnection])

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ block: 'end' })
  }, [logs])

  const suggestionQuery = commandText.startsWith('/') ? commandText.slice(1).trimStart().toLowerCase() : ''
  const inputDisabled = connectionStatus !== 'ready'
  const showSuggestions = !inputDisabled && commandFocused && commandText.startsWith('/')
  const suggestions = useMemo(() => {
    if (!commandText.startsWith('/')) return []
    const matched = suggestionQuery
      ? RCON_COMMANDS.filter((item) => {
        const haystack = `${item.command} ${item.signature} ${item.description} ${item.category}`.toLowerCase()
        return haystack.includes(suggestionQuery)
      })
      : RCON_COMMANDS
    return matched.slice(0, 8)
  }, [commandText, suggestionQuery])

  useEffect(() => {
    setActiveSuggestionIndex(0)
  }, [suggestionQuery])

  const groupedCommands = useMemo(() => categoryOrder.map((category) => ({
    category,
    items: RCON_COMMANDS.filter((item) => item.category === category),
  })), [])

  const insertCommand = useCallback((item: RconCommandDoc) => {
    if (inputDisabled) {
      messageApi.warning('RCON 未连接，命令输入已禁用')
      return
    }
    setCommandText(`/${item.insertText}`)
    setCommandFocused(true)
    window.setTimeout(() => document.querySelector<HTMLTextAreaElement>('.rcon-command-input textarea')?.focus(), 0)
  }, [inputDisabled, messageApi])

  const sendCommand = useCallback(async () => {
    const command = normalizeRconInput(commandText)
    if (!instanceId) {
      messageApi.error('缺少实例 ID，无法发送 RCON 命令')
      return
    }
    if (inputDisabled) {
      messageApi.warning('RCON 未连接，命令输入已禁用')
      return
    }
    if (!command) {
      messageApi.warning('请输入 RCON 命令')
      return
    }

    appendLog('command', `> ${command}`)
    setSending(true)
    try {
      const response = await executeRconCommand(instanceId, command)
      appendLog('response', response.trim() || '执行完成，RCON 未返回内容。')
      setCommandText('')
    } catch (error) {
      const errorText = String(error)
      appendLog('error', errorText)
      messageApi.error(`RCON 执行失败：${errorText}`)
    } finally {
      setSending(false)
    }
  }, [appendLog, commandText, inputDisabled, instanceId, messageApi])

  const handleCommandKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (showSuggestions && suggestions.length > 0) {
      if (event.key === 'ArrowDown') {
        event.preventDefault()
        setActiveSuggestionIndex((current) => (current + 1) % suggestions.length)
        return
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault()
        setActiveSuggestionIndex((current) => (current - 1 + suggestions.length) % suggestions.length)
        return
      }
      const exactSuggestion = suggestions.some((item) => item.command.toLowerCase() === normalizeRconInput(commandText).toLowerCase())
      if (event.key === 'Tab' || (event.key === 'Enter' && !event.shiftKey && !exactSuggestion)) {
        event.preventDefault()
        insertCommand(suggestions[activeSuggestionIndex] ?? suggestions[0])
        return
      }
    }

    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      void sendCommand()
    }
  }

  const titleName = instanceName || '实例'
  const rconPort = instance?.rconPort ?? initialRconPort ?? '-'
  const connectionTag = connectionStatus === 'ready'
    ? { color: 'success', text: 'RCON已连接' }
    : connectionStatus === 'checking'
      ? { color: 'processing', text: 'RCON检测中' }
      : { color: 'error', text: 'RCON连接失败' }
  const commandPlaceholder = inputDisabled
    ? 'RCON 连接失败或尚未就绪，命令输入已禁用'
    : '输入 / 查看常用命令，例如 /ListPlayers；也可直接输入 SaveWorld'
  const canSend = Boolean(instanceId && normalizeRconInput(commandText) && !sending && !inputDisabled)

  return (
    <div className="rcon-window">
      {contextHolder}
      <header className="rcon-header">
        <div className="rcon-header__mark"><CodeOutlined /></div>
        <div className="rcon-header__main">
          <h3>{titleName} RCON管理</h3>
          <Text>127.0.0.1:{rconPort} · 命令会自动移除前导 / 后发送到 ASA RCON</Text>
        </div>
        <Space size={8}>
          <Tag color={statusColor(instance?.status)}>{statusText(instance?.status)}</Tag>
          <Tag color={connectionTag.color}>{connectionTag.text}</Tag>
          <Tag color="blue">RCON {rconPort}</Tag>
        </Space>
      </header>

      <main className="rcon-layout">
        <section className="rcon-left">
          <section className="rcon-panel rcon-log-panel">
            <div className="rcon-panel__title">
              <span><ThunderboltOutlined /> RCON日志</span>
              <small>{logs.length}</small>
            </div>
            <div className="rcon-log">
              {logs.length === 0 ? (
                <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="暂无 RCON 日志" />
              ) : logs.map((line) => (
                <div className={`rcon-log-line rcon-log-line--${line.kind}`} key={line.id}>
                  <span>[{line.time}]</span>
                  <b>{logKindLabel(line.kind)}</b>
                  <pre>{line.message}</pre>
                </div>
              ))}
              <div ref={logEndRef} />
            </div>
          </section>

          <section className="rcon-panel rcon-command-panel">
            <div className="rcon-panel__title">
              <span>命令输入</span>
              <small>Enter 发送 · Shift+Enter 换行 · / 补全</small>
            </div>
            <div className="rcon-command-box">
              {showSuggestions && (
                <div className="rcon-suggestions" onMouseDown={(event) => event.preventDefault()}>
                  {suggestions.length === 0 ? (
                    <div className="rcon-suggestions__empty">没有匹配的 RCON 命令</div>
                  ) : suggestions.map((item, index) => (
                    <button
                      type="button"
                      className={`rcon-suggestion${index === activeSuggestionIndex ? ' rcon-suggestion--active' : ''}`}
                      key={item.command}
                      onClick={() => insertCommand(item)}
                    >
                      <strong>/{item.signature}</strong>
                      <span>{item.description}</span>
                    </button>
                  ))}
                </div>
              )}
              <Input.TextArea
                className="rcon-command-input"
                value={commandText}
                onChange={(event) => setCommandText(event.target.value)}
                onFocus={() => setCommandFocused(true)}
                onBlur={() => window.setTimeout(() => setCommandFocused(false), 120)}
                onKeyDown={handleCommandKeyDown}
                disabled={inputDisabled}
                autoSize={{ minRows: 3, maxRows: 5 }}
                placeholder={commandPlaceholder}
              />
              <div className="rcon-command-actions">
                <Text type="secondary">建议先用 ListPlayers 测试连接；维护前执行 SaveWorld。</Text>
                <Space>
                  <Button icon={<ClearOutlined />} disabled={logs.length === 0} onClick={() => setLogs([])}>清空日志</Button>
                  <Button type="primary" icon={<SendOutlined />} loading={sending} disabled={!canSend} onClick={() => void sendCommand()}>发送命令</Button>
                </Space>
              </div>
            </div>
          </section>
        </section>

        <aside className="rcon-panel rcon-docs">
          <div className="rcon-panel__title">
            <span>ASA RCON命令说明</span>
            <small>常用</small>
          </div>
          <div className="rcon-docs__content">
            <div className="rcon-docs__note">
              <b>输入技巧：</b>在左侧输入框键入 <code>/</code> 会列出命令；点击右侧命令卡片也会自动填入输入框。
            </div>
            {groupedCommands.map((group) => (
              <section className="rcon-doc-group" key={group.category}>
                <h4>{group.category}</h4>
                <div className="rcon-doc-list">
                  {group.items.map((item) => (
                    <button type="button" className="rcon-doc-item" key={item.command} onClick={() => insertCommand(item)}>
                      <div>
                        <strong>{item.signature}</strong>
                        <Tag color={commandToneColor(item.tone)}>{item.category}</Tag>
                      </div>
                      <p>{item.description}</p>
                      <code>{item.example}</code>
                    </button>
                  ))}
                </div>
              </section>
            ))}
          </div>
        </aside>
      </main>
    </div>
  )
}
