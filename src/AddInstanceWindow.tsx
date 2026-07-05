import { useEffect, useMemo, useRef, useState } from 'react'
import {
  ApartmentOutlined,
  CheckCircleOutlined,
  CloseOutlined,
  CloudServerOutlined,
  DatabaseOutlined,
  DeploymentUnitOutlined,
  FieldTimeOutlined,
  FolderOpenOutlined,
  PlusOutlined,
  SafetyCertificateOutlined,
} from '@ant-design/icons'
import { open } from '@tauri-apps/plugin-dialog'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { Button, Form, Input, InputNumber, Radio, Select, Space, Switch, Tooltip, Typography, message } from 'antd'
import { checkInstancePort, createInstance, readServerDirectoryConfig } from './backendApi'
import { defaultConfig, defaultGlobalSettings, serverMapOptions } from './data'
import { isTauriRuntime } from './runtime'
import type {
  AddInstancePayload,
  ImportedServerConfigPreview,
  InstanceCreatedEvent,
  InstancePortKind,
} from './types'

const { Text, Title } = Typography

const formatMapLabel = (map: { name: string; zhName: string }) => `${map.zhName}(${map.name})`

const PORT_FIELDS = [
  { name: 'gamePort', label: '游戏端口' },
  { name: 'queryPort', label: '查询端口' },
  { name: 'rconPort', label: 'RCON 端口' },
] as const

type PortField = InstancePortKind
type PortCheckStatus = 'idle' | 'checking' | 'available' | 'unavailable'

interface PortCheckState {
  status: PortCheckStatus
  port?: number
  message?: string
}

const createInitialPortChecks = (): Record<PortField, PortCheckState> => ({
  gamePort: { status: 'idle' },
  queryPort: { status: 'idle' },
  rconPort: { status: 'idle' },
})

function isValidPort(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 1024 && value <= 65535
}

interface AddInstanceFormValues {
  name: string
  mapCode: string
  mode: 'PvE' | 'PvP'
  gamePort: number
  queryPort: number
  rconPort: number
  maxPlayers: number
  installPath: string
  clusterId: string
  serverPassword: string
  adminPassword: string
  autoInstall: boolean
  description: string
}

interface AddInstanceWindowProps {
  initialParams?: URLSearchParams
  onCreated?: (payload: InstanceCreatedEvent) => void
  onClose?: () => void
}

function readNumberParam(params: URLSearchParams, name: string, fallback: number) {
  const value = Number(params.get(name))
  return Number.isFinite(value) && value > 0 ? value : fallback
}

function readTextParam(params: URLSearchParams, name: string, fallback: string) {
  return params.get(name) ?? fallback
}

export default function AddInstanceWindow({ initialParams, onCreated, onClose }: AddInstanceWindowProps = {}) {
  const [form] = Form.useForm<AddInstanceFormValues>()
  const [messageApi, contextHolder] = message.useMessage()
  const [submitting, setSubmitting] = useState(false)
  const [selectingDirectory, setSelectingDirectory] = useState(false)
  const [importedPreview, setImportedPreview] = useState<ImportedServerConfigPreview | null>(null)
  const [portChecks, setPortChecks] = useState<Record<PortField, PortCheckState>>(createInitialPortChecks)
  const portCheckRequestRef = useRef(0)
  const params = useMemo(() => initialParams ?? new URLSearchParams(window.location.search), [initialParams])

  const closeCurrentWindow = async () => {
    if (onClose) {
      onClose()
      return
    }
    if (!isTauriRuntime()) return
    try {
      await WebviewWindow.getCurrent().close()
    } catch (error) {
      console.error('关闭添加实例窗口失败', error)
      messageApi.error(`无法关闭窗口：${String(error)}`)
    }
  }

  const sequence = readNumberParam(params, 'index', 10)
  const defaultName = 'ASA-' + String(sequence).padStart(2, '0')
  const initialValues = useMemo<AddInstanceFormValues>(() => ({
    name: defaultName,
    mapCode: 'TheIsland_WP',
    mode: 'PvE',
    gamePort: readNumberParam(params, 'gamePort', 7867),
    queryPort: readNumberParam(params, 'queryPort', 27105),
    rconPort: readNumberParam(params, 'rconPort', 32340),
    maxPlayers: defaultConfig.maxPlayers,
    installPath: readTextParam(params, 'serverRoot', defaultGlobalSettings.serverStoragePath) + '\\' + defaultName,
    clusterId: defaultConfig.clusterId,
    serverPassword: defaultConfig.serverPassword,
    adminPassword: defaultConfig.adminPassword,
    autoInstall: true,
    description: '',
  }), [defaultName, params])

  const watchedValues = Form.useWatch([], form) ?? initialValues
  const selectedMap = serverMapOptions.find((item) => item.code === watchedValues.mapCode) ?? serverMapOptions[0]
  const portsReady = PORT_FIELDS.every(({ name }) => {
    const check = portChecks[name]
    return check.status === 'available' && check.port === watchedValues[name]
  })
  const hasUnavailablePort = PORT_FIELDS.some(({ name }) => portChecks[name].status === 'unavailable')
  const portPlanText = portsReady ? '可用' : hasUnavailablePort ? '不可用' : '检查中'
  const importPlanText = importedPreview ? '已读取' : '可选'
  const readyCount = [
    watchedValues.name,
    watchedValues.mapCode,
    watchedValues.gamePort,
    watchedValues.queryPort,
    watchedValues.installPath,
  ].filter(Boolean).length

  useEffect(() => {
    const requestId = ++portCheckRequestRef.current
    const portValues = PORT_FIELDS.map(({ name, label }) => ({ name, label, port: watchedValues[name] }))
    const occurrences = new Map<number, number>()

    for (const item of portValues) {
      if (!isValidPort(item.port)) continue
      occurrences.set(item.port, (occurrences.get(item.port) ?? 0) + 1)
    }

    const checkable = portValues.filter((item) => (
      isValidPort(item.port) && (occurrences.get(item.port) ?? 0) === 1
    ))

    setPortChecks((current) => {
      const next = { ...current }
      for (const item of portValues) {
        if (!isValidPort(item.port)) {
          next[item.name] = { status: 'idle' }
        } else if ((occurrences.get(item.port) ?? 0) > 1) {
          next[item.name] = {
            status: 'unavailable',
            port: item.port,
            message: '当前表单内端口重复',
          }
        } else {
          next[item.name] = { status: 'checking', port: item.port }
        }
      }
      return next
    })

    if (checkable.length === 0) return

    const timer = window.setTimeout(() => {
      void (async () => {
        const results = await Promise.all(checkable.map(async (item) => {
          try {
            return {
              ...item,
              result: await checkInstancePort(item.port, item.name),
              error: null,
            }
          } catch (error) {
            return {
              ...item,
              result: null,
              error,
            }
          }
        }))

        if (portCheckRequestRef.current !== requestId) return

        const suggestedValues: Partial<Pick<AddInstanceFormValues, PortField>> = {}
        for (const item of results) {
          if (item.result?.exists && item.result.suggestedPort && item.result.suggestedPort !== item.port) {
            suggestedValues[item.name] = item.result.suggestedPort
          }
        }

        if (Object.keys(suggestedValues).length > 0) {
          form.setFieldsValue(suggestedValues)
          setPortChecks((current) => {
            const next = { ...current }
            for (const [name, port] of Object.entries(suggestedValues) as Array<[PortField, number]>) {
              next[name] = { status: 'checking', port }
            }
            return next
          })
          return
        }

        setPortChecks((current) => {
          const next = { ...current }
          for (const item of results) {
            if (form.getFieldValue(item.name) !== item.port) continue
            if (item.error) {
              next[item.name] = {
                status: 'unavailable',
                port: item.port,
                message: `端口检查失败：${String(item.error)}`,
              }
            } else if (item.result?.available) {
              next[item.name] = { status: 'available', port: item.port }
            } else {
              next[item.name] = {
                status: 'unavailable',
                port: item.port,
                message: item.result?.reason ?? `${item.label}不可用`,
              }
            }
          }
          return next
        })
      })()
    }, 220)

    return () => window.clearTimeout(timer)
  }, [form, watchedValues.gamePort, watchedValues.queryPort, watchedValues.rconPort])

  const renderPortAddon = (field: PortField) => {
    const check = portChecks[field]
    if (check.status === 'available') {
      return <span className="port-check-addon port-check-addon--available" role="img" aria-label="端口可用" title="端口可用">✅</span>
    }
    if (check.status === 'checking') {
      return <span className="port-check-addon port-check-addon--checking" title="正在检查端口">…</span>
    }
    if (check.status === 'unavailable') {
      return <span className="port-check-addon port-check-addon--unavailable" title={check.message}>!</span>
    }
    return <span className="port-check-addon port-check-addon--idle" />
  }

  const portItemStatus = (field: PortField) => {
    const check = portChecks[field]
    if (check.status === 'checking') return { validateStatus: 'validating' as const }
    if (check.status === 'unavailable') {
      return {
        validateStatus: 'error' as const,
        help: check.message,
      }
    }
    return {}
  }

  const clearImportedPreview = () => {
    if (importedPreview) {
      setImportedPreview(null)
    }
  }

  const selectServerDirectory = async () => {
    setSelectingDirectory(true)
    try {
      if (!isTauriRuntime()) {
        messageApi.info('Web 版无法打开本机目录选择器，请手动输入运行主机上的 ASA 服务端文件夹路径')
        return
      }
      const selectedPath = await open({
        defaultPath: form.getFieldValue('installPath') || undefined,
        directory: true,
        multiple: false,
        title: '选择 ASA 服务端文件夹',
      })
      if (!selectedPath) return

      const imported = await readServerDirectoryConfig(selectedPath)
      const importedValues: Partial<AddInstanceFormValues> = {
        installPath: imported.installPath || selectedPath,
        autoInstall: true,
      }

      if (imported.name) importedValues.name = imported.name
      if (imported.mapCode && serverMapOptions.some((item) => item.code === imported.mapCode)) {
        importedValues.mapCode = imported.mapCode
      }
      if (imported.mode) importedValues.mode = imported.mode
      if (imported.gamePort) importedValues.gamePort = imported.gamePort
      if (imported.queryPort) importedValues.queryPort = imported.queryPort
      if (imported.rconPort) importedValues.rconPort = imported.rconPort
      if (imported.maxPlayers) importedValues.maxPlayers = imported.maxPlayers
      if (imported.clusterId) importedValues.clusterId = imported.clusterId
      if (imported.serverPassword !== null) importedValues.serverPassword = imported.serverPassword
      if (imported.adminPassword) importedValues.adminPassword = imported.adminPassword

      form.setFieldsValue(importedValues)
      setImportedPreview(imported.foundFiles.length > 0 ? imported : null)

      if (imported.foundFiles.length > 0) {
        const modText = imported.mods.length > 0 ? `，包含 ${imported.mods.length} 个 MOD` : ''
        messageApi.success(`已读取 ${imported.foundFiles.length} 个配置文件${modText}，创建后将执行更新/校验`)
      } else {
        messageApi.info('已选择目录；新安装的服务端通常要启动一次或保存配置后才会生成可导入配置，创建后将先执行更新/校验')
      }
    } catch (error) {
      setImportedPreview(null)
      messageApi.error(`读取服务端目录失败：${String(error)}`)
    } finally {
      setSelectingDirectory(false)
    }
  }

  const handleFinish = async (values: AddInstanceFormValues) => {
    const portsReadyForValues = PORT_FIELDS.every(({ name }) => {
      const check = portChecks[name]
      return check.status === 'available' && check.port === values[name]
    })
    if (!portsReadyForValues) {
      messageApi.warning('请等待端口可用性检查完成')
      return
    }

    const map = serverMapOptions.find((item) => item.code === values.mapCode) ?? serverMapOptions[0]
    const payload: AddInstancePayload = {
      name: values.name.trim(),
      map: map.name,
      mapCode: map.code,
      mode: values.mode,
      status: 'stopped',
      gamePort: values.gamePort,
      queryPort: values.queryPort,
      players: 0,
      maxPlayers: values.maxPlayers,
      installPath: values.installPath.trim(),
      rconPort: values.rconPort,
      clusterId: values.clusterId.trim(),
      serverPassword: (values.serverPassword ?? '').trim(),
      adminPassword: (values.adminPassword ?? '').trim(),
      autoInstall: values.autoInstall,
      description: values.description.trim(),
      importedConfig: importedPreview?.config,
      importedMods: importedPreview?.mods,
    }

    setSubmitting(true)
    try {
      const instance = await createInstance(payload)
      const eventPayload: InstanceCreatedEvent = { instance, autoInstall: payload.autoInstall }
      if (onCreated) {
        onCreated(eventPayload)
      }
      messageApi.success('实例已创建')
      window.setTimeout(() => {
        void closeCurrentWindow()
      }, 420)
    } catch (error) {
      messageApi.error(`无法添加实例：${String(error)}`)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="add-instance-window">
      {contextHolder}
      <header className="add-instance-header">
        <div className="add-instance-header__mark"><PlusOutlined /></div>
        <div>
          <Title level={3}>添加服务器实例</Title>
          <Text>新实例将以停止状态加入主窗口列表</Text>
        </div>
      </header>

      <Form
        className="add-instance-form"
        form={form}
        initialValues={initialValues}
        layout="vertical"
        onFinish={handleFinish}
        requiredMark={false}
      >
        <main className="add-instance-content">
          <section className="add-instance-main">
            <div className="add-instance-section">
              <div className="add-instance-section__title"><CloudServerOutlined /> 基础信息</div>
              <div className="add-instance-two-column">
                <Form.Item
                  label="实例名称"
                  name="name"
                  rules={[{ required: true, message: '请输入实例名称' }, { max: 28, message: '实例名称不能超过 28 个字符' }]}
                >
                  <Input placeholder="例如：ASA-10" />
                </Form.Item>
                <Form.Item label="运行模式" name="mode" rules={[{ required: true }]}>
                  <Radio.Group optionType="button" buttonStyle="solid" options={[
                    { value: 'PvE', label: 'PvE' },
                    { value: 'PvP', label: 'PvP' },
                  ]} />
                </Form.Item>
                <Form.Item label="地图" name="mapCode" rules={[{ required: true, message: '请选择地图' }]}>
                  <Select
                    options={serverMapOptions.map((item) => ({ value: item.code, label: formatMapLabel(item) }))}
                  />
                </Form.Item>
                <Form.Item
                  label="最大玩家数"
                  name="maxPlayers"
                  rules={[{ required: true, message: '请输入最大玩家数' }]}
                >
                  <InputNumber min={1} max={250} addonAfter="人" />
                </Form.Item>
              </div>
            </div>

            <div className="add-instance-section">
              <div className="add-instance-section__title"><DeploymentUnitOutlined /> 端口与集群</div>
              <div className="add-instance-three-column">
                <Form.Item
                  label="游戏端口"
                  name="gamePort"
                  rules={[{ required: true, message: '请输入游戏端口' }]}
                  {...portItemStatus('gamePort')}
                >
                  <InputNumber min={1024} max={65535} addonAfter={renderPortAddon('gamePort')} />
                </Form.Item>
                <Form.Item
                  label="查询端口"
                  name="queryPort"
                  rules={[{ required: true, message: '请输入查询端口' }]}
                  {...portItemStatus('queryPort')}
                >
                  <InputNumber min={1024} max={65535} addonAfter={renderPortAddon('queryPort')} />
                </Form.Item>
                <Form.Item
                  label="RCON 端口"
                  name="rconPort"
                  rules={[{ required: true, message: '请输入 RCON 端口' }]}
                  {...portItemStatus('rconPort')}
                >
                  <InputNumber min={1024} max={65535} addonAfter={renderPortAddon('rconPort')} />
                </Form.Item>
              </div>
              <Form.Item label="集群 ID" name="clusterId" rules={[{ required: true, message: '请输入集群 ID' }]}>
                <Input prefix={<ApartmentOutlined />} />
              </Form.Item>
            </div>

            <div className="add-instance-section">
              <div className="add-instance-section__title"><FolderOpenOutlined /> 安装与权限</div>
              <Form.Item
                label="实例目录"
                name="installPath"
                rules={[{ required: true, message: '请输入实例目录' }]}
              >
                <Input
                  onChange={clearImportedPreview}
                  prefix={<DatabaseOutlined />}
                  suffix={(
                    <Tooltip title="选择服务端文件夹并读取配置">
                      <Button
                        aria-label="选择服务端文件夹"
                        className="add-instance-path-picker"
                        htmlType="button"
                        icon={<FolderOpenOutlined />}
                        loading={selectingDirectory}
                        onClick={(event) => {
                          event.preventDefault()
                          event.stopPropagation()
                          void selectServerDirectory()
                        }}
                        onMouseDown={(event) => event.preventDefault()}
                        size="small"
                        type="text"
                      />
                    </Tooltip>
                  )}
                />
              </Form.Item>
              {importedPreview && (
                <div className="add-instance-import-status">
                  <span>已读取 {importedPreview.foundFiles.length} 个配置文件</span>
                  <b>{importedPreview.mods.length > 0 ? `${importedPreview.mods.length} 个 MOD` : '未发现 MOD'}</b>
                </div>
              )}
              <div className="add-instance-two-column">
                <Form.Item label="服务器加入密码" name="serverPassword" tooltip="玩家加入服务器时需要输入的密码，留空表示不设置">
                  <Input.Password prefix={<SafetyCertificateOutlined />} placeholder="留空表示无需密码" />
                </Form.Item>
                <Form.Item label="管理员密码" name="adminPassword" rules={[{ required: true, message: '请输入管理员密码' }]}>
                  <Input.Password prefix={<SafetyCertificateOutlined />} />
                </Form.Item>
              </div>
              <div className="add-instance-two-column">
                <div className="add-instance-switch-row add-instance-switch-row--full">
                  <div>
                    <strong>创建后更新/校验服务端文件</strong>
                    <span>使用当前 SteamCMD 配置执行 app_update validate</span>
                  </div>
                  <Form.Item name="autoInstall" valuePropName="checked"><Switch /></Form.Item>
                </div>
              </div>
              <Form.Item label="备注" name="description">
                <Input.TextArea rows={3} maxLength={120} showCount />
              </Form.Item>
            </div>
          </section>

          <aside className="add-instance-preview">
            <div className="add-instance-preview__title"><CheckCircleOutlined /> 创建预览</div>
            <div className="add-instance-progress">
              <div><span>基础资料</span><b>{readyCount >= 2 ? '完成' : '待补全'}</b></div>
              <div><span>端口规划</span><b>{portPlanText}</b></div>
              <div><span>实例目录</span><b>{watchedValues.installPath ? '完成' : '待补全'}</b></div>
              <div><span>配置导入</span><b>{importPlanText}</b></div>
            </div>
            <div className="add-instance-summary">
              <div><span>名称</span><strong>{watchedValues.name || '未命名'}</strong></div>
              <div><span>地图</span><strong>{formatMapLabel(selectedMap)}</strong></div>
              <div><span>模式</span><strong>{watchedValues.mode}</strong></div>
              <div><span>玩家上限</span><strong>{watchedValues.maxPlayers ?? 0}</strong></div>
              <div><span>加入密码</span><strong>{watchedValues.serverPassword ? '已设置' : '未设置'}</strong></div>
              <div><span>端口</span><strong>{watchedValues.gamePort ?? '-'} / {watchedValues.queryPort ?? '-'}</strong></div>
              <div><span>RCON</span><strong>{watchedValues.rconPort ?? '-'}</strong></div>
              {importedPreview && <div><span>导入 MOD</span><strong>{importedPreview.mods.length}</strong></div>}
            </div>
            <div className="add-instance-path">
              <FieldTimeOutlined />
              <span>{watchedValues.autoInstall ? '创建后排队更新/校验' : '仅创建实例配置'}</span>
            </div>
            <Text className="add-instance-preview__note" type="secondary">
              {watchedValues.installPath || '等待实例目录'}
            </Text>
          </aside>
        </main>

        <footer className="add-instance-footer">
          <Text type="secondary">原型创建不会立即启动服务器进程</Text>
          <Space>
            <Button htmlType="button" icon={<CloseOutlined />} onClick={() => void closeCurrentWindow()}>取消</Button>
            <Button disabled={!portsReady} loading={submitting} type="primary" icon={<PlusOutlined />} htmlType="submit">添加实例</Button>
          </Space>
        </footer>
      </Form>
    </div>
  )
}
