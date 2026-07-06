import { useCallback, useEffect, useState, type KeyboardEvent } from 'react'
import {
  BgColorsOutlined,
  CloudSyncOutlined,
  DatabaseOutlined,
  DeleteOutlined,
  FolderOpenOutlined,
  GlobalOutlined,
  InfoCircleOutlined,
  KeyOutlined,
  LinkOutlined,
  PlusOutlined,
  ReloadOutlined,
  SaveOutlined,
  SafetyCertificateOutlined,
  SettingOutlined,
  UnlockOutlined,
} from '@ant-design/icons'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { Button, Form, Input, InputNumber, Popover, Radio, Select, Space, Switch, Tag, Typography, message } from 'antd'
import { checkSteamCmd, listWebSecurityBans, unbanWebSecurityIp } from './backendApi'
import { loadGlobalSettings, loadGlobalSettingsFromBackend, saveGlobalSettings } from './globalSettings'
import { isTauriRuntime } from './runtime'
import type { GlobalSettings, WebIpWhitelistEntry, WebSecurityBanRecord } from './types'

const { Text, Title } = Typography

const closeBehaviorOptions = [
  { value: 'askEveryTime', label: '每次询问' },
  { value: 'minimizeToTray', label: '最小化托盘' },
  { value: 'exitApp', label: '退出应用' },
]

const chinaMainlandIpToken = 'CN_MAINLAND'

const defaultWebIpWhitelistEntry: WebIpWhitelistEntry = {
  value: chinaMainlandIpToken,
  group: '默认',
  note: '内置中国大陆 IPv4 CIDR',
}

function normalizeWebIpWhitelist(value: unknown): WebIpWhitelistEntry[] {
  if (!Array.isArray(value)) return [defaultWebIpWhitelistEntry]
  const seen = new Set<string>()
  const normalized = value
    .map((item): WebIpWhitelistEntry | null => {
      if (typeof item === 'string') {
        const raw = item.trim()
        if (!raw) return null
        return {
          value: raw.toUpperCase() === chinaMainlandIpToken ? chinaMainlandIpToken : raw,
          group: '',
          note: '',
        }
      }
      if (item && typeof item === 'object') {
        const candidate = item as Partial<WebIpWhitelistEntry>
        const raw = String(candidate.value ?? '').trim()
        if (!raw) return null
        return {
          value: raw.toUpperCase() === chinaMainlandIpToken ? chinaMainlandIpToken : raw,
          group: String(candidate.group ?? '').trim(),
          note: String(candidate.note ?? '').trim(),
        }
      }
      return null
    })
    .filter((item): item is WebIpWhitelistEntry => Boolean(item))
    .filter((item) => {
      if (seen.has(item.value)) return false
      seen.add(item.value)
      return true
    })
  return normalized.length > 0 ? normalized : [defaultWebIpWhitelistEntry]
}

function isValidIpv4(ip: string) {
  const parts = ip.split('.')
  return parts.length === 4 && parts.every((part) => {
    if (!/^\d{1,3}$/.test(part)) return false
    const value = Number(part)
    return value >= 0 && value <= 255
  })
}

function isValidWebIpWhitelistEntry(entry: string) {
  if (entry.toUpperCase() === chinaMainlandIpToken) return true
  if (isValidIpv4(entry)) return true
  const [ip, prefix, extra] = entry.split('/')
  if (extra !== undefined || !ip || prefix === undefined) return false
  const prefixValue = Number(prefix)
  return isValidIpv4(ip) && /^\d{1,2}$/.test(prefix) && prefixValue >= 0 && prefixValue <= 32
}

function webIpWhitelistSignature(value: unknown) {
  return JSON.stringify(normalizeWebIpWhitelist(value))
}

function formatBanSource(source: string) {
  const labels: Record<string, string> = {
    login: '登录失败',
    ua: '异常 UA',
    body: '危险请求体',
    path: '路径探测',
    rate: '频率限制',
    security: '安全策略',
  }
  return labels[source] ?? source
}

function formatRemainingSeconds(seconds: number) {
  if (seconds <= 0) return '即将过期'
  if (seconds < 60) return `${seconds} 秒`
  const minutes = Math.floor(seconds / 60)
  const restSeconds = seconds % 60
  if (minutes < 60) return restSeconds > 0 ? `${minutes} 分 ${restSeconds} 秒` : `${minutes} 分`
  const hours = Math.floor(minutes / 60)
  const restMinutes = minutes % 60
  return restMinutes > 0 ? `${hours} 小时 ${restMinutes} 分` : `${hours} 小时`
}

function formatBanTime(timestamp: number) {
  if (!timestamp) return '未知时间'
  return new Date(timestamp).toLocaleString('zh-CN', { hour12: false })
}

type ShortcutParseResult =
  | { type: 'key'; value: string }
  | { type: 'modifier' }
  | { type: 'unsupported' }

function normalizeShortcutEvent(event: KeyboardEvent<HTMLInputElement>): ShortcutParseResult {
  const code = event.code || ''
  const key = event.key || ''
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(key)) return { type: 'modifier' }
  if (/^Key[A-Z]$/.test(code)) return { type: 'key', value: code.replace('Key', '') }
  if (/^Digit[0-9]$/.test(code)) return { type: 'key', value: code.replace('Digit', '') }
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(key.toUpperCase())) return { type: 'key', value: key.toUpperCase() }

  const namedKeys: Record<string, string> = {
    Escape: 'ESC',
    ' ': 'SPACE',
    Spacebar: 'SPACE',
    ArrowUp: 'UP',
    ArrowDown: 'DOWN',
    ArrowLeft: 'LEFT',
    ArrowRight: 'RIGHT',
    Home: 'HOME',
    End: 'END',
    PageUp: 'PAGEUP',
    PageDown: 'PAGEDOWN',
    Insert: 'INSERT',
    Delete: 'DELETE',
  }
  const namedKey = namedKeys[key]
  return namedKey ? { type: 'key', value: namedKey } : { type: 'unsupported' }
}

function formatShortcutKey(key?: string) {
  if (!key) return 'A'
  const labels: Record<string, string> = {
    ESC: 'Esc',
    SPACE: 'Space',
    UP: '↑',
    DOWN: '↓',
    LEFT: '←',
    RIGHT: '→',
    PAGEUP: 'PageUp',
    PAGEDOWN: 'PageDown',
    INSERT: 'Insert',
    DELETE: 'Delete',
  }
  return labels[key] ?? key
}

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
  const watchedShortcutKey = Form.useWatch('globalToggleShortcutKey', form) ?? settings.globalToggleShortcutKey
  const webSettingsDisabled = !watchedWebManagementEnabled
  const reverseProxySettingsDisabled = webSettingsDisabled || !watchedReverseProxyEnabled
  const webPasswordPlaceholder = settings.webAdminPasswordConfigured
    ? '留空则保留当前密码，输入新密码则替换'
    : '建议设置高强度密码'
  const trimmedReverseProxyDomain = String(watchedReverseProxyDomain ?? '').trim()
  const reverseProxyUrl = trimmedReverseProxyDomain
    ? `http://${trimmedReverseProxyDomain}:${watchedReverseProxyPort}`
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

            <section className="settings-card settings-card--full">
              <div className="settings-card__heading settings-card__heading--compact">
                <div className="settings-card__icon"><GlobalOutlined /></div>
                <div><h2>Web 访问</h2><p>浏览器版本入口与监听端口</p></div>
              </div>
              <div className="settings-card__body settings-card__body--two-column">
                <div className="settings-toggle-row settings-toggle-row--wide">
                  <div>
                    <strong>启用 Web 管理</strong>
                    <span>关闭时不启动浏览器管理端，并锁定下面的 Web 访问、登录与反代字段；开启后点击保存才会启动。</span>
                  </div>
                  <Form.Item name="webManagementEnabled" valuePropName="checked">
                    <Switch
                      onChange={(enabled) => {
                        if (!enabled) form.setFieldValue('webReverseProxyEnabled', false)
                      }}
                    />
                  </Form.Item>
                </div>
                <Form.Item
                  label="Web 访问端口"
                  name="webServerPort"
                  tooltip="Web 管理启用后监听该端口；修改保存后会即时重启 Web 管理服务"
                  rules={[
                    { required: true, message: '请输入 Web 访问端口' },
                    { type: 'number', min: 1024, max: 65535, message: '端口必须在 1024-65535 之间' },
                  ]}
                >
                  <InputNumber disabled={webSettingsDisabled} min={1024} max={65535} addonBefore="127.0.0.1:" style={{ width: '100%' }} />
                </Form.Item>
                <div className="settings-web-port-note">
                  <Text type="secondary">{watchedWebManagementEnabled ? '保存后访问地址：' : '启用并保存后访问地址：'}</Text>
                  <Text code copyable={watchedWebManagementEnabled ? { text: `http://127.0.0.1:${watchedWebPort}` } : false}>http://127.0.0.1:{watchedWebPort}</Text>
                  <Text type="secondary">关闭时不会监听端口；开启或修改端口后，点击保存才会启动或切换 Web 管理服务。</Text>
                </div>
                <div className="settings-toggle-row settings-toggle-row--wide">
                  <div>
                    <strong>
                      启用域名反向代理
                      <Popover
                        content={(
                          <div className="settings-popover-note">
                            <p>使用 OpenResty 在公开端口监听域名，再转发到本机 127.0.0.1 的应用 Web 端口。</p>
                            <p>Lua 安全网关会处理动态封禁、异常 UA、危险请求体、频率与路径组合风险。</p>
                            <p>需要你自行把域名 DNS 解析到这台 Windows 主机；本功能不会修改公网 DNS。</p>
                            <p>启用后，Web 后端会拒绝非“域名:公开端口”的 Host 请求。</p>
                          </div>
                        )}
                        placement="right"
                        title="域名访问说明"
                      >
                        <InfoCircleOutlined className="settings-inline-help" />
                      </Popover>
                    </strong>
                    <span>使用 OpenResty for Windows，便于后续接入 Redis 黑白名单与更复杂 Lua 风控</span>
                  </div>
                  <Form.Item name="webReverseProxyEnabled" valuePropName="checked"><Switch disabled={webSettingsDisabled} /></Form.Item>
                </div>
                <Form.Item
                  label="公开访问域名"
                  name="webReverseProxyDomain"
                  tooltip="只填写域名主机名，例如 ark.example.com；不要填写 http://、端口或路径"
                  rules={[
                    ({ getFieldValue }) => ({
                      validator(_, value) {
                        if (!getFieldValue('webManagementEnabled') || !getFieldValue('webReverseProxyEnabled')) return Promise.resolve()
                        const domain = String(value ?? '').trim()
                        if (!domain) return Promise.reject(new Error('启用反向代理时必须填写访问域名'))
                        if (/(:\/\/|[\\/:*])/.test(domain)) return Promise.reject(new Error('域名不能包含协议、端口、路径或通配符'))
                        if (/^(localhost|(\d{1,3}\.){3}\d{1,3})$/i.test(domain)) return Promise.reject(new Error('请填写真实域名，不要填写 localhost 或 IP 地址'))
                        const labelPattern = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/i
                        if (!domain.replace(/\.$/, '').split('.').every((label) => labelPattern.test(label))) {
                          return Promise.reject(new Error('域名格式无效，只支持字母、数字、短横线和点号'))
                        }
                        return Promise.resolve()
                      },
                    }),
                  ]}
                >
                  <Input disabled={reverseProxySettingsDisabled} prefix={<LinkOutlined />} placeholder="例如：ark.example.com" />
                </Form.Item>
                <Form.Item
                  label="反代公开端口"
                  name="webReverseProxyPort"
                  tooltip="OpenResty 对外监听的 HTTP 端口；不能与应用 Web 内部端口相同"
                  rules={[
                    ({ getFieldValue }) => ({
                      validator(_, value) {
                        if (!getFieldValue('webManagementEnabled') || !getFieldValue('webReverseProxyEnabled')) return Promise.resolve()
                        if (typeof value !== 'number') return Promise.reject(new Error('请输入反代公开端口'))
                        if (value < 1 || value > 65535) return Promise.reject(new Error('端口必须在 1-65535 之间'))
                        if (value === getFieldValue('webServerPort')) return Promise.reject(new Error('公开端口不能与应用 Web 内部端口相同'))
                        return Promise.resolve()
                      },
                    }),
                  ]}
                >
                  <InputNumber disabled={reverseProxySettingsDisabled} min={1} max={65535} addonBefore="0.0.0.0:" style={{ width: '100%' }} />
                </Form.Item>
                <Form.Item
                  label="OpenResty 安装目录"
                  name="webReverseProxyOpenRestyPath"
                  tooltip="填写 OpenResty 解压目录或其中的 nginx.exe 路径；应用会使用独立前缀目录生成专用配置和 Lua 安全脚本"
                  rules={[
                    ({ getFieldValue }) => ({
                      validator(_, value) {
                        if (!getFieldValue('webManagementEnabled') || !getFieldValue('webReverseProxyEnabled')) return Promise.resolve()
                        if (String(value ?? '').trim()) return Promise.resolve()
                        return Promise.reject(new Error('启用反向代理时必须填写 OpenResty 安装目录或 nginx.exe 路径'))
                      },
                    }),
                  ]}
                >
                  <Input
                    disabled={reverseProxySettingsDisabled}
                    prefix={<CloudSyncOutlined />}
                    suffix={directoryPicker('webReverseProxyOpenRestyPath', 'OpenResty 安装目录', reverseProxySettingsDisabled)}
                    placeholder="例如：C:\\openresty-1.25.3.2-win64 或 C:\\openresty-1.25.3.2-win64\\nginx.exe"
                  />
                </Form.Item>
                <Form.Item
                  label="登录失败封禁阈值"
                  name="webLoginFailureBanThreshold"
                  tooltip="同一 IP 在登录接口返回 401 的累计次数达到阈值后，OpenResty Lua 会临时封禁该 IP"
                  rules={[
                    { required: true, message: '请输入登录失败封禁阈值' },
                    { type: 'number', min: 1, max: 100, message: '登录失败封禁阈值必须在 1-100 次之间' },
                  ]}
                >
                  <InputNumber disabled={reverseProxySettingsDisabled} min={1} max={100} addonAfter="次" style={{ width: '100%' }} />
                </Form.Item>
                <Form.Item
                  label="登录失败封禁时长"
                  name="webLoginFailureBanSeconds"
                  tooltip="同一 IP 触发登录失败封禁后的封禁秒数；默认 1800 秒"
                  rules={[
                    { required: true, message: '请输入登录失败封禁时长' },
                    { type: 'number', min: 1, max: 86400, message: '登录失败封禁时长必须在 1-86400 秒之间' },
                  ]}
                >
                  <InputNumber disabled={reverseProxySettingsDisabled} min={1} max={86400} addonAfter="秒" style={{ width: '100%' }} />
                </Form.Item>
                <Form.Item
                  label="验证码字符池"
                  name="webCaptchaCharset"
                  tooltip="登录失败后生成字符串验证码时使用的候选字符；会写入 config.toml，可手动编辑"
                  rules={[
                    { required: true, message: '请输入验证码字符池' },
                    { max: 128, message: '验证码字符池不能超过 128 个字符' },
                    {
                      validator(_, value) {
                        const visibleLength = String(value ?? '').trim().replace(/\s/g, '').length
                        if (visibleLength < 2) return Promise.reject(new Error('验证码字符池至少需要 2 个可见字符'))
                        return Promise.resolve()
                      },
                    },
                  ]}
                >
                  <Input disabled={webSettingsDisabled} placeholder="例如：ABCDEFGHJKLMNPQRSTUVWXYZ23456789" />
                </Form.Item>
                <Form.Item
                  label="验证码字符数量"
                  name="webCaptchaLength"
                  tooltip="每张验证码随机抽取的字符数量；会写入 config.toml"
                  rules={[
                    { required: true, message: '请输入验证码字符数量' },
                    { type: 'number', min: 1, max: 12, message: '验证码字符数量必须在 1-12 之间' },
                  ]}
                >
                  <InputNumber disabled={webSettingsDisabled} min={1} max={12} addonAfter="个字符" style={{ width: '100%' }} />
                </Form.Item>
                <Form.Item
                  label="验证码字体大小"
                  name="webCaptchaFontSize"
                  tooltip="SVG 验证码文字的字号；会写入 config.toml"
                  rules={[
                    { required: true, message: '请输入验证码字体大小' },
                    { type: 'number', min: 18, max: 56, message: '验证码字体大小必须在 18-56 之间' },
                  ]}
                >
                  <InputNumber disabled={webSettingsDisabled} min={18} max={56} addonAfter="px" style={{ width: '100%' }} />
                </Form.Item>
                <Form.Item
                  label="验证码杂点数量"
                  name="webCaptchaNoisePoints"
                  tooltip="SVG 验证码中的随机杂点数量；0 表示不添加杂点，会写入 config.toml"
                  rules={[
                    { required: true, message: '请输入验证码杂点数量' },
                    { type: 'number', min: 0, max: 120, message: '验证码杂点数量必须在 0-120 之间' },
                  ]}
                >
                  <InputNumber disabled={webSettingsDisabled} min={0} max={120} addonAfter="个" style={{ width: '100%' }} />
                </Form.Item>
                <div className="settings-web-captcha-note">
                  <Text type="secondary">验证码策略：</Text>
                  <Text>首次 Web 登录失败后，同一访问来源在后续一小时内登录都必须先通过字符串验证码；验证码参数会持久化到 <code>config.toml</code>。</Text>
                </div>
                <div className="settings-web-whitelist-editor">
                  <div className="settings-web-whitelist-editor__head">
                    <div>
                      <strong>准入 IP 白名单</strong>
                      <span>支持 CN_MAINLAND、单个 IPv4 或 IPv4 CIDR，并可填写分组与备注</span>
                    </div>
                    <Popover
                      content={(
                        <div className="settings-popover-note">
                          <p>白名单条目会持久化保存到全局设置；OpenResty 运行目录下的 CIDR 文件只是根据这些设置派生生成。</p>
                          <p>如果手动修改运行目录里的 CIDR 文件，下次应用设置或重启反代时会被全局设置覆盖。</p>
                          <p>内网与环回地址始终放行，避免本机管理被误拦截。</p>
                        </div>
                      )}
                      placement="right"
                      title="白名单持久化说明"
                    >
                      <InfoCircleOutlined className="settings-inline-help" />
                    </Popover>
                  </div>
                  <Form.List name="webIpWhitelist">
                    {(fields, { add, remove }) => (
                      <div className="settings-web-whitelist-list">
                        {fields.map((field) => (
                          <div className="settings-web-whitelist-row" key={field.key}>
                            <Form.Item
                              {...field}
                              label="IP / CIDR"
                              name={[field.name, 'value']}
                              rules={[
                                {
                                  validator(_, value) {
                                    const entry = String(value ?? '').trim()
                                    if (!entry) return Promise.reject(new Error('请输入白名单 IP、CIDR 或 CN_MAINLAND'))
                                    if (!isValidWebIpWhitelistEntry(entry)) {
                                      return Promise.reject(new Error('仅支持 CN_MAINLAND、IPv4 或 IPv4 CIDR'))
                                    }
                                    return Promise.resolve()
                                  },
                                },
                              ]}
                            >
                              <Input disabled={reverseProxySettingsDisabled} placeholder="CN_MAINLAND 或 203.0.113.0/24" />
                            </Form.Item>
                            <Form.Item
                              {...field}
                              label="分组"
                              name={[field.name, 'group']}
                              rules={[{ max: 32, message: '分组不能超过 32 个字符' }]}
                            >
                              <Input disabled={reverseProxySettingsDisabled} placeholder="例如：大陆默认 / 运维 / 临时" />
                            </Form.Item>
                            <Form.Item
                              {...field}
                              label="备注"
                              name={[field.name, 'note']}
                              rules={[{ max: 120, message: '备注不能超过 120 个字符' }]}
                            >
                              <Input disabled={reverseProxySettingsDisabled} placeholder="例如：办公室固定出口 IP" />
                            </Form.Item>
                            <Button
                              danger
                              disabled={reverseProxySettingsDisabled || fields.length <= 1}
                              icon={<DeleteOutlined />}
                              onClick={() => remove(field.name)}
                            >
                              删除
                            </Button>
                          </div>
                        ))}
                        <Button
                          disabled={reverseProxySettingsDisabled}
                          icon={<PlusOutlined />}
                          onClick={() => add({ value: '', group: '', note: '' })}
                        >
                          添加白名单条目
                        </Button>
                      </div>
                    )}
                  </Form.List>
                </div>
                <div className="settings-web-proxy-note">
                  <Text type="secondary">域名入口：</Text>
                  <Text code copyable={trimmedReverseProxyDomain ? { text: reverseProxyUrl } : false}>{reverseProxyUrl}</Text>
                  <Text type="secondary">保存后会生成独立 OpenResty 配置与 Lua 安全脚本，未知 Host 默认 403；未命中 IP 白名单的公网 IP 默认拦截。</Text>
                </div>
                <div className="settings-web-ban-panel">
                  <div className="settings-web-ban-panel__head">
                    <div>
                      <strong>登录失败封禁记录</strong>
                      <span>读取当前 OpenResty Lua 共享内存中的动态封禁，可手动解封 IP</span>
                    </div>
                    <Button disabled={reverseProxySettingsDisabled} icon={<ReloadOutlined />} loading={securityBansLoading} onClick={() => void loadSecurityBans()} size="small">
                      刷新
                    </Button>
                  </div>
                  {securityBansError ? (
                    <Text type="secondary">封禁记录暂不可用：{securityBansError}</Text>
                  ) : securityBans.length === 0 ? (
                    <Text type="secondary">暂无活动封禁记录。启用 OpenResty 反代并触发登录失败封禁后会在这里显示。</Text>
                  ) : (
                    <div className="settings-web-ban-list">
                      {securityBans.map((record) => (
                        <div className="settings-web-ban-row" key={`${record.ip}-${record.bannedAtMs}`}>
                          <div>
                            <Space wrap size={6}>
                              <Text code>{record.ip}</Text>
                              <Tag color={record.source === 'login' ? 'red' : 'orange'}>{formatBanSource(record.source)}</Tag>
                              <Text type="secondary">剩余 {formatRemainingSeconds(record.remainingSeconds)}</Text>
                            </Space>
                            <p>{record.reason} · 封禁时间：{formatBanTime(record.bannedAtMs)}</p>
                          </div>
                          <Button
                            icon={<UnlockOutlined />}
                            disabled={reverseProxySettingsDisabled}
                            loading={unbanningIp === record.ip}
                            onClick={() => void handleUnbanSecurityIp(record.ip)}
                            size="small"
                          >
                            解封
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
                <Form.Item
                  label="Web 管理员账号"
                  name="webAdminUsername"
                  tooltip="仅用于浏览器 Web 端登录；桌面端不需要登录"
                  rules={[
                    { max: 64, message: '账号不能超过 64 个字符' },
                    ({ getFieldValue }) => ({
                      validator(_, value) {
                        if (!getFieldValue('webAdminPassword') || String(value ?? '').trim()) return Promise.resolve()
                        if (!getFieldValue('webManagementEnabled')) return Promise.resolve()
                        return Promise.reject(new Error('设置密码时管理员账号不能为空'))
                      },
                    }),
                  ]}
                >
                  <Input disabled={webSettingsDisabled} prefix={<SafetyCertificateOutlined />} placeholder="例如：admin" />
                </Form.Item>
                <Form.Item
                  label="Web 管理员密码"
                  name="webAdminPassword"
                  tooltip="后端不会回传明文密码；已设置密码时留空保存会保留旧密码"
                  rules={[{ max: 128, message: '密码不能超过 128 个字符' }]}
                >
                  <Input.Password disabled={webSettingsDisabled} autoComplete="new-password" placeholder={webPasswordPlaceholder} />
                </Form.Item>
                <div className="settings-web-auth-note">
                  <Text type="secondary">登录页提示：</Text>
                  <Text>Web 端会提示用户回到桌面端「全局设置 → Web 访问」部署管理员账号和密码。</Text>
                  {settings.webAdminPasswordConfigured && (
                    <Text type="secondary">当前已配置管理员密码。密码输入框留空时只保存其他设置，不会覆盖旧密码。</Text>
                  )}
                </div>
              </div>
            </section>

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
