import { Channel, invoke } from '@tauri-apps/api/core'
import { getWebApiBaseUrl, isTauriRuntime } from './runtime'
import type {
  AddInstancePayload,
  BackupItem,
  ExportResult,
  GlobalSettings,
  ImportResult,
  ImportedServerConfigPreview,
  InstancePortKind,
  JobProgress,
  LogClearScope,
  LogLine,
  ModItem,
  PortCheckResult,
  ServerConfig,
  ServerInstance,
  SteamCmdCheck,
  SteamCmdInstallResult,
  SteamCmdProgress,
  WebSecurityBanRecord,
  WebSecurityUnbanResult,
} from './types'

const WEB_AUTH_TOKEN_KEY = 'asa-web-auth-token'

export interface WebAuthStatus {
  configured: boolean
  captchaRequired: boolean
}

export interface WebCaptcha {
  required: boolean
  token: string
  imageSvg: string
  expiresInSeconds: number
}

export interface WebCaptchaInput {
  token: string
  answer: string
}

export function getWebAuthToken() {
  return window.localStorage.getItem(WEB_AUTH_TOKEN_KEY)
}

export function setWebAuthToken(token: string) {
  window.localStorage.setItem(WEB_AUTH_TOKEN_KEY, token)
}

export function clearWebAuthToken() {
  window.localStorage.removeItem(WEB_AUTH_TOKEN_KEY)
}

export function getAuthorizedWebUrl(path: string) {
  const token = getWebAuthToken()
  const url = new URL(path, `${getWebApiBaseUrl()}/`)
  if (token) url.searchParams.set('token', token)
  return url.toString()
}

async function parseWebApiPayload<T>(response: Response, fallbackError: string) {
  const payload = await response.json().catch(() => null) as { ok?: boolean; data?: T; error?: string } | null
  if (!response.ok || !payload?.ok) {
    if (response.status === 401) clearWebAuthToken()
    throw new Error(payload?.error ?? fallbackError)
  }
  return payload.data as T
}

export async function getWebAuthStatus() {
  const response = await fetch(`${getWebApiBaseUrl()}/api/auth/status`)
  return parseWebApiPayload<WebAuthStatus>(response, '无法读取 Web 鉴权状态')
}

export async function getWebCaptcha() {
  const response = await fetch(`${getWebApiBaseUrl()}/api/auth/captcha`, {
    credentials: 'same-origin',
  })
  return parseWebApiPayload<WebCaptcha>(response, '无法刷新 Web 登录验证码')
}

export async function loginWeb(username: string, password: string, captcha?: WebCaptchaInput) {
  const response = await fetch(`${getWebApiBaseUrl()}/api/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'same-origin',
    body: JSON.stringify({
      username,
      password,
      captchaToken: captcha?.token,
      captchaAnswer: captcha?.answer,
    }),
  })
  const data = await parseWebApiPayload<{ token: string }>(response, 'Web 登录失败')
  clearWebAuthToken()
  return data
}

export async function logoutWeb() {
  const token = getWebAuthToken()
  try {
    if (token) {
      await fetch(`${getWebApiBaseUrl()}/api/auth/logout`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { Authorization: `Bearer ${token}` },
      })
    } else {
      await fetch(`${getWebApiBaseUrl()}/api/auth/logout`, {
        method: 'POST',
        credentials: 'same-origin',
      })
    }
  } finally {
    clearWebAuthToken()
  }
}

async function invokeCommand<T>(command: string, args: Record<string, unknown> = {}) {
  if (isTauriRuntime()) return invoke<T>(command, args)
  return webInvoke<T>(command, args)
}

async function webInvoke<T>(command: string, args: Record<string, unknown>) {
  const token = getWebAuthToken()
  const response = await fetch(`${getWebApiBaseUrl()}/api/invoke`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({ command, args }),
  })

  return parseWebApiPayload<T>(response, `Web API 调用失败：${command}`)
}

export const getSettings = () => invokeCommand<GlobalSettings>('get_settings')

export const saveSettings = (settings: GlobalSettings) => invokeCommand<GlobalSettings>('save_settings', { settings })

export const listWebSecurityBans = () => invokeCommand<WebSecurityBanRecord[]>('list_web_security_bans')

export const unbanWebSecurityIp = (ip: string) => invokeCommand<WebSecurityUnbanResult>('unban_web_security_ip', { ip })

export const checkSteamCmd = (path: string) => invokeCommand<SteamCmdCheck>('check_steamcmd', { path })

export const installSteamCmd = (parentPath: string, onProgress: (progress: SteamCmdProgress) => void) => {
  if (isTauriRuntime()) {
    const progress = new Channel<SteamCmdProgress>()
    progress.onmessage = onProgress
    return invoke<SteamCmdInstallResult>('install_steamcmd', { parentPath, progress })
  }
  return invokeCommand<SteamCmdInstallResult>('install_steamcmd', { parentPath })
}

export const listInstances = () => invokeCommand<ServerInstance[]>('list_instances')

export const clearStartupAutoUpdateSkipFlags = (instanceIds: string[]) =>
  invokeCommand<ServerInstance[]>('clear_startup_auto_update_skip_flags', { instanceIds })

export const checkInstancePort = (port: number, portKind: InstancePortKind) =>
  invokeCommand<PortCheckResult>('check_instance_port', { port, portKind })

export const createInstance = (payload: AddInstancePayload) => invokeCommand<ServerInstance>('create_instance', { payload })

export const readServerDirectoryConfig = (path: string) =>
  invokeCommand<ImportedServerConfigPreview>('read_server_directory_config', { path })

export const getInstanceConfig = (instanceId: string) => invokeCommand<Partial<ServerConfig>>('get_instance_config', { instanceId })

export const getInstanceMods = (instanceId: string) => invokeCommand<ModItem[]>('get_instance_mods', { instanceId })

export const saveInstanceConfig = (instanceId: string, config: ServerConfig, mods: ModItem[]) =>
  invokeCommand<ServerInstance>('save_instance_config', { instanceId, config, mods })

export const applyInstanceConfig = (instanceId: string, config: ServerConfig, mods: ModItem[]) =>
  invokeCommand<ServerInstance>('apply_instance_config', { instanceId, config, mods })

export const updateInstanceMods = (instanceId: string, mods: ModItem[]) =>
  invokeCommand<ModItem[]>('update_instance_mods', { instanceId, mods })

export const checkModUpdates = (mods: ModItem[]) => invokeCommand<ModItem[]>('check_mod_updates', { mods })

export const installOrUpdateInstance = (instanceId: string, onProgress: (progress: JobProgress) => void) => {
  if (!isTauriRuntime()) return invokeCommand<ServerInstance>('install_or_update_instance', { instanceId })
  const progress = new Channel<JobProgress>()
  progress.onmessage = onProgress
  return invoke<ServerInstance>('install_or_update_instance', { instanceId, progress })
}

export const startInstance = (instanceId: string) => invokeCommand<ServerInstance>('start_instance', { instanceId })

export const stopInstance = (instanceId: string) => invokeCommand<ServerInstance>('stop_instance', { instanceId })

export const restartInstance = (instanceId: string) => invokeCommand<ServerInstance>('restart_instance', { instanceId })

export const refreshInstanceStatus = (instanceId: string) => invokeCommand<ServerInstance>('refresh_instance_status', { instanceId })

export const executeRconCommand = (instanceId: string, command: string) =>
  invokeCommand<string>('execute_rcon_command', { instanceId, command })

export const queryLogs = (limit = 500) => invokeCommand<LogLine[]>('query_logs', { limit })

export const clearLogs = () => invokeCommand<void>('clear_logs')

export const clearScopedLogs = (scope: LogClearScope) =>
  invokeCommand<void>('clear_scoped_logs', {
    source: scope.source,
    instance: scope.instance ?? undefined,
    serverLogKind: scope.serverLogKind ?? undefined,
  })

export const createBackup = (instanceId: string) => invokeCommand<BackupItem>('create_backup', { instanceId })

export const listBackups = (instanceId: string) => invokeCommand<BackupItem[]>('list_backups', { instanceId })

export const restoreBackup = (instanceId: string, backupPath: string) =>
  invokeCommand<void>('restore_backup', { instanceId, backupPath })

export const exportInstanceConfig = (instanceIds: string[]) =>
  invokeCommand<ExportResult>('export_instance_config', { instanceIds })

export const exportCluster = () => invokeCommand<ExportResult>('export_cluster')

export const importInstanceConfig = (path: string) => invokeCommand<ImportResult>('import_instance_config', { path })

export const deleteInstance = (instanceId: string) => invokeCommand<ServerInstance>('delete_instance', { instanceId })

export const openInstanceDirectory = (instanceId: string) => invokeCommand<void>('open_instance_directory', { instanceId })

export const openDirectoryPath = (path: string) => invokeCommand<void>('open_directory_path', { path })
