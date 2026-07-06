type TauriGlobal = typeof globalThis & {
  __ASA_RUNTIME__?: unknown
  __TAURI_INTERNALS__?: {
    invoke?: unknown
  }
  isTauri?: boolean
}

export type RuntimeMode = 'desktop' | 'web'

function hasTauriInvokeBridge(tauriGlobal: TauriGlobal) {
  return typeof tauriGlobal.__TAURI_INTERNALS__?.invoke === 'function'
}

export function getRuntimeMode(): RuntimeMode {
  const tauriGlobal = globalThis as TauriGlobal
  const declaredRuntime = tauriGlobal.__ASA_RUNTIME__

  if (declaredRuntime === 'web') return 'web'
  if (declaredRuntime === 'desktop') {
    return hasTauriInvokeBridge(tauriGlobal) ? 'desktop' : 'web'
  }

  return hasTauriInvokeBridge(tauriGlobal) ? 'desktop' : 'web'
}

export function isTauriRuntime() {
  return getRuntimeMode() === 'desktop'
}

export function getWebApiBaseUrl() {
  return import.meta.env.VITE_ASA_API_BASE_URL?.replace(/\/$/, '') ?? window.location.origin
}

export function isWebRuntime() {
  return !isTauriRuntime()
}
