import type { RefObject } from 'react'
import { DeleteOutlined } from '@ant-design/icons'
import { Button, Tooltip } from 'antd'
import type { LogClearScope, LogLine } from './types'

export const APPLICATION_LOG_TAB_KEY = 'application'
export const SERVER_CONSOLE_LOG_KIND = 'console'
export const SERVER_FILE_LOG_KIND = 'file'

export type ServerLogKind = NonNullable<LogLine['serverLogKind']>

export function getServerLogKind(line: LogLine): ServerLogKind {
  return line.serverLogKind ?? SERVER_CONSOLE_LOG_KIND
}

export function matchesLogClearScope(line: LogLine, scope: LogClearScope) {
  if (line.source !== scope.source) return false
  if (scope.instance && line.instance !== scope.instance) return false
  if (scope.serverLogKind && getServerLogKind(line) !== scope.serverLogKind) return false
  return true
}

export function LogTabLabel({ title, count }: { title: string; count: number }) {
  return (
    <span className="log-tab-label" title={title}>
      <span>{title}</span>
      <small>{count}</small>
    </span>
  )
}

export function LogConsole({
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
  consoleRef?: RefObject<HTMLDivElement | null>
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
            <span>[{line.time}]</span>{showInstance && <b title={line.instance}>[{line.instance}]</b>}<span>{line.message}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

export function InstanceLogPanes({
  consoleLines,
  fileLines,
  consoleRef,
  fileRef,
  onClearConsole,
  onClearFile,
}: {
  consoleLines: LogLine[]
  fileLines: LogLine[]
  consoleRef?: RefObject<HTMLDivElement | null>
  fileRef?: RefObject<HTMLDivElement | null>
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
