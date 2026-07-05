/// <reference types="vite/client" />

interface Window {
  __TAURI_INTERNALS__?: unknown
}

interface ImportMetaEnv {
  readonly VITE_ASA_API_BASE_URL?: string
}
