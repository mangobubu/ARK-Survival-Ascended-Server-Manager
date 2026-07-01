import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { ConfigProvider, theme } from 'antd'
import zhCN from 'antd/locale/zh_CN'
import App from './App'
import SettingsWindow from './SettingsWindow'
import './styles.css'

const isSettingsWindow = new URLSearchParams(window.location.search).get('window') === 'settings'
document.documentElement.classList.toggle('settings-document', isSettingsWindow)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ConfigProvider
      locale={zhCN}
      theme={{
        algorithm: theme.darkAlgorithm,
        token: {
          colorPrimary: '#00a9ff',
          colorInfo: '#00a9ff',
          colorSuccess: '#16d17c',
          colorWarning: '#ffb020',
          colorError: '#ff3158',
          colorBgBase: '#020b14',
          colorBgContainer: '#061523',
          colorBgElevated: '#0a1b2b',
          colorBorder: '#14334d',
          colorText: '#dbe8f4',
          colorTextSecondary: '#7f95a8',
          borderRadius: 5,
          fontFamily: 'Inter, "Microsoft YaHei UI", "PingFang SC", sans-serif',
          controlHeight: 34,
        },
        components: {
          Button: { defaultBg: '#071a2b', defaultBorderColor: '#17405f' },
          Input: { activeBg: '#06131f', hoverBg: '#06131f', colorBgContainer: '#06131f' },
          InputNumber: { activeBg: '#06131f', hoverBg: '#06131f', colorBgContainer: '#06131f' },
          Select: { selectorBg: '#06131f' },
          Tabs: { itemColor: '#91a4b5', itemSelectedColor: '#10b8ff', inkBarColor: '#10b8ff' },
          Table: { headerBg: '#06121e', rowHoverBg: '#092139', borderColor: '#102b41' },
          Switch: { colorPrimary: '#138dff', colorPrimaryHover: '#29a4ff' },
        },
      }}
    >
      {isSettingsWindow ? <SettingsWindow /> : <App />}
    </ConfigProvider>
  </StrictMode>,
)
