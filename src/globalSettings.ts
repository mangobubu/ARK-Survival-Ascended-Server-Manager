import { emit, listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { defaultGlobalSettings } from './data'
import type { GlobalSettings } from './types'

const STORAGE_KEY = 'asa-global-settings'
const SETTINGS_EVENT = 'asa-global-settings-changed'
const LOCAL_SETTINGS_EVENT = 'asa-global-settings-local'

export function loadGlobalSettings(): GlobalSettings {
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Record<string, unknown>
    delete saved.steamApiKey
    delete saved.steamCmdLoginMode
    delete saved.steamCmdUsername
    delete saved.steamCmdPassword
    return { ...defaultGlobalSettings, ...saved } as GlobalSettings
  } catch {
    return defaultGlobalSettings
  }
}

export function saveGlobalSettings(settings: GlobalSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
  window.dispatchEvent(new CustomEvent<GlobalSettings>(LOCAL_SETTINGS_EVENT, { detail: settings }))
  void emit(SETTINGS_EVENT, settings).catch((error) => {
    console.error('同步全局设置失败', error)
  })
}

export function ensureGlobalStorageDirectories(settings: GlobalSettings) {
  return invoke<void>('ensure_storage_directories', {
    serverStoragePath: settings.serverStoragePath,
    backupStoragePath: settings.backupStoragePath,
  })
}

export function subscribeGlobalSettings(onChange: (settings: GlobalSettings) => void) {
  let disposed = false
  let unlistenTauri: (() => void) | undefined

  const handleLocal = (event: Event) => {
    onChange((event as CustomEvent<GlobalSettings>).detail)
  }
  const handleStorage = (event: StorageEvent) => {
    if (event.key === STORAGE_KEY) onChange(loadGlobalSettings())
  }
  const handleFocus = () => onChange(loadGlobalSettings())

  window.addEventListener(LOCAL_SETTINGS_EVENT, handleLocal)
  window.addEventListener('storage', handleStorage)
  window.addEventListener('focus', handleFocus)

  void listen<GlobalSettings>(SETTINGS_EVENT, (event) => onChange(event.payload)).then((unlisten) => {
    if (disposed) unlisten()
    else unlistenTauri = unlisten
  }).catch((error) => {
    console.error('监听全局设置同步失败', error)
  })

  return () => {
    disposed = true
    unlistenTauri?.()
    window.removeEventListener(LOCAL_SETTINGS_EVENT, handleLocal)
    window.removeEventListener('storage', handleStorage)
    window.removeEventListener('focus', handleFocus)
  }
}
