import { StrictMode, useEffect, useMemo, useState } from 'react'
import { createRoot } from 'react-dom/client'
import { ConfigProvider, theme } from 'antd'
import enUS from 'antd/locale/en_US'
import zhCN from 'antd/locale/zh_CN'
import AddInstanceWindow from './AddInstanceWindow'
import App from './App'
import RconWindow from './RconWindow'
import SettingsWindow from './SettingsWindow'
import { loadGlobalSettings, loadGlobalSettingsFromBackend, subscribeGlobalSettings } from './globalSettings'
import './styles.css'
import type { GlobalSettings } from './types'

const isSettingsWindow = new URLSearchParams(window.location.search).get('window') === 'settings'
const isAddInstanceWindow = new URLSearchParams(window.location.search).get('window') === 'add-instance'
const isRconWindow = new URLSearchParams(window.location.search).get('window') === 'rcon'
document.documentElement.classList.toggle('settings-document', isSettingsWindow)
document.documentElement.classList.toggle('child-window-document', isSettingsWindow || isAddInstanceWindow || isRconWindow)

function systemPrefersDark() {
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? true
}

function resolveThemeMode(settings: GlobalSettings, systemDark: boolean) {
  if (settings.theme === 'system') return systemDark ? 'dark' : 'light'
  return settings.theme
}

function Root() {
  const [settings, setSettings] = useState<GlobalSettings>(loadGlobalSettings)
  const [systemDark, setSystemDark] = useState(systemPrefersDark)
  const themeMode = resolveThemeMode(settings, systemDark)

  useEffect(() => {
    const unsubscribe = subscribeGlobalSettings(setSettings)
    void loadGlobalSettingsFromBackend().then(setSettings).catch((error) => {
      console.error('加载全局设置失败', error)
    })
    return unsubscribe
  }, [])

  useEffect(() => {
    const media = window.matchMedia?.('(prefers-color-scheme: dark)')
    if (!media) return
    const handleChange = () => setSystemDark(media.matches)
    media.addEventListener('change', handleChange)
    return () => media.removeEventListener('change', handleChange)
  }, [])

  useEffect(() => {
    document.documentElement.lang = settings.language
    document.documentElement.classList.toggle('theme-light', themeMode === 'light')
    document.documentElement.classList.toggle('theme-dark', themeMode === 'dark')
  }, [settings.language, themeMode])

  const appTheme = useMemo(() => {
    const isDark = themeMode === 'dark'
    return {
      algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
      token: {
        colorPrimary: '#00a9ff',
        colorInfo: '#00a9ff',
        colorSuccess: '#16d17c',
        colorWarning: '#ffb020',
        colorError: '#ff3158',
        colorBgBase: isDark ? '#020b14' : '#f3f7fb',
        colorBgContainer: isDark ? '#061523' : '#ffffff',
        colorBgElevated: isDark ? '#0a1b2b' : '#ffffff',
        colorBorder: isDark ? '#14334d' : '#cbd8e3',
        colorText: isDark ? '#dbe8f4' : '#172433',
        colorTextSecondary: isDark ? '#7f95a8' : '#5a6b7d',
        borderRadius: 5,
        fontFamily: 'Inter, "Microsoft YaHei UI", "PingFang SC", sans-serif',
        controlHeight: 34,
      },
      components: {
        Button: {
          defaultBg: isDark ? '#071a2b' : '#ffffff',
          defaultBorderColor: isDark ? '#17405f' : '#bfd0dd',
        },
        Input: {
          activeBg: isDark ? '#06131f' : '#ffffff',
          hoverBg: isDark ? '#06131f' : '#ffffff',
          colorBgContainer: isDark ? '#06131f' : '#ffffff',
        },
        InputNumber: {
          activeBg: isDark ? '#06131f' : '#ffffff',
          hoverBg: isDark ? '#06131f' : '#ffffff',
          colorBgContainer: isDark ? '#06131f' : '#ffffff',
        },
        Select: { selectorBg: isDark ? '#06131f' : '#ffffff' },
        Tabs: {
          itemColor: isDark ? '#91a4b5' : '#5f7080',
          itemSelectedColor: '#10b8ff',
          inkBarColor: '#10b8ff',
        },
        Table: {
          headerBg: isDark ? '#06121e' : '#eef5fb',
          rowHoverBg: isDark ? '#092139' : '#eaf6ff',
          borderColor: isDark ? '#102b41' : '#d6e3ee',
        },
        Switch: { colorPrimary: '#138dff', colorPrimaryHover: '#29a4ff' },
      },
    }
  }, [themeMode])

  return (
    <ConfigProvider locale={settings.language === 'en-US' ? enUS : zhCN} theme={appTheme}>
      {isSettingsWindow ? <SettingsWindow /> : isAddInstanceWindow ? <AddInstanceWindow /> : isRconWindow ? <RconWindow /> : <App />}
    </ConfigProvider>
  )
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
)
