import { Channel, invoke } from '@tauri-apps/api/core'
import type {
  AddInstancePayload,
  BackupItem,
  ExportResult,
  GlobalSettings,
  ImportResult,
  JobProgress,
  LogLine,
  ModItem,
  ServerConfig,
  ServerInstance,
} from './types'

export const getSettings = () => invoke<GlobalSettings>('get_settings')

export const saveSettings = (settings: GlobalSettings) => invoke<GlobalSettings>('save_settings', { settings })

export const listInstances = () => invoke<ServerInstance[]>('list_instances')

export const createInstance = (payload: AddInstancePayload) => invoke<ServerInstance>('create_instance', { payload })

export const getInstanceConfig = (instanceId: string) => invoke<Partial<ServerConfig>>('get_instance_config', { instanceId })

export const getInstanceMods = (instanceId: string) => invoke<ModItem[]>('get_instance_mods', { instanceId })

export const saveInstanceConfig = (instanceId: string, config: ServerConfig, mods: ModItem[]) =>
  invoke<void>('save_instance_config', { instanceId, config, mods })

export const applyInstanceConfig = (instanceId: string, config: ServerConfig, mods: ModItem[]) =>
  invoke<ServerInstance>('apply_instance_config', { instanceId, config, mods })

export const updateInstanceMods = (instanceId: string, mods: ModItem[]) =>
  invoke<ModItem[]>('update_instance_mods', { instanceId, mods })

export const checkModUpdates = (mods: ModItem[]) => invoke<ModItem[]>('check_mod_updates', { mods })

export const installOrUpdateInstance = (instanceId: string, onProgress: (progress: JobProgress) => void) => {
  const progress = new Channel<JobProgress>()
  progress.onmessage = onProgress
  return invoke<ServerInstance>('install_or_update_instance', { instanceId, progress })
}

export const startInstance = (instanceId: string) => invoke<ServerInstance>('start_instance', { instanceId })

export const stopInstance = (instanceId: string) => invoke<ServerInstance>('stop_instance', { instanceId })

export const restartInstance = (instanceId: string) => invoke<ServerInstance>('restart_instance', { instanceId })

export const refreshInstanceStatus = (instanceId: string) => invoke<ServerInstance>('refresh_instance_status', { instanceId })

export const queryLogs = (limit = 500) => invoke<LogLine[]>('query_logs', { limit })

export const clearLogs = () => invoke<void>('clear_logs')

export const createBackup = (instanceId: string) => invoke<BackupItem>('create_backup', { instanceId })

export const listBackups = (instanceId: string) => invoke<BackupItem[]>('list_backups', { instanceId })

export const restoreBackup = (instanceId: string, backupPath: string) =>
  invoke<void>('restore_backup', { instanceId, backupPath })

export const exportInstanceConfig = (instanceIds: string[]) =>
  invoke<ExportResult>('export_instance_config', { instanceIds })

export const exportCluster = () => invoke<ExportResult>('export_cluster')

export const importInstanceConfig = (path: string) => invoke<ImportResult>('import_instance_config', { path })

export const openInstanceDirectory = (instanceId: string) => invoke<void>('open_instance_directory', { instanceId })
