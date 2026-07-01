import { useMemo, useState } from 'react'
import {
  AppstoreAddOutlined,
  CheckCircleOutlined,
  CloudServerOutlined,
  DatabaseOutlined,
  EditOutlined,
  EllipsisOutlined,
  ExclamationCircleFilled,
  ExportOutlined,
  FolderOpenOutlined,
  ImportOutlined,
  LineChartOutlined,
  PlayCircleFilled,
  PlusOutlined,
  ReloadOutlined,
  SaveOutlined,
  SettingOutlined,
  StopFilled,
  TeamOutlined,
} from '@ant-design/icons'
import { Button, Checkbox, Dropdown, message, Modal, Progress, Space, Table, Tag, Tooltip, Typography } from 'antd'
import type { ColumnsType } from 'antd/es/table'
import ConfigPanel from './ConfigPanel'
import { defaultConfig, initialInstances, initialLogs, initialMods } from './data'
import type { LogLine, ModItem, ServerConfig, ServerInstance } from './types'

const { Text } = Typography

const mapGlyphs: Record<string, string> = {
  'The Island': '◆',
  'Scorched Earth': '✣',
  Aberration: '◇',
  'The Center': '✦',
  Extinction: '⬡',
  Astraeos: '✧',
  Ragnarok: 'ᛉ',
  Valguero: '△',
  'Lost Colony': '◈',
}

const nowTime = () => new Date().toLocaleTimeString('zh-CN', { hour12: false })

function Logo() {
  return (
    <div className="brand">
      <img className="brand__emblem" src="/app-icon.png" alt="ASA 服务器管理器" />
      <div><div className="brand__title">方舟进化飞升服务器管理器</div><div className="brand__subtitle">ARK SURVIVAL ASCENDED SERVER MANAGER</div></div>
    </div>
  )
}

function StatCard({ icon, label, value, suffix, tone = 'blue' }: { icon: React.ReactNode; label: string; value: string | number; suffix: string; tone?: 'blue' | 'green' }) {
  return (
    <div className={`stat-card stat-card--${tone}`}>
      <div className="stat-card__icon">{icon}</div>
      <div><div className="stat-card__label">{label}</div><div className="stat-card__value">{value} <small>{suffix}</small></div></div>
    </div>
  )
}

export default function App() {
  const [instances, setInstances] = useState(initialInstances)
  const [selectedId, setSelectedId] = useState('asa-01')
  const [selectedRows, setSelectedRows] = useState<React.Key[]>(['asa-01'])
  const [config, setConfig] = useState<ServerConfig>(() => {
    try { return { ...defaultConfig, ...JSON.parse(localStorage.getItem('asa-config') ?? '{}') } }
    catch { return defaultConfig }
  })
  const [mods, setMods] = useState<ModItem[]>(initialMods)
  const [logs, setLogs] = useState<LogLine[]>(initialLogs)
  const [dirty, setDirty] = useState(false)
  const [messageApi, contextHolder] = message.useMessage()
  const [modal, modalContext] = Modal.useModal()

  const selected = instances.find((item) => item.id === selectedId) ?? instances[0]
  const running = instances.filter((item) => item.status === 'running').length
  const totalPlayers = instances.reduce((sum, item) => sum + item.players, 0)
  const playerCapacity = instances.reduce((sum, item) => sum + item.maxPlayers, 0)

  const appendLog = (instance: string, level: LogLine['level'], text: string) => {
    setLogs((current) => [...current, { id: Date.now(), time: nowTime(), instance, level, message: text }])
  }

  const changeStatus = (id: string, status: ServerInstance['status']) => {
    setInstances((current) => current.map((item) => item.id === id ? { ...item, status } : item))
  }

  const startInstance = (item: ServerInstance) => {
    if (item.status === 'running') return messageApi.info(`${item.name} 已在运行`)
    changeStatus(item.id, 'starting')
    appendLog(item.name, 'info', `正在启动 ${item.map}，检查端口与配置…`)
    window.setTimeout(() => {
      changeStatus(item.id, 'running')
      appendLog(item.name, 'success', `启动成功，地图：${item.map}`)
      messageApi.success(`${item.name} 已启动`)
    }, 900)
  }

  const stopInstance = (item: ServerInstance) => {
    if (item.status === 'stopped') return messageApi.info(`${item.name} 已停止`)
    modal.confirm({
      title: `停止 ${item.name}？`, icon: <ExclamationCircleFilled />, content: '管理器将先执行世界保存，再结束服务端进程。',
      okText: '保存并停止', cancelText: '取消', okButtonProps: { danger: true },
      onOk: () => {
        changeStatus(item.id, 'stopped')
        appendLog(item.name, 'warn', '实例已保存并停止')
        messageApi.success(`${item.name} 已停止`)
      },
    })
  }

  const saveConfig = () => {
    localStorage.setItem('asa-config', JSON.stringify(config))
    setDirty(false)
    appendLog(selected.name, 'success', '配置保存成功')
    messageApi.success('实例配置已保存')
  }

  const applyConfig = () => {
    modal.confirm({
      title: `保存并重启 ${selected.name}？`,
      icon: <ReloadOutlined className="confirm-blue-icon" />,
      content: '将先保存世界与配置，然后重启实例。在线玩家会被断开连接。',
      okText: '保存并重启', cancelText: '取消',
      onOk: () => {
        saveConfig()
        changeStatus(selected.id, 'starting')
        appendLog(selected.name, 'info', '正在应用配置并重启…')
        window.setTimeout(() => {
          changeStatus(selected.id, 'running')
          appendLog(selected.name, 'success', '新配置已生效，实例重启完成')
        }, 900)
      },
    })
  }

  const updateConfig = <K extends keyof ServerConfig>(key: K, value: ServerConfig[K]) => {
    setConfig((current) => ({ ...current, [key]: value }))
    setDirty(true)
  }

  const columns: ColumnsType<ServerInstance> = useMemo(() => [
    {
      title: '实例名称', dataIndex: 'name', width: 112,
      render: (name: string, item) => <div className="instance-name"><span className="instance-node"><span /><span /><span /></span><strong>{name}</strong>{item.id === selectedId && <span className="selected-dot" />}</div>,
    },
    { title: '地图', dataIndex: 'map', width: 118, render: (map: string) => <span className="map-name"><b>{mapGlyphs[map] ?? '◆'}</b>{map}</span> },
    { title: '模式', dataIndex: 'mode', width: 48 },
    {
      title: '状态', dataIndex: 'status', width: 80,
      render: (status: ServerInstance['status']) => <Tag color={status === 'running' ? 'success' : status === 'starting' ? 'processing' : 'error'}>{status === 'running' ? '⊙ 运行中' : status === 'starting' ? '◌ 启动中' : '⊖ 已停止'}</Tag>,
    },
    { title: '端口 / 查询', width: 96, render: (_, item) => <span className="mono-text">{item.gamePort} / {item.queryPort}</span> },
    {
      title: '玩家数 / 上限', width: 108,
      render: (_, item) => <div className="player-cell"><span>{item.players} / {item.maxPlayers}</span><Progress percent={item.players / item.maxPlayers * 100} showInfo={false} size="small" strokeColor="#16cc79" trailColor="#152838" /></div>,
    },
    {
      title: '操作', width: 96,
      render: (_, item) => <Space.Compact>
        <Tooltip title="启动"><Button size="small" type="text" icon={<PlayCircleFilled />} disabled={item.status !== 'stopped'} onClick={(e) => { e.stopPropagation(); startInstance(item) }} /></Tooltip>
        <Tooltip title="停止"><Button size="small" type="text" icon={<StopFilled />} danger={item.status === 'running'} disabled={item.status === 'stopped'} onClick={(e) => { e.stopPropagation(); stopInstance(item) }} /></Tooltip>
        <Dropdown menu={{ items: [{ key: 'edit', label: '编辑实例', icon: <EditOutlined /> }, { key: 'folder', label: '打开目录', icon: <FolderOpenOutlined /> }] }} trigger={['click']}>
          <Button size="small" type="text" icon={<EllipsisOutlined />} onClick={(e) => e.stopPropagation()} />
        </Dropdown>
      </Space.Compact>,
    },
  ], [selectedId])

  return (
    <div className="app-shell">
      {contextHolder}{modalContext}
      <header className="topbar">
        <Logo />
        <div className="topbar__actions">
          <Button icon={<PlayCircleFilled />} onClick={() => selectedRows.forEach((id) => { const item = instances.find((i) => i.id === id); if (item) startInstance(item) })}>启动所选</Button>
          <Button danger icon={<StopFilled />} onClick={() => selectedRows.forEach((id) => { const item = instances.find((i) => i.id === id); if (item) stopInstance(item) })}>停止所选</Button>
          <Button icon={<SaveOutlined />} onClick={saveConfig}>保存配置</Button>
          <Button className="apply-button" icon={<ReloadOutlined />} onClick={applyConfig}>应用并重启</Button>
          <Button icon={<SettingOutlined />} aria-label="全局设置" />
        </div>
      </header>

      <main className="workspace">
        <section className="stats-grid">
          <StatCard icon={<CloudServerOutlined />} label="总服务器数" value={instances.length} suffix="个实例" />
          <StatCard icon={<CheckCircleOutlined />} label="运行中" value={`${running} / ${instances.length}`} suffix="个实例" tone="green" />
          <StatCard icon={<DatabaseOutlined />} label="地图数量" value={new Set(instances.map((item) => item.map)).size} suffix="张地图" />
          <StatCard icon={<TeamOutlined />} label="总玩家数" value={`${totalPlayers} / ${playerCapacity}`} suffix="" />
        </section>

        <div className="main-grid">
          <div className="left-column">
            <section className="surface instance-list-card">
              <div className="surface__title">
                <span>服务器实例列表</span>
                <Space size={8}>
                  <Button size="small" icon={<PlusOutlined />} onClick={() => messageApi.info('添加实例向导将在下一步接入')}>添加实例</Button>
                  <Button size="small" icon={<AppstoreAddOutlined />}>批量操作</Button>
                  <Button size="small" icon={<ReloadOutlined />} onClick={() => messageApi.success('实例状态已刷新')}>刷新列表</Button>
                </Space>
              </div>
              <Table
                rowKey="id"
                columns={columns}
                dataSource={instances}
                pagination={false}
                size="small"
                tableLayout="fixed"
                scroll={{ y: 360 }}
                rowSelection={{ selectedRowKeys: selectedRows, onChange: setSelectedRows, columnWidth: 36 }}
                onRow={(item) => ({ onClick: () => { setSelectedId(item.id); setSelectedRows([item.id]) } })}
                rowClassName={(item) => item.id === selectedId ? 'selected-instance-row' : ''}
              />
            </section>

            <section className="surface cluster-log-card">
              <div className="surface__title">
                <span><LineChartOutlined /> 集群日志 / 实例状态</span>
                <Space><Checkbox defaultChecked>自动滚动</Checkbox><Button size="small" onClick={() => setLogs([])}>清空日志</Button></Space>
              </div>
              <div className="log-console">
                {logs.length === 0 ? <div className="empty-log">暂无日志</div> : logs.map((line) => (
                  <div className={`log-line log-line--${line.level}`} key={line.id}>
                    <span>[{line.time}]</span><b>[{line.instance}]</b><span>{line.message}</span>
                  </div>
                ))}
              </div>
            </section>
          </div>

          <ConfigPanel
            instance={selected}
            config={config}
            mods={mods}
            dirty={dirty}
            onConfigChange={updateConfig}
            onModsChange={(next) => { setMods(next); setDirty(true) }}
            onSave={saveConfig}
            onApply={applyConfig}
          />
        </div>

        <section className="surface quick-actions">
          <div className="quick-actions__title">快捷操作</div>
          <div className="quick-actions__buttons">
            <Button icon={<ImportOutlined />}>导入实例配置</Button>
            <Button className="green-button" icon={<ExportOutlined />}>导出所选实例</Button>
            <Button className="gold-button" icon={<DatabaseOutlined />}>导出整个集群</Button>
            <Button icon={<FolderOpenOutlined />}>打开存档目录</Button>
          </div>
        </section>
      </main>

      <footer className="app-footer">
        <Text type="secondary">v0.1.0 Prototype</Text>
        <div><span>▣ 上次保存：{dirty ? '存在未保存修改' : '刚刚'}</span><span>▧ 配置目录：D:\ASA-Server\ShooterGame\Saved\Config\WindowsServer</span></div>
      </footer>
    </div>
  )
}
