import { useEffect, useState, type KeyboardEvent } from 'react'
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
  SafetyCertificateOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { Button, Form, Input, InputNumber, Popover, Radio, Select, Space, Switch, Typography, message } from 'antd'
import { checkSteamCmd } from './backendApi'
import { loadGlobalSettings, loadGlobalSettingsFromBackend, saveGlobalSettings } from './globalSettings'
import { isTauriRuntime } from './runtime'
import type { GlobalSettings } from './types'

const { Text, Title } = Typography

const closeBehaviorOptions = [
  { value: 'askEveryTime', label: '每次询问' },
  { value: 'minimizeToTray', label: '最小化托盘' },
  { value: 'exitApp', label: '退出应用' },
]

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

export default function SettingsWindow({ onClose }: SettingsWindowProps = {}) {
  const [form] = Form.useForm<GlobalSettings>()
  const [messageApi, contextHolder] = message.useMessage()
  const [selectingPath, setSelectingPath] = useState<keyof GlobalSettings | null>(null)
  const [settings, setSettings] = useState<GlobalSettings>(loadGlobalSettings)
  const [saving, setSaving] = useState(false)
  const watchedWebPort = Form.useWatch('webServerPort', form) ?? settings.webServerPort
  const watchedShortcutKey = Form.useWatch('globalToggleShortcutKey', form) ?? settings.globalToggleShortcutKey

  const closeWindow = () => {
    if (onClose) {
      onClose()
      return
    }
    if (isTauriRuntime()) void getCurrentWindow().close()
  }

  const handleFinish = async (values: GlobalSettings) => {
    setSaving(true)
    try {
      let steamCmdPath = values.steamCmdPath
      if (values.steamCmdPath !== settings.steamCmdPath) {
        const check = await checkSteamCmd(values.steamCmdPath)
        if (!check.valid) {
          messageApi.error(`SteamCMD 目录无效：${check.reason ?? '未找到 steamcmd.exe'}`)
          return
        }
        steamCmdPath = check.path
      }
      const next = await saveGlobalSettings({ ...values, steamCmdPath })
      setSettings(next)
      messageApi.success(values.webServerPort !== settings.webServerPort ? '全局设置已保存，Web 端口将在重启应用后生效' : '全局设置已保存')
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
      setSettings(loaded)
      form.setFieldsValue(loaded)
    }).catch((error) => {
      messageApi.error(`无法加载全局设置：${String(error)}`)
    })
  }, [form, messageApi])

  const selectDirectory = async (
    field: 'steamCmdPath' | 'serverStoragePath' | 'backupStoragePath',
    label: string,
  ) => {
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
                <div className="settings-card__icon"><GlobalOutlined /></div>
                <div><h2>Web 访问</h2><p>浏览器版本入口与监听端口</p></div>
              </div>
              <div className="settings-card__body settings-card__body--two-column">
                <Form.Item
                  label="Web 访问端口"
                  name="webServerPort"
                  tooltip="应用启动时读取该端口；修改保存后需要重启应用才会生效"
                  rules={[
                    { required: true, message: '请输入 Web 访问端口' },
                    { type: 'number', min: 1024, max: 65535, message: '端口必须在 1024-65535 之间' },
                  ]}
                >
                  <InputNumber min={1024} max={65535} addonBefore="127.0.0.1:" style={{ width: '100%' }} />
                </Form.Item>
                <div className="settings-web-port-note">
                  <Text type="secondary">当前/重启后访问地址：</Text>
                  <Text code copyable={{ text: `http://127.0.0.1:${watchedWebPort}` }}>http://127.0.0.1:{watchedWebPort}</Text>
                  <Text type="secondary">端口修改保存后不会立即迁移当前 Web 服务，请重启应用后生效。</Text>
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
                        return Promise.reject(new Error('设置密码时管理员账号不能为空'))
                      },
                    }),
                  ]}
                >
                  <Input prefix={<SafetyCertificateOutlined />} placeholder="例如：admin" />
                </Form.Item>
                <Form.Item
                  label="Web 管理员密码"
                  name="webAdminPassword"
                  tooltip="保存后 Web 端所有 API、实时事件与操作都需要先登录"
                  rules={[{ max: 128, message: '密码不能超过 128 个字符' }]}
                >
                  <Input.Password autoComplete="new-password" placeholder="建议设置高强度密码" />
                </Form.Item>
                <div className="settings-web-auth-note">
                  <Text type="secondary">登录页提示：</Text>
                  <Text>Web 端会提示用户回到桌面端「全局设置 → Web 访问」部署管理员账号和密码。</Text>
                </div>
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
            <Button onClick={closeWindow}>取消</Button>
            <Button loading={saving} type="primary" icon={<SaveOutlined />} htmlType="submit">保存设置</Button>
          </Space>
        </footer>
      </Form>
    </div>
  )
}
