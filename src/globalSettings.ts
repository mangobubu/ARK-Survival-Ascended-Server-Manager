import { emit, listen } from '@tauri-apps/api/event'
import { defaultGlobalSettings } from './data'
import { getSettings, saveSettings } from './backendApi'
import type { GlobalSettings } from './types'
import { SETTINGS_CHANGED_EVENT } from './windowEvents'

const LOCAL_SETTINGS_EVENT = 'asa-global-settings-local'

export function loadGlobalSettings(): GlobalSettings {
  return defaultGlobalSettings
}

export async function loadGlobalSettingsFromBackend(): Promise<GlobalSettings> {
  return { ...defaultGlobalSettings, ...await getSettings() }
}

export async function saveGlobalSettings(settings: GlobalSettings) {
  const saved = await saveSettings(settings)
  window.dispatchEvent(new CustomEvent<GlobalSettings>(LOCAL_SETTINGS_EVENT, { detail: saved }))
  void emit(SETTINGS_CHANGED_EVENT, saved).catch((error) => {
    console.error('同步全局设置失败', error)
  })
  return saved
}

export async function ensureGlobalStorageDirectories(settings: GlobalSettings) {
  await saveSettings(settings)
}

export function subscribeGlobalSettings(onChange: (settings: GlobalSettings) => void) {
  let disposed = false
  let unlistenTauri: (() => void) | undefined

  const handleLocal = (event: Event) => {
    onChange((event as CustomEvent<GlobalSettings>).detail)
  }
  const handleFocus = () => {
    void loadGlobalSettingsFromBackend().then(onChange).catch((error) => {
      console.error('刷新全局设置失败', error)
    })
  }

  window.addEventListener(LOCAL_SETTINGS_EVENT, handleLocal)
  window.addEventListener('focus', handleFocus)

  void listen<GlobalSettings>(SETTINGS_CHANGED_EVENT, (event) => onChange(event.payload)).then((unlisten) => {
    if (disposed) unlisten()
    else unlistenTauri = unlisten
  }).catch((error) => {
    console.error('监听全局设置同步失败', error)
  })

  return () => {
    disposed = true
    unlistenTauri?.()
    window.removeEventListener(LOCAL_SETTINGS_EVENT, handleLocal)
    window.removeEventListener('focus', handleFocus)
  }
}
