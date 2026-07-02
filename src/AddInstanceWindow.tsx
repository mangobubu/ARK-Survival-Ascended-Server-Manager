import { useMemo, useState } from 'react'
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
import { emitTo } from '@tauri-apps/api/event'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { Button, Form, Input, InputNumber, Radio, Select, Space, Switch, Typography, message } from 'antd'
import { defaultConfig, defaultGlobalSettings, serverMapOptions } from './data'
import type { AddInstancePayload } from './types'
import { ADD_INSTANCE_CREATED_EVENT, MAIN_WINDOW_LABEL } from './windowEvents'

const { Text, Title } = Typography

const formatMapLabel = (map: { name: string; zhName: string }) => `${map.zhName}(${map.name})`

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

function readNumberParam(name: string, fallback: number) {
  const value = Number(new URLSearchParams(window.location.search).get(name))
  return Number.isFinite(value) && value > 0 ? value : fallback
}

function readTextParam(name: string, fallback: string) {
  return new URLSearchParams(window.location.search).get(name) ?? fallback
}

export default function AddInstanceWindow() {
  const [form] = Form.useForm<AddInstanceFormValues>()
  const [messageApi, contextHolder] = message.useMessage()
  const [submitting, setSubmitting] = useState(false)

  const closeCurrentWindow = async () => {
    try {
      await WebviewWindow.getCurrent().close()
    } catch (error) {
      console.error('关闭添加实例窗口失败', error)
      messageApi.error(`无法关闭窗口：${String(error)}`)
    }
  }

  const sequence = readNumberParam('index', 10)
  const defaultName = `ASA-${String(sequence).padStart(2, '0')}`
  const initialValues = useMemo<AddInstanceFormValues>(() => ({
    name: defaultName,
    mapCode: 'TheIsland_WP',
    mode: 'PvE',
    gamePort: readNumberParam('gamePort', 7867),
    queryPort: readNumberParam('queryPort', 27105),
    rconPort: readNumberParam('rconPort', 32340),
    maxPlayers: defaultConfig.maxPlayers,
    installPath: `${readTextParam('serverRoot', defaultGlobalSettings.serverStoragePath)}\\${defaultName}`,
    clusterId: defaultConfig.clusterId,
    serverPassword: defaultConfig.serverPassword,
    adminPassword: defaultConfig.adminPassword,
    autoInstall: true,
    description: '',
  }), [defaultName])

  const watchedValues = Form.useWatch([], form) ?? initialValues
  const selectedMap = serverMapOptions.find((item) => item.code === watchedValues.mapCode) ?? serverMapOptions[0]
  const readyCount = [
    watchedValues.name,
    watchedValues.mapCode,
    watchedValues.gamePort,
    watchedValues.queryPort,
    watchedValues.installPath,
  ].filter(Boolean).length

  const handleFinish = async (values: AddInstanceFormValues) => {
    const map = serverMapOptions.find((item) => item.code === values.mapCode) ?? serverMapOptions[0]
    const payload: AddInstancePayload = {
      id: `asa-${Date.now()}`,
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
    }

    setSubmitting(true)
    try {
      await emitTo(MAIN_WINDOW_LABEL, ADD_INSTANCE_CREATED_EVENT, payload)
      messageApi.success('实例已添加到主窗口')
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
                <Form.Item label="游戏端口" name="gamePort" rules={[{ required: true, message: '请输入游戏端口' }]}>
                  <InputNumber min={1024} max={65535} />
                </Form.Item>
                <Form.Item label="查询端口" name="queryPort" rules={[{ required: true, message: '请输入查询端口' }]}>
                  <InputNumber min={1024} max={65535} />
                </Form.Item>
                <Form.Item label="RCON 端口" name="rconPort" rules={[{ required: true, message: '请输入 RCON 端口' }]}>
                  <InputNumber min={1024} max={65535} />
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
                <Input prefix={<DatabaseOutlined />} />
              </Form.Item>
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
                    <strong>创建后安装服务端文件</strong>
                    <span>使用当前 SteamCMD 配置</span>
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
              <div><span>端口规划</span><b>{readyCount >= 4 ? '完成' : '待补全'}</b></div>
              <div><span>实例目录</span><b>{watchedValues.installPath ? '完成' : '待补全'}</b></div>
            </div>
            <div className="add-instance-summary">
              <div><span>名称</span><strong>{watchedValues.name || '未命名'}</strong></div>
              <div><span>地图</span><strong>{formatMapLabel(selectedMap)}</strong></div>
              <div><span>模式</span><strong>{watchedValues.mode}</strong></div>
              <div><span>玩家上限</span><strong>{watchedValues.maxPlayers ?? 0}</strong></div>
              <div><span>加入密码</span><strong>{watchedValues.serverPassword ? '已设置' : '未设置'}</strong></div>
              <div><span>端口</span><strong>{watchedValues.gamePort ?? '-'} / {watchedValues.queryPort ?? '-'}</strong></div>
              <div><span>RCON</span><strong>{watchedValues.rconPort ?? '-'}</strong></div>
            </div>
            <div className="add-instance-path">
              <FieldTimeOutlined />
              <span>{watchedValues.autoInstall ? '创建后排队安装' : '仅创建实例配置'}</span>
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
            <Button loading={submitting} type="primary" icon={<PlusOutlined />} htmlType="submit">添加实例</Button>
          </Space>
        </footer>
      </Form>
    </div>
  )
}
