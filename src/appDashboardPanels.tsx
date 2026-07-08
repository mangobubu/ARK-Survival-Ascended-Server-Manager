import { DatabaseOutlined, DeleteOutlined, ExportOutlined, FolderOpenOutlined, ImportOutlined, LineChartOutlined } from '@ant-design/icons'
import { Button, Checkbox, Space, Tabs } from 'antd'
import type { TabsProps } from 'antd'
import { canDeleteInstance } from './appShell'
import type { ServerInstance } from './types'

interface ClusterLogCardProps {
  activeLogTab: string
  autoScrollLogs: boolean
  items: TabsProps['items']
  onActiveLogTabChange: (key: string) => void
  onAutoScrollLogsChange: (enabled: boolean) => void
  onClearAllLogs: () => void
}

export function ClusterLogCard({
  activeLogTab,
  autoScrollLogs,
  items,
  onActiveLogTabChange,
  onAutoScrollLogsChange,
  onClearAllLogs,
}: ClusterLogCardProps) {
  return (
    <section className="surface cluster-log-card">
      <div className="surface__title">
        <span><LineChartOutlined /> 集群日志 / 实例状态</span>
        <Space>
          <Checkbox checked={autoScrollLogs} onChange={(event) => onAutoScrollLogsChange(event.target.checked)}>自动滚动</Checkbox>
          <Button size="small" onClick={onClearAllLogs}>清除所有日志</Button>
        </Space>
      </div>
      <Tabs
        className="log-tabs"
        activeKey={activeLogTab}
        onChange={onActiveLogTabChange}
        items={items}
        size="small"
      />
    </section>
  )
}

interface QuickActionsProps {
  instancesCount: number
  selected: ServerInstance | undefined
  selectedRowsCount: number
  onImportConfig: () => void
  onExportSelected: () => void
  onExportAll: () => void
  onCreateSelectedBackup: () => void
  onOpenSelectedDirectory: () => void
  onDeleteSelected: () => void
}

export function QuickActions({
  instancesCount,
  selected,
  selectedRowsCount,
  onImportConfig,
  onExportSelected,
  onExportAll,
  onCreateSelectedBackup,
  onOpenSelectedDirectory,
  onDeleteSelected,
}: QuickActionsProps) {
  return (
    <section className="surface quick-actions">
      <div className="quick-actions__title">快捷操作</div>
      <div className="quick-actions__buttons">
        <Button icon={<ImportOutlined />} onClick={onImportConfig}>导入实例配置</Button>
        <Button className="green-button" icon={<ExportOutlined />} disabled={selectedRowsCount === 0} onClick={onExportSelected}>导出所选实例</Button>
        <Button className="gold-button" icon={<DatabaseOutlined />} disabled={instancesCount === 0} onClick={onExportAll}>导出整个集群</Button>
        <Button icon={<DatabaseOutlined />} disabled={!selected} onClick={onCreateSelectedBackup}>创建所选备份</Button>
        <Button icon={<FolderOpenOutlined />} disabled={!selected} onClick={onOpenSelectedDirectory}>打开实例目录</Button>
        <Button danger icon={<DeleteOutlined />} disabled={!selected || !canDeleteInstance(selected.status)} onClick={onDeleteSelected}>删除所选实例</Button>
      </div>
    </section>
  )
}
