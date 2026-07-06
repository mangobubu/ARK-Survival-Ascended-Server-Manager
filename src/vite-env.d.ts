/// <reference types="vite/client" />

interface Window {
  __ASA_RUNTIME__?: 'desktop' | 'web'
  __TAURI_INTERNALS__?: unknown
}

interface ImportMetaEnv {
  readonly VITE_ASA_API_BASE_URL?: string
}
