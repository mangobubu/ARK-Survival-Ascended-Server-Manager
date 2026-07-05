import { getSettings, saveSettings } from './backendApi'
import { defaultGlobalSettings } from './data'
import { SETTINGS_CHANGED_EVENT, subscribeBackendEvent } from './syncEvents'
import type { GlobalSettings } from './types'

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
  return saved
}

export async function ensureGlobalStorageDirectories(settings: GlobalSettings) {
  await saveSettings(settings)
}

export function subscribeGlobalSettings(onChange: (settings: GlobalSettings) => void) {
  let disposed = false

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

  const unsubscribeBackend = subscribeBackendEvent(SETTINGS_CHANGED_EVENT, (settings) => {
    if (!disposed) onChange(settings)
  })

  return () => {
    disposed = true
    unsubscribeBackend()
    window.removeEventListener(LOCAL_SETTINGS_EVENT, handleLocal)
    window.removeEventListener('focus', handleFocus)
  }
}
