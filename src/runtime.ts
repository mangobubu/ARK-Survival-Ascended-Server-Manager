import { isTauri as tauriIsTauri } from '@tauri-apps/api/core'

type TauriGlobal = typeof globalThis & {
  __TAURI_INTERNALS__?: unknown
  __TAURI__?: unknown
  isTauri?: boolean
}

export function isTauriRuntime() {
  const tauriGlobal = globalThis as TauriGlobal
  return tauriIsTauri() || Boolean(tauriGlobal.isTauri || tauriGlobal.__TAURI_INTERNALS__ || tauriGlobal.__TAURI__)
}

export function getWebApiBaseUrl() {
  return import.meta.env.VITE_ASA_API_BASE_URL?.replace(/\/$/, '') ?? window.location.origin
}

export function isWebRuntime() {
  return !isTauriRuntime()
}
