import {
  THEME_PREFERENCE_STORAGE_KEY,
  normalizeThemePreference,
  resolveThemeMode,
  themeMetaColor,
} from './themePreference'

const cachedTheme = (() => {
  try {
    return normalizeThemePreference(localStorage.getItem(THEME_PREFERENCE_STORAGE_KEY))
  } catch {
    return 'dark'
  }
})()

const resolvedTheme = resolveThemeMode(cachedTheme)
const root = document.documentElement
root.dataset.theme = cachedTheme
root.dataset.resolvedTheme = resolvedTheme
root.style.colorScheme = resolvedTheme
root.classList.toggle('theme-light', resolvedTheme === 'light')
root.classList.toggle('theme-dark', resolvedTheme === 'dark')
document
  .querySelector('meta[name="theme-color"]')
  ?.setAttribute('content', themeMetaColor(resolvedTheme))
