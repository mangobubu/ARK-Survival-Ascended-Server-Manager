import { useCallback, useEffect, useState, type KeyboardEvent } from 'react'
import {
  BgColorsOutlined,
  CloudSyncOutlined,
  DatabaseOutlined,
  FolderOpenOutlined,
  GlobalOutlined,
  InfoCircleOutlined,
  KeyOutlined,
  ReloadOutlined,
  SaveOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { Button, Form, Input, InputNumber, Popover, Radio, Select, Space, Switch, Tag, Typography, message } from 'antd'
import { checkSteamCmd, listWebSecurityBans, unbanWebSecurityIp } from './backendApi'
import { loadGlobalSettings, loadGlobalSettingsFromBackend, saveGlobalSettings } from './globalSettings'
import { isTauriRuntime } from './runtime'
import SettingsWebAccessSection from './SettingsWebAccessSection'
import type { GlobalSettings, WebSecurityBanRecord } from './types'
import {
  closeBehaviorOptions,
  formatShortcutKey,
  normalizeShortcutEvent,
  normalizeWebIpWhitelist,
  type ShortcutParseResult,
  webIpWhitelistSignature,
} from './settingsWindowUtils'

const { Text, Title } = Typography

interface SettingsWindowProps {
  onClose?: () => void
}

type DirectorySettingField = 'steamCmdPath' | 'serverStoragePath' | 'backupStoragePath' | 'webReverseProxyOpenRestyPath'

export default function SettingsWindow({ onClose }: SettingsWindowProps = {}) {
  const [form] = Form.useForm<GlobalSettings>()
  const [messageApi, contextHolder] = message.useMessage()
  const [selectingPath, setSelectingPath] = useState<keyof GlobalSettings | null>(null)
  const [settings, setSettings] = useState<GlobalSettings>(loadGlobalSettings)
  const [saving, setSaving] = useState(false)
  const [securityBans, setSecurityBans] = useState<WebSecurityBanRecord[]>([])
  const [securityBansLoading, setSecurityBansLoading] = useState(false)
  const [securityBansError, setSecurityBansError] = useState('')
  const [unbanningIp, setUnbanningIp] = useState<string | null>(null)
  const watchedWebManagementEnabled = Form.useWatch('webManagementEnabled', form) ?? settings.webManagementEnabled
  const watchedWebPort = Form.useWatch('webServerPort', form) ?? settings.webServerPort
  const watchedReverseProxyEnabled = Form.useWatch('webReverseProxyEnabled', form) ?? settings.webReverseProxyEnabled
  const watchedReverseProxyDomain = Form.useWatch('webReverseProxyDomain', form) ?? settings.webReverseProxyDomain
  const watchedReverseProxyPort = Form.useWatch('webReverseProxyPort', form) ?? settings.webReverseProxyPort
  const watchedHttpsEnabled = Form.useWatch('webHttpsEnabled', form) ?? settings.webHttpsEnabled
  const watchedAcmeEnabled = Form.useWatch('webAcmeAutoIssueEnabled', form) ?? settings.webAcmeAutoIssueEnabled
  const watchedShortcutKey = Form.useWatch('globalToggleShortcutKey', form) ?? settings.globalToggleShortcutKey
  const webSettingsDisabled = !watchedWebManagementEnabled
  const reverseProxySettingsDisabled = webSettingsDisabled || !watchedReverseProxyEnabled
  const httpsSettingsDisabled = reverseProxySettingsDisabled || !watchedHttpsEnabled
  const acmeSettingsDisabled = httpsSettingsDisabled || !watchedAcmeEnabled
  const webPasswordPlaceholder = settings.webAdminPasswordConfigured
    ? '留空则保留当前密码，输入新密码则替换'
    : '建议设置高强度密码'
  const tencentSecretKeyPlaceholder = settings.webAcmeTencentSecretKeyConfigured
    ? '留空则保留当前腾讯云 Secret Key'
    : '请输入腾讯云 Secret Key'
  const trimmedReverseProxyDomain = String(watchedReverseProxyDomain ?? '').trim()
  const reverseProxyUrl = trimmedReverseProxyDomain
    ? `${watchedHttpsEnabled ? 'https' : 'http'}://${trimmedReverseProxyDomain}:${watchedReverseProxyPort}`
    : '配置域名后生成'

  const closeWindow = () => {
    if (onClose) {
      onClose()
      return
    }
    if (isTauriRuntime()) void getCurrentWindow().close()
  }

  const loadSecurityBans = useCallback(async () => {
    setSecurityBansLoading(true)
    setSecurityBansError('')
    try {
      const records = await listWebSecurityBans()
      setSecurityBans(records)
    } catch (error) {
      const detail = String(error)
      setSecurityBansError(detail)
      setSecurityBans([])
    } finally {
      setSecurityBansLoading(false)
    }
  }, [])

  const handleUnbanSecurityIp = async (ip: string) => {
    setUnbanningIp(ip)
    try {
      const result = await unbanWebSecurityIp(ip)
      messageApi.success(result.existed ? `已解封 ${result.ip}` : `${result.ip} 当前没有活动封禁`)
      await loadSecurityBans()
    } catch (error) {
      messageApi.error(`解封失败：${String(error)}`)
    } finally {
      setUnbanningIp(null)
    }
  }

  const handleFinish = async (values: GlobalSettings) => {
    setSaving(true)
    try {
      const normalizedValues: GlobalSettings = {
        ...values,
        webReverseProxyEnabled: values.webManagementEnabled ? values.webReverseProxyEnabled : false,
        webHttpsEnabled: values.webManagementEnabled && values.webReverseProxyEnabled ? values.webHttpsEnabled : false,
        webAcmeAutoIssueEnabled: values.webManagementEnabled && values.webReverseProxyEnabled && values.webHttpsEnabled
          ? values.webAcmeAutoIssueEnabled
          : false,
        webIpWhitelist: normalizeWebIpWhitelist(values.webIpWhitelist),
      }
      let steamCmdPath = normalizedValues.steamCmdPath
      if (normalizedValues.steamCmdPath !== settings.steamCmdPath) {
        const check = await checkSteamCmd(normalizedValues.steamCmdPath)
        if (!check.valid) {
          messageApi.error(`SteamCMD 目录无效：${check.reason ?? '未找到 steamcmd.exe'}`)
          return
        }
        steamCmdPath = check.path
      }
      const next = await saveGlobalSettings({ ...normalizedValues, steamCmdPath })
      setSettings(next)
      const webManagementChanged = normalizedValues.webManagementEnabled !== settings.webManagementEnabled
      const webPortChanged = normalizedValues.webServerPort !== settings.webServerPort
      const reverseProxyChanged = normalizedValues.webReverseProxyEnabled !== settings.webReverseProxyEnabled
        || normalizedValues.webReverseProxyDomain !== settings.webReverseProxyDomain
        || normalizedValues.webReverseProxyPort !== settings.webReverseProxyPort
        || normalizedValues.webReverseProxyOpenRestyPath !== settings.webReverseProxyOpenRestyPath
        || normalizedValues.webHttpsEnabled !== settings.webHttpsEnabled
        || normalizedValues.webAcmeAutoIssueEnabled !== settings.webAcmeAutoIssueEnabled
        || normalizedValues.webAcmeAccountEmail !== settings.webAcmeAccountEmail
        || normalizedValues.webAcmeTencentSecretId !== settings.webAcmeTencentSecretId
        || normalizedValues.webLoginFailureBanThreshold !== settings.webLoginFailureBanThreshold
        || normalizedValues.webLoginFailureBanSeconds !== settings.webLoginFailureBanSeconds
        || webIpWhitelistSignature(normalizedValues.webIpWhitelist) !== webIpWhitelistSignature(settings.webIpWhitelist)
      messageApi.success(
        normalizedValues.webManagementEnabled
          ? webManagementChanged
            ? `全局设置已保存，Web 管理已启动：http://127.0.0.1:${next.webServerPort}`
            : webPortChanged
              ? `全局设置已保存，Web 管理端口已切换到：http://127.0.0.1:${next.webServerPort}`
              : reverseProxyChanged
                ? '全局设置已保存，Web 管理与域名反向代理已应用'
                : '全局设置已保存'
          : webManagementChanged
            ? '全局设置已保存，Web 管理已关闭'
            : '全局设置已保存',
      )
      window.setTimeout(() => {
        closeWindow()
      }, 600)
    } catch (error) {
      messageApi.error(`无法保存全局设置：${String(error)}`)
    } finally {
      setSaving(false)
    }
  }

  useEffect(() => {
    void loadGlobalSettingsFromBackend().then((loaded) => {
      const normalizedLoaded = {
        ...loaded,
        webIpWhitelist: normalizeWebIpWhitelist(loaded.webIpWhitelist),
      }
      setSettings(normalizedLoaded)
      form.setFieldsValue(normalizedLoaded)
    }).catch((error) => {
      messageApi.error(`无法加载全局设置：${String(error)}`)
    })
  }, [form, messageApi])

  useEffect(() => {
    void loadSecurityBans()
  }, [loadSecurityBans])

  const selectDirectory = async (field: DirectorySettingField, label: string) => {
    setSelectingPath(field)

    try {
      if (!isTauriRuntime()) {
        messageApi.info('Web 版无法打开本机目录选择器，请手动输入运行主机上的绝对路径')
        return
      }
      const currentPath = form.getFieldValue(field)
      const selectedPath = await open({
        defaultPath: currentPath || undefined,
        directory: true,
        multiple: false,
        title: `选择${label}`,
      })

      if (selectedPath) {
        if (field === 'steamCmdPath') {
          const check = await checkSteamCmd(selectedPath)
          if (!check.valid) {
            messageApi.error(`SteamCMD 目录无效：${check.reason ?? '未找到 steamcmd.exe'}`)
            return
          }
          form.setFieldValue(field, check.path)
          return
        }
        form.setFieldValue(field, selectedPath)
      }
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      messageApi.error(`无法选择${label}：${detail}`)
    } finally {
      setSelectingPath(null)
    }
  }

  const directoryPicker = (field: DirectorySettingField, label: string, disabled = false) => (
    <Button
      aria-label={`选择${label}`}
      className="settings-path-picker"
      disabled={disabled}
      icon={<FolderOpenOutlined />}
      loading={selectingPath === field}
      onClick={() => void selectDirectory(field, label)}
      size="small"
      type="text"
    />
  )

  const handleShortcutKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    event.preventDefault()
    event.stopPropagation()

    const shortcut: ShortcutParseResult = normalizeShortcutEvent(event)
    if (shortcut.type === 'modifier') return
    if (shortcut.type === 'unsupported') {
      messageApi.warning('该按键不适合作为全局快捷键，请选择字母、数字、F1-F24 或常用功能键')
      return
    }

    form.setFieldValue('globalToggleShortcutKey', shortcut.value)
    messageApi.success(`快捷键已设置为 Ctrl + Alt + ${formatShortcutKey(shortcut.value)}`)
  }

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
                <div className="settings-card__icon"><KeyOutlined /></div>
                <div><h2>窗口与托盘</h2><p>关闭按钮、托盘入口与全局呼出快捷键</p></div>
              </div>
              <div className="settings-card__body settings-card__body--two-column">
                <Form.Item
                  label="窗口关闭行为"
                  name="windowCloseBehavior"
                  tooltip="点击主窗口关闭按钮时采用的处理方式，默认每次询问"
                >
                  <Select options={closeBehaviorOptions} />
                </Form.Item>
                <Form.Item
                  label="呼出/最小化快捷键"
                  name="globalToggleShortcutKey"
                  tooltip="只需要按下第三个自定义按键；保存后实际快捷键固定为 Ctrl + Alt + 该按键"
                  rules={[{ required: true, message: '请设置呼出/最小化快捷键' }]}
                  getValueProps={(value) => ({ value: `Ctrl + Alt + ${formatShortcutKey(value)}` })}
                >
                  <Input
                    className="settings-shortcut-input"
                    prefix={<KeyOutlined />}
                    readOnly
                    onKeyDown={handleShortcutKeyDown}
                    onPaste={(event) => event.preventDefault()}
                    placeholder="请直接按第三个自定义按键"
                  />
                </Form.Item>
                <div className="settings-toggle-row settings-toggle-row--wide">
                  <div>
                    <strong>
                      隐藏托盘图标
                      <Popover
                        content={(
                          <div className="settings-popover-note">
                            <p>开启后，只有窗口最小化到托盘面板时，托盘图标才会隐藏。</p>
                            <p>如果已最小化到托盘且隐藏图标，可使用快捷键 Ctrl + Alt + {formatShortcutKey(watchedShortcutKey)} 呼出应用。</p>
                            <p>主窗口显示时，托盘图标会自动恢复，便于继续从托盘菜单操作。</p>
                          </div>
                        )}
                        placement="right"
                        title="隐藏规则说明"
                      >
                        <InfoCircleOutlined className="settings-inline-help" />
                      </Popover>
                    </strong>
                    <span>仅隐藏最小化到托盘面板后的图标；主窗口显示时仍保留托盘入口</span>
                  </div>
                  <Form.Item name="hideTrayIconWhenMinimized" valuePropName="checked"><Switch /></Form.Item>
                </div>
              </div>
            </section>

            <section className="settings-card">
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

            <SettingsWebAccessSection
              form={form}
              settings={settings}
              watchedWebManagementEnabled={watchedWebManagementEnabled}
              watchedWebPort={watchedWebPort}
              webSettingsDisabled={webSettingsDisabled}
              reverseProxySettingsDisabled={reverseProxySettingsDisabled}
              httpsSettingsDisabled={httpsSettingsDisabled}
              acmeSettingsDisabled={acmeSettingsDisabled}
              webPasswordPlaceholder={webPasswordPlaceholder}
              tencentSecretKeyPlaceholder={tencentSecretKeyPlaceholder}
              trimmedReverseProxyDomain={trimmedReverseProxyDomain}
              reverseProxyUrl={reverseProxyUrl}
              securityBans={securityBans}
              securityBansLoading={securityBansLoading}
              securityBansError={securityBansError}
              unbanningIp={unbanningIp}
              directoryPicker={directoryPicker}
              onLoadSecurityBans={() => void loadSecurityBans()}
              onUnbanSecurityIp={(ip) => void handleUnbanSecurityIp(ip)}
            />

          </div>
        </main>

        <footer className="settings-footer">
          <Text type="secondary">设置仅保存在当前设备</Text>
          <Space>
            <Button onClick={closeWindow}>取消</Button>
            <Button loading={saving} type="primary" icon={<SaveOutlined />} htmlType="submit">保存设置</Button>
          </Space>
        </footer>
      </Form>
    </div>
  )
}
