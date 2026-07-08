import { type Key, useMemo } from 'react'
import {
  AppstoreAddOutlined,
  CloudDownloadOutlined,
  CodeOutlined,
  DatabaseOutlined,
  DeleteOutlined,
  EditOutlined,
  EllipsisOutlined,
  FolderOpenOutlined,
  PlayCircleFilled,
  PlusOutlined,
  ReloadOutlined,
  StopFilled,
} from '@ant-design/icons'
import { Button, Dropdown, Empty, Progress, Space, Table, Tag, Tooltip } from 'antd'
import type { MenuProps } from 'antd'
import type { ColumnsType } from 'antd/es/table'
import {
  canDeleteInstance,
  formatServerVersion,
  mapGlyphs,
  statusMeta,
} from './appShell'
import { InstanceJobProgress, isActiveJobProgress } from './jobProgressView'
import type { JobProgress, ServerInstance, ServerStatus } from './types'

interface InstanceTableProps {
  instances: ServerInstance[]
  selectedId: string
  selectedRows: Key[]
  jobProgress: Record<string, JobProgress>
  batchMenu: MenuProps
  onAddInstance: () => void
  onRefreshStatus: () => void
  onSelectedRowsChange: (keys: Key[]) => void
  onSelectInstance: (id: string) => void
  onStartInstance: (item: ServerInstance) => void
  onStopInstance: (item: ServerInstance) => void
  onInstallInstance: (item: ServerInstance) => void
  onCreateBackup: (item: ServerInstance) => void
  onOpenDirectory: (item: ServerInstance) => void
  onOpenRcon: (item: ServerInstance) => void
  onDeleteInstance: (item: ServerInstance) => void
}

export default function InstanceTable({
  instances,
  selectedId,
  selectedRows,
  jobProgress,
  batchMenu,
  onAddInstance,
  onRefreshStatus,
  onSelectedRowsChange,
  onSelectInstance,
  onStartInstance,
  onStopInstance,
  onInstallInstance,
  onCreateBackup,
  onOpenDirectory,
  onOpenRcon,
  onDeleteInstance,
}: InstanceTableProps) {
  const columns: ColumnsType<ServerInstance> = useMemo(() => [
    {
      title: '实例名称',
      dataIndex: 'name',
      width: 112,
      render: (name: string, item) => <div className="instance-name"><span className="instance-node"><span /><span /><span /></span><strong>{name}</strong>{item.id === selectedId && <span className="selected-dot" />}</div>,
    },
    { title: '地图', dataIndex: 'map', width: 118, render: (map: string) => <span className="map-name"><b>{mapGlyphs[map] ?? '◆'}</b>{map}</span> },
    { title: '模式', dataIndex: 'mode', width: 48 },
    {
      title: '状态',
      dataIndex: 'status',
      width: 86,
      render: (status: ServerStatus) => {
        const meta = statusMeta(status)
        return <Tag color={meta.color}>{meta.text}</Tag>
      },
    },
    {
      title: '服务端版本号',
      dataIndex: 'serverVersion',
      width: 112,
      render: (_, item) => <span className="mono-text">{formatServerVersion(item.serverVersion, item.versionState)}</span>,
    },
    { title: '端口 / 查询', width: 96, render: (_, item) => <span className="mono-text">{item.gamePort} / {item.queryPort}</span> },
    {
      title: '玩家数 / 上限',
      width: 108,
      render: (_, item) => <div className="player-cell"><span>{item.players} / {item.maxPlayers}</span><Progress percent={item.maxPlayers > 0 ? item.players / item.maxPlayers * 100 : 0} showInfo={false} size="small" strokeColor="#16cc79" railColor="#152838" /></div>,
    },
    {
      title: '操作',
      width: 108,
      render: (_, item) => (
        <Space.Compact>
          <Tooltip title="启动"><Button size="small" type="text" icon={<PlayCircleFilled />} disabled={item.status !== 'stopped' && item.status !== 'error'} onClick={(event) => { event.stopPropagation(); onStartInstance(item) }} /></Tooltip>
          <Tooltip title="停止"><Button size="small" type="text" icon={<StopFilled />} danger={item.status === 'running'} disabled={item.status === 'stopped' || item.status === 'stopping'} onClick={(event) => { event.stopPropagation(); onStopInstance(item) }} /></Tooltip>
          <Dropdown
            menu={{
              items: [
                { key: 'install', label: '安装/更新', icon: <CloudDownloadOutlined /> },
                { key: 'backup', label: '创建备份', icon: <DatabaseOutlined /> },
                { key: 'folder', label: '打开目录', icon: <FolderOpenOutlined /> },
                { key: 'rcon', label: 'RCON管理', icon: <CodeOutlined /> },
                { key: 'edit', label: '编辑实例', icon: <EditOutlined /> },
                { key: 'delete', label: '删除实例', icon: <DeleteOutlined />, danger: true, disabled: !canDeleteInstance(item.status) },
              ],
              onClick: ({ key }) => {
                if (key === 'install') onInstallInstance(item)
                if (key === 'backup') onCreateBackup(item)
                if (key === 'folder') onOpenDirectory(item)
                if (key === 'rcon') onOpenRcon(item)
                if (key === 'edit') onSelectInstance(item.id)
                if (key === 'delete') onDeleteInstance(item)
              },
            }}
            trigger={['click']}
          >
            <Button size="small" type="text" icon={<EllipsisOutlined />} onClick={(event) => event.stopPropagation()} />
          </Dropdown>
        </Space.Compact>
      ),
    },
  ], [
    onCreateBackup,
    onDeleteInstance,
    onInstallInstance,
    onOpenDirectory,
    onOpenRcon,
    onSelectInstance,
    onStartInstance,
    onStopInstance,
    selectedId,
  ])

  const activeProgressIds = useMemo(
    () => instances
      .filter((item) => item.status === 'updating' && isActiveJobProgress(jobProgress[item.id]))
      .map((item) => item.id),
    [instances, jobProgress],
  )

  return (
    <section className="surface instance-list-card">
      <div className="surface__title">
        <span>服务器实例列表</span>
        <Space size={8}>
          <Button size="small" icon={<PlusOutlined />} onClick={onAddInstance}>添加实例</Button>
          <Dropdown menu={batchMenu} disabled={selectedRows.length === 0}>
            <Button size="small" icon={<AppstoreAddOutlined />}>批量操作</Button>
          </Dropdown>
          <Button size="small" icon={<ReloadOutlined />} onClick={onRefreshStatus}>刷新列表</Button>
        </Space>
      </div>
      <Table
        rowKey="id"
        columns={columns}
        dataSource={instances}
        locale={{ emptyText: <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="尚未创建服务器实例" /> }}
        pagination={false}
        size="small"
        tableLayout="fixed"
        scroll={{ y: 360 }}
        expandable={{
          expandedRowKeys: activeProgressIds,
          expandedRowRender: (item) => <InstanceJobProgress progress={jobProgress[item.id]} />,
          rowExpandable: (item) => isActiveJobProgress(jobProgress[item.id]),
          showExpandColumn: false,
        }}
        rowSelection={{ selectedRowKeys: selectedRows, onChange: onSelectedRowsChange, columnWidth: 36 }}
        onRow={(item) => ({ onClick: () => onSelectInstance(item.id) })}
        rowClassName={(item) => item.id === selectedId ? 'selected-instance-row' : ''}
      />
    </section>
  )
}
