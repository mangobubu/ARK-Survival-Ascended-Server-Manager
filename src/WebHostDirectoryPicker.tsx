import {
  ArrowUpOutlined,
  CheckCircleOutlined,
  FolderOpenOutlined,
  FolderOutlined,
  HomeOutlined,
  ReloadOutlined,
} from '@ant-design/icons'
import { Alert, Button, Empty, Input, List, Modal, Space, Spin, Tag, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { listHostDirectories } from './backendApi'
import type { HostDirectoryListing } from './types'

const { Text } = Typography

interface WebHostDirectoryPickerProps {
  open: boolean
  initialPath: string
  choosing: boolean
  onChoose: (path: string) => void
  onCancel: () => void
}

function formatEntryMeta(entry: HostDirectoryListing['entries'][number]) {
  if (entry.serverConfigDetected) return '已发现 ASA 配置'
  if (entry.serverExecutableDetected) return '已发现 ASA 服务端程序'
  if (entry.hasChildren) return '包含子文件夹'
  return '空文件夹'
}

export default function WebHostDirectoryPicker({
  open,
  initialPath,
  choosing,
  onChoose,
  onCancel,
}: WebHostDirectoryPickerProps) {
  const [listing, setListing] = useState<HostDirectoryListing | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const loadDirectory = async (path?: string | null) => {
    setLoading(true)
    setError('')
    try {
      setListing(await listHostDirectories(path))
    } catch (loadError) {
      setListing(null)
      setError(String(loadError))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (!open) return
    void loadDirectory(initialPath)
  }, [initialPath, open])

  const currentPath = listing?.currentPath ?? initialPath
  const rootPath = listing?.rootPath

  return (
    <Modal
      cancelButtonProps={{ disabled: choosing }}
      cancelText="取消"
      className="web-host-directory-picker"
      okButtonProps={{ disabled: !listing || loading }}
      okText="选择当前目录"
      onCancel={onCancel}
      onOk={() => listing && onChoose(listing.currentPath)}
      open={open}
      title="选择运行主机上的 ASA 服务端文件夹"
      width={760}
      confirmLoading={choosing}
    >
      <div className="web-host-directory-picker__intro">
        浏览的是运行 Web 管理服务的主机目录，目录范围限定在服务器存储目录内；选择后会自动读取 ASA 配置文件并回填表单。
      </div>

      <div className="web-host-directory-picker__toolbar">
        <Input readOnly value={currentPath} prefix={<FolderOpenOutlined />} />
        <Space.Compact>
          <Button
            aria-label="返回服务器存储根目录"
            disabled={!rootPath || loading || choosing}
            icon={<HomeOutlined />}
            onClick={() => void loadDirectory(rootPath)}
          />
          <Button
            aria-label="返回上一级目录"
            disabled={!listing?.parentPath || loading || choosing}
            icon={<ArrowUpOutlined />}
            onClick={() => void loadDirectory(listing?.parentPath)}
          />
          <Button
            aria-label="刷新目录"
            disabled={!listing || loading || choosing}
            icon={<ReloadOutlined />}
            onClick={() => void loadDirectory(listing?.currentPath)}
          />
        </Space.Compact>
      </div>

      {error && <Alert showIcon type="error" message="无法读取目录" description={error} />}

      <div className="web-host-directory-picker__list">
        <Spin spinning={loading}>
          {listing && listing.entries.length > 0 ? (
            <List
              dataSource={listing.entries}
              renderItem={(entry) => (
                <List.Item
                  actions={[
                    <Button
                      disabled={choosing}
                      key="choose"
                      onClick={() => onChoose(entry.path)}
                      size="small"
                      type="primary"
                    >
                      选择
                    </Button>,
                    <Button
                      disabled={loading || choosing}
                      key="open"
                      onClick={() => void loadDirectory(entry.path)}
                      size="small"
                    >
                      打开
                    </Button>,
                  ]}
                >
                  <List.Item.Meta
                    avatar={<FolderOutlined className="web-host-directory-picker__folder-icon" />}
                    title={(
                      <span className="web-host-directory-picker__entry-title">
                        {entry.name}
                        {entry.serverConfigDetected && <Tag color="green">ASA 配置</Tag>}
                        {entry.serverExecutableDetected && <Tag color="blue">服务端程序</Tag>}
                      </span>
                    )}
                    description={(
                      <span className="web-host-directory-picker__entry-description">
                        <span>{formatEntryMeta(entry)}</span>
                        <Text copyable={{ text: entry.path }} type="secondary">{entry.path}</Text>
                      </span>
                    )}
                  />
                </List.Item>
              )}
            />
          ) : (
            <Empty
              description={loading ? '正在读取目录…' : '当前目录下没有子文件夹，可直接选择当前目录'}
              image={Empty.PRESENTED_IMAGE_SIMPLE}
            />
          )}
        </Spin>
      </div>

      {listing?.truncated && (
        <Alert
          showIcon
          type="info"
          message={`当前目录共有 ${listing.totalEntries} 个子文件夹，已显示前 500 个；请进入更精确的上级目录后继续选择。`}
        />
      )}

      <div className="web-host-directory-picker__selected">
        <CheckCircleOutlined />
        <span>当前将选择：</span>
        <strong>{listing?.currentPath ?? '等待目录加载'}</strong>
      </div>
    </Modal>
  )
}
