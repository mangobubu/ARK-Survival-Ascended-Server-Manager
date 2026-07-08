import { Channel, invoke } from '@tauri-apps/api/core'
import { getWebApiBaseUrl, isTauriRuntime } from './runtime'
import type {
  AddInstancePayload,
  AsaConfigMetadataDocument,
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
  WebAcmeCertificateStatus,
  WebSecurityBanRecord,
  WebSecurityUnbanResult,
} from './types'

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

interface WebRiskConfirmation {
  token: string
  command: string
  expiresInSeconds: number
}

type WebCommandRisk = 'read' | 'write' | 'high'

interface WebCommandSecurityPolicy {
  command: string
  label: string
  risk: WebCommandRisk
}

let webCommandSecurityPoliciesPromise: Promise<Map<string, WebCommandSecurityPolicy>> | null = null

export function getAuthorizedWebUrl(path: string) {
  const url = new URL(path, `${getWebApiBaseUrl()}/`)
  return url.toString()
}

async function parseWebApiPayload<T>(response: Response, fallbackError: string) {
  const payload = await response.json().catch(() => null) as { ok?: boolean; data?: T; error?: string } | null
  if (!response.ok || !payload?.ok) {
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
  return parseWebApiPayload<void>(response, 'Web 登录失败')
}

export async function logoutWeb() {
  try {
    await fetch(`${getWebApiBaseUrl()}/api/auth/logout`, {
      method: 'POST',
      credentials: 'same-origin',
    })
  } finally {
    // Web 登录态仅由 HttpOnly Cookie 承载，前端不保存可读取 token。
  }
}

async function invokeCommand<T>(command: string, args: Record<string, unknown> = {}) {
  if (isTauriRuntime()) return invoke<T>(command, args)
  return webInvoke<T>(command, args)
}

async function webInvoke<T>(command: string, args: Record<string, unknown>) {
  const policy = await getWebCommandSecurityPolicy(command)
  const payloadArgs = policy.risk === 'high'
    ? { ...args, riskConfirmationToken: await requestWebRiskConfirmation(policy) }
    : args
  const response = await fetch(`${getWebApiBaseUrl()}/api/invoke`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ command, args: payloadArgs }),
  })

  return parseWebApiPayload<T>(response, `Web API 调用失败：${command}`)
}

async function getWebCommandSecurityPolicy(command: string) {
  const policies = await loadWebCommandSecurityPolicies()
  const policy = policies.get(command)
  if (!policy) {
    throw new Error(`Web 命令 ${command} 未出现在后端安全元数据中，已阻止执行`)
  }
  return policy
}

async function loadWebCommandSecurityPolicies() {
  if (!webCommandSecurityPoliciesPromise) {
    webCommandSecurityPoliciesPromise = fetch(`${getWebApiBaseUrl()}/api/commands/security`, {
      credentials: 'same-origin',
    })
      .then((response) => parseWebApiPayload<WebCommandSecurityPolicy[]>(response, '无法读取 Web 命令安全元数据'))
      .then((policies) => new Map(policies.map((policy) => [policy.command, policy])))
      .catch((error) => {
        webCommandSecurityPoliciesPromise = null
        throw error
      })
  }
  return webCommandSecurityPoliciesPromise
}

async function requestWebRiskConfirmation(policy: WebCommandSecurityPolicy) {
  const confirmed = window.confirm(
    `即将执行高风险 Web 管理操作：${policy.label}\n\n该操作可能修改服务器配置、执行 RCON、恢复备份或删除实例。确认继续吗？`,
  )
  if (!confirmed) {
    throw new Error('已取消高风险 Web 管理操作')
  }

  const response = await fetch(`${getWebApiBaseUrl()}/api/risk/confirm`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ command: policy.command }),
  })
  const confirmation = await parseWebApiPayload<WebRiskConfirmation>(response, '无法获取 Web 高风险操作确认令牌')
  return confirmation.token
}

export const getSettings = () => invokeCommand<GlobalSettings>('get_settings')

export const getAsaConfigMetadata = () => invokeCommand<AsaConfigMetadataDocument>('get_asa_config_metadata')

export const saveSettings = (settings: GlobalSettings) => invokeCommand<GlobalSettings>('save_settings', { settings })

export const listWebSecurityBans = () => invokeCommand<WebSecurityBanRecord[]>('list_web_security_bans')

export const getWebAcmeCertificateStatus = () => invokeCommand<WebAcmeCertificateStatus | null>('get_web_acme_certificate_status')

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
