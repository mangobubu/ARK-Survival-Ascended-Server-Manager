import { StrictMode, Suspense, lazy, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { createRoot } from 'react-dom/client'
import { setTheme as setTauriAppTheme } from '@tauri-apps/api/app'
import { ConfigProvider, theme } from 'antd'
import enUS from 'antd/locale/en_US'
import zhCN from 'antd/locale/zh_CN'
import { getSettings } from './backendApi'
import { loadGlobalSettings, loadGlobalSettingsFromBackend, subscribeGlobalSettings } from './globalSettings'
import { isTauriRuntime } from './runtime'
import { resolveThemeMode, systemPrefersDark, themeMetaColor } from './themePreference'
import './styles.css'
import type { GlobalSettings } from './types'

const AddInstanceWindow = lazy(() => import('./AddInstanceWindow'))
const App = lazy(() => import('./App'))
const LoginPage = lazy(() => import('./LoginPage'))
const RconWindow = lazy(() => import('./RconWindow'))
const SettingsWindow = lazy(() => import('./SettingsWindow'))

const isSettingsWindow = new URLSearchParams(window.location.search).get('window') === 'settings'
const isAddInstanceWindow = new URLSearchParams(window.location.search).get('window') === 'add-instance'
const isRconWindow = new URLSearchParams(window.location.search).get('window') === 'rcon'
document.documentElement.classList.toggle('settings-document', isSettingsWindow)
document.documentElement.classList.toggle('child-window-document', isSettingsWindow || isAddInstanceWindow || isRconWindow)

function applyDocumentTheme(settings: GlobalSettings, themeMode: 'dark' | 'light') {
  document.documentElement.lang = settings.language
  document.documentElement.dataset.theme = settings.theme
  document.documentElement.dataset.resolvedTheme = themeMode
  document.documentElement.style.colorScheme = themeMode
  document.documentElement.classList.toggle('theme-light', themeMode === 'light')
  document.documentElement.classList.toggle('theme-dark', themeMode === 'dark')
  document.querySelector('meta[name="theme-color"]')?.setAttribute('content', themeMetaColor(themeMode))
}

function applyNativeThemePreference(themePreference: GlobalSettings['theme']) {
  if (!isTauriRuntime()) return

  void setTauriAppTheme(themePreference === 'system' ? null : themePreference).catch((error) => {
    console.error('同步桌面端原生主题失败', error)
  })
}

function RouteLoading({ text = '正在加载界面...' }: { text?: string }) {
  return <div className="web-login-loading">{text}</div>
}

function Root() {
  const [settings, setSettings] = useState<GlobalSettings>(loadGlobalSettings)
  const [systemDark, setSystemDark] = useState(systemPrefersDark)
  const [webAuthenticated, setWebAuthenticated] = useState(() => isTauriRuntime())
  const [checkingWebAuth, setCheckingWebAuth] = useState(!isTauriRuntime())
  const themeMode = resolveThemeMode(settings.theme, systemDark)
  const needsWebLogin = !isTauriRuntime() && (!webAuthenticated || checkingWebAuth)

  useEffect(() => {
    if (!isTauriRuntime() && (!webAuthenticated || checkingWebAuth)) return
    const unsubscribe = subscribeGlobalSettings(setSettings)
    void loadGlobalSettingsFromBackend().then(setSettings).catch((error) => {
      console.error('加载全局设置失败', error)
    })
    return unsubscribe
  }, [checkingWebAuth, webAuthenticated])

  useEffect(() => {
    const media = window.matchMedia?.('(prefers-color-scheme: dark)')
    if (!media) return
    const handleChange = () => setSystemDark(media.matches)
    handleChange()
    if (typeof media.addEventListener === 'function') {
      media.addEventListener('change', handleChange)
      return () => media.removeEventListener('change', handleChange)
    }
    media.addListener(handleChange)
    return () => media.removeListener(handleChange)
  }, [])

  useLayoutEffect(() => {
    applyDocumentTheme(settings, themeMode)
  }, [settings, themeMode])

  useEffect(() => {
    applyNativeThemePreference(settings.theme)
  }, [settings.theme])

  useEffect(() => {
    document.documentElement.classList.toggle('web-login-document', needsWebLogin)
    return () => document.documentElement.classList.remove('web-login-document')
  }, [needsWebLogin])

  useEffect(() => {
    if (isTauriRuntime()) return
    let disposed = false
    void getSettings()
      .then(() => {
        if (!disposed) setWebAuthenticated(true)
      })
      .catch((error) => {
        console.error('Web 登录状态已失效', error)
        if (!disposed) setWebAuthenticated(false)
      })
      .finally(() => {
        if (!disposed) setCheckingWebAuth(false)
      })
    return () => {
      disposed = true
    }
  }, [])

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
      {needsWebLogin ? (
        checkingWebAuth ? (
          <RouteLoading text="正在校验 Web 登录状态..." />
        ) : (
          <Suspense fallback={<RouteLoading text="正在加载登录界面..." />}>
            <LoginPage onAuthenticated={() => setWebAuthenticated(true)} />
          </Suspense>
        )
      ) : (
        <Suspense fallback={<RouteLoading />}>
          {isSettingsWindow ? (
            <SettingsWindow />
          ) : isAddInstanceWindow ? (
            <AddInstanceWindow />
          ) : isRconWindow ? (
            <RconWindow />
          ) : (
            <App />
          )}
        </Suspense>
      )}
    </ConfigProvider>
  )
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
)
