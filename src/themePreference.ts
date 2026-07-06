import { defaultGlobalSettings } from './data'
import type { GlobalSettings } from './types'

export type ThemeMode = 'dark' | 'light'
export type ThemePreference = GlobalSettings['theme']

export const THEME_PREFERENCE_STORAGE_KEY = 'asa-theme-preference'

const DARK_WINDOW_BACKGROUND = '#020a13'
const LIGHT_WINDOW_BACKGROUND = '#f3f7fb'
const DARK_THEME_COLOR = '#020c17'
const LIGHT_THEME_COLOR = '#f3f7fb'

export function normalizeThemePreference(value: unknown): ThemePreference {
  return value === 'dark' || value === 'light' || value === 'system'
    ? value
    : defaultGlobalSettings.theme
}

export function systemPrefersDark() {
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? true
}

export function resolveThemeMode(themePreference: ThemePreference, systemDark = systemPrefersDark()): ThemeMode {
  if (themePreference === 'system') return systemDark ? 'dark' : 'light'
  return themePreference
}

export function themeWindowBackgroundColor(themeMode: ThemeMode) {
  return themeMode === 'light' ? LIGHT_WINDOW_BACKGROUND : DARK_WINDOW_BACKGROUND
}

export function themeMetaColor(themeMode: ThemeMode) {
  return themeMode === 'light' ? LIGHT_THEME_COLOR : DARK_THEME_COLOR
}

export function currentWindowBackgroundColor(themePreference: ThemePreference) {
  return themeWindowBackgroundColor(resolveThemeMode(themePreference))
}

export function loadCachedThemePreference(): ThemePreference {
  try {
    return normalizeThemePreference(window.localStorage.getItem(THEME_PREFERENCE_STORAGE_KEY))
  } catch {
    return defaultGlobalSettings.theme
  }
}

export function cacheThemePreference(themePreference: ThemePreference) {
  try {
    window.localStorage.setItem(THEME_PREFERENCE_STORAGE_KEY, themePreference)
  } catch {
    // localStorage 不可用时直接忽略，后端 config.toml 仍是权威来源
  }
}
