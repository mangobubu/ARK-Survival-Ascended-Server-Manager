import { useEffect, useState } from 'react'
import {
  BgColorsOutlined,
  CloudSyncOutlined,
  DatabaseOutlined,
  FolderOpenOutlined,
  GlobalOutlined,
  ReloadOutlined,
  SaveOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { Button, Form, Input, InputNumber, Radio, Select, Space, Switch, Typography, message } from 'antd'
import { defaultGlobalSettings } from './data'
import type { GlobalSettings } from './types'

const { Text, Title } = Typography

export default function SettingsWindow() {
  const [form] = Form.useForm<GlobalSettings>()
  const [messageApi, contextHolder] = message.useMessage()
  const [selectingPath, setSelectingPath] = useState<keyof GlobalSettings | null>(null)
  const [settings] = useState<GlobalSettings>(() => {
    try {
      const saved = JSON.parse(localStorage.getItem('asa-global-settings') ?? '{}') as Record<string, unknown>
      delete saved.steamApiKey
      delete saved.steamCmdLoginMode
      delete saved.steamCmdUsername
      delete saved.steamCmdPassword
      return { ...defaultGlobalSettings, ...saved } as GlobalSettings
    } catch {
      return defaultGlobalSettings
    }
  })

  useEffect(() => {
    form.setFieldsValue(settings)
    localStorage.setItem('asa-global-settings', JSON.stringify(settings))
  }, [settings, form])

  const handleFinish = (values: GlobalSettings) => {
    localStorage.setItem('asa-global-settings', JSON.stringify(values))
    messageApi.success('全局设置已保存')

    window.setTimeout(() => {
      getCurrentWindow().close()
    }, 600)
  }

  const selectDirectory = async (
    field: 'steamCmdPath' | 'serverStoragePath' | 'backupStoragePath',
    label: string,
  ) => {
    setSelectingPath(field)

    try {
      const currentPath = form.getFieldValue(field)
      const selectedPath = await open({
        defaultPath: currentPath || undefined,
        directory: true,
        multiple: false,
        title: `选择${label}`,
      })

      if (selectedPath) {
        form.setFieldValue(field, selectedPath)
      }
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      messageApi.error(`无法选择${label}：${detail}`)
    } finally {
      setSelectingPath(null)
    }
  }

  const directoryPicker = (
    field: 'steamCmdPath' | 'serverStoragePath' | 'backupStoragePath',
    label: string,
  ) => (
    <Button
      aria-label={`选择${label}`}
      className="settings-path-picker"
      icon={<FolderOpenOutlined />}
      loading={selectingPath === field}
      onClick={() => void selectDirectory(field, label)}
      size="small"
      type="text"
    />
  )

  return (
    <div className="settings-window">
      {contextHolder}

      <header className="settings-header">
        <div className="settings-header__mark"><SettingOutlined /></div>
        <div>
          <Title level={3}>全局设置</Title>
          <Text>配置管理器界面、运行路径与自动维护策略</Text>
        </div>
      </header>

      <Form
        className="settings-form"
        form={form}
        layout="vertical"
        onFinish={handleFinish}
        initialValues={settings}
        requiredMark={false}
      >
        <main className="settings-content">
          <div className="settings-grid">
            <section className="settings-card">
              <div className="settings-card__heading settings-card__heading--compact">
                <div className="settings-card__icon"><BgColorsOutlined /></div>
                <div><h2>界面偏好</h2><p>语言与显示主题</p></div>
              </div>
              <div className="settings-card__body settings-card__body--two-column">
                <Form.Item label={<span><GlobalOutlined /> 应用语言</span>} name="language">
                  <Select options={[
                    { value: 'zh-CN', label: '简体中文' },
                    { value: 'en-US', label: 'English' },
                  ]} />
                </Form.Item>
                <Form.Item label="应用主题" name="theme">
                  <Radio.Group optionType="button" buttonStyle="solid" options={[
                    { value: 'dark', label: '暗色' },
                    { value: 'light', label: '亮色' },
                    { value: 'system', label: '跟随系统' },
                  ]} />
                </Form.Item>
              </div>
            </section>

            <section className="settings-card">
              <div className="settings-card__heading settings-card__heading--compact">
                <div className="settings-card__icon"><DatabaseOutlined /></div>
                <div><h2>运行与存储路径</h2><p>SteamCMD、实例与备份文件位置</p></div>
              </div>
              <div className="settings-card__body settings-card__body--two-column">
                <Form.Item label="SteamCMD 目录" name="steamCmdPath" tooltip="SteamCMD 可执行文件所在目录">
                  <Input
                    prefix={<CloudSyncOutlined />}
                    suffix={directoryPicker('steamCmdPath', 'SteamCMD 目录')}
                    placeholder="例如：C:\\SteamCMD"
                  />
                </Form.Item>
                <Form.Item label="服务器存储目录" name="serverStoragePath" tooltip="所有服务器实例默认安装到此目录">
                  <Input suffix={directoryPicker('serverStoragePath', '服务器存储目录')} placeholder="例如：D:\\ASA-Servers" />
                </Form.Item>
                <Form.Item label="备份存储目录" name="backupStoragePath" tooltip="自动备份文件的存放位置">
                  <Input suffix={directoryPicker('backupStoragePath', '备份存储目录')} placeholder="例如：D:\\ASA-Backups" />
                </Form.Item>
              </div>
            </section>

            <section className="settings-card settings-card--full">
              <div className="settings-card__heading settings-card__heading--compact">
                <div className="settings-card__icon"><ReloadOutlined /></div>
                <div><h2>自动化与维护</h2><p>设置启动、故障恢复和备份保留策略</p></div>
              </div>
              <div className="settings-automation-grid">
                <div className="settings-toggle-row">
                  <div><strong>启动时检查更新</strong><span>管理器启动后检查服务端版本</span></div>
                  <Form.Item name="autoUpdateOnStart" valuePropName="checked"><Switch /></Form.Item>
                </div>
                <div className="settings-toggle-row">
                  <div><strong>崩溃后自动重启</strong><span>检测到异常退出时恢复实例</span></div>
                  <Form.Item name="autoRestartOnCrash" valuePropName="checked"><Switch /></Form.Item>
                </div>
                <Form.Item label="自动备份保留数量" name="maxBackupRetention" tooltip="自动备份最多保留的文件数">
                  <InputNumber min={1} max={100} addonAfter="个备份" />
                </Form.Item>
              </div>
            </section>
          </div>
        </main>

        <footer className="settings-footer">
          <Text type="secondary">设置仅保存在当前设备</Text>
          <Space>
            <Button onClick={() => getCurrentWindow().close()}>取消</Button>
            <Button type="primary" icon={<SaveOutlined />} htmlType="submit">保存设置</Button>
          </Space>
        </footer>
      </Form>
    </div>
  )
}
