import { listen } from '@tauri-apps/api/event'
import { getAuthorizedWebUrl } from './backendApi'
import { isTauriRuntime } from './runtime'
import type {
  GlobalSettings,
  InstanceCreatedEvent,
  JobProgress,
  LogClearScope,
  LogLine,
  ModItem,
  ServerConfig,
  ServerInstance,
} from './types'
import { ADD_INSTANCE_CREATED_EVENT, SETTINGS_CHANGED_EVENT } from './windowEvents'

export { ADD_INSTANCE_CREATED_EVENT, SETTINGS_CHANGED_EVENT }

export const LOG_LINE_EVENT = 'asa:log-line'
export const LOGS_CLEARED_EVENT = 'asa:logs-cleared'
export const LOGS_RESET_EVENT = 'asa:logs-reset'
export const INSTANCE_STATUS_EVENT = 'asa:instance-status'
export const JOB_PROGRESS_EVENT = 'asa:job-progress'
export const INSTANCE_CONFIG_CHANGED_EVENT = 'asa:instance-config-changed'
export const INSTANCE_DELETED_EVENT = 'asa:instance-deleted'
export const INSTANCES_CHANGED_EVENT = 'asa:instances-changed'

export interface InstanceConfigChangedEvent {
  instanceId: string
  instance: ServerInstance
  config: Partial<ServerConfig>
  mods: ModItem[]
}

type BackendEventPayloadMap = {
  [ADD_INSTANCE_CREATED_EVENT]: InstanceCreatedEvent
  [SETTINGS_CHANGED_EVENT]: GlobalSettings
  [LOG_LINE_EVENT]: LogLine
  [LOGS_CLEARED_EVENT]: LogClearScope
  [LOGS_RESET_EVENT]: Record<string, never>
  [INSTANCE_STATUS_EVENT]: ServerInstance
  [JOB_PROGRESS_EVENT]: JobProgress
  [INSTANCE_CONFIG_CHANGED_EVENT]: InstanceConfigChangedEvent
  [INSTANCE_DELETED_EVENT]: ServerInstance
  [INSTANCES_CHANGED_EVENT]: Record<string, never>
}

type BackendEventName = keyof BackendEventPayloadMap & string
type BackendEventListener<Name extends BackendEventName> = (payload: BackendEventPayloadMap[Name]) => void
type UnknownListener = (payload: unknown) => void

const webListeners = new Map<string, Set<UnknownListener>>()
const registeredWebEventNames = new Set<string>()
let webEventSource: EventSource | null = null
let webSubscriptionCount = 0

function webEventsUrl() {
  return getAuthorizedWebUrl('/api/events')
}

function ensureWebEventSource() {
  if (webEventSource) return webEventSource
  webEventSource = new EventSource(webEventsUrl())
  webEventSource.onerror = (error) => {
    console.error('Web 实时同步连接异常，浏览器会自动重连', error)
  }
  return webEventSource
}

function handleWebEvent(event: Event) {
  const message = event as MessageEvent<string>
  const listeners = webListeners.get(message.type)
  if (!listeners || listeners.size === 0) return

  let payload: unknown
  try {
    payload = message.data ? JSON.parse(message.data) : {}
  } catch (error) {
    console.error(`解析实时同步事件失败：${message.type}`, error)
    return
  }

  listeners.forEach((listener) => listener(payload))
}

function subscribeWebEvent<Name extends BackendEventName>(
  eventName: Name,
  onPayload: BackendEventListener<Name>,
) {
  const eventSource = ensureWebEventSource()
  if (!registeredWebEventNames.has(eventName)) {
    eventSource.addEventListener(eventName, handleWebEvent)
    registeredWebEventNames.add(eventName)
  }

  const listeners = webListeners.get(eventName) ?? new Set<UnknownListener>()
  listeners.add(onPayload as UnknownListener)
  webListeners.set(eventName, listeners)
  webSubscriptionCount += 1

  return () => {
    listeners.delete(onPayload as UnknownListener)
    if (listeners.size === 0) webListeners.delete(eventName)
    webSubscriptionCount = Math.max(0, webSubscriptionCount - 1)
    if (webSubscriptionCount === 0 && webEventSource) {
      webEventSource.close()
      webEventSource = null
      registeredWebEventNames.clear()
    }
  }
}

function subscribeTauriEvent<Name extends BackendEventName>(
  eventName: Name,
  onPayload: BackendEventListener<Name>,
) {
  let disposed = false
  let unlistenTauri: (() => void) | undefined

  void listen<BackendEventPayloadMap[Name]>(eventName, (event) => {
    if (!disposed) onPayload(event.payload)
  }).then((unlisten) => {
    if (disposed) unlisten()
    else unlistenTauri = unlisten
  }).catch((error) => {
    console.error(`监听实时同步事件失败：${eventName}`, error)
  })

  return () => {
    disposed = true
    unlistenTauri?.()
  }
}

export function subscribeBackendEvent<Name extends BackendEventName>(
  eventName: Name,
  onPayload: BackendEventListener<Name>,
) {
  return isTauriRuntime()
    ? subscribeTauriEvent(eventName, onPayload)
    : subscribeWebEvent(eventName, onPayload)
}
