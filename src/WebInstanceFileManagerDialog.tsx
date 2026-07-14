import {
  ArrowUpOutlined,
  CopyOutlined,
  DeleteOutlined,
  EditOutlined,
  EllipsisOutlined,
  FileAddOutlined,
  FileOutlined,
  FolderAddOutlined,
  FolderOpenOutlined,
  FolderOutlined,
  HomeOutlined,
  ReloadOutlined,
  SnippetsOutlined,
} from '@ant-design/icons'
import { Alert, Button, Dropdown, Empty, Input, List, Modal, Space, Spin, Tag, Typography, message } from 'antd'
import type { MenuProps } from 'antd'
import { useEffect, useState } from 'react'
import {
  copyInstanceFileEntry,
  createInstanceFileEntry,
  deleteInstanceFileEntry,
  listInstanceFiles,
  renameInstanceFileEntry,
} from './backendApi'
import type { InstanceDirectoryListing, InstanceFileEntry, ServerInstance } from './types'

const { Text } = Typography

type EntryEditorState =
  | { mode: 'create-directory'; value: string }
  | { mode: 'create-file'; value: string }
  | { mode: 'rename'; value: string; entry: InstanceFileEntry }
  | null

interface WebInstanceFileManagerDialogProps {
  instance: ServerInstance | null
  onClose: () => void
}

function formatBytes(sizeBytes: number | null) {
  if (sizeBytes === null) return '—'
  if (sizeBytes < 1024) return `${sizeBytes} B`
  const units = ['KB', 'MB', 'GB', 'TB']
  let value = sizeBytes / 1024
  let unitIndex = 0
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }
  return `${value >= 10 ? value.toFixed(1) : value.toFixed(2)} ${units[unitIndex]}`
}

function formatModifiedTime(modifiedAt: number | null) {
  if (modifiedAt === null) return '未知'
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(new Date(modifiedAt))
}

function entryTypeText(entry: InstanceFileEntry) {
  if (entry.entryType === 'directory') return entry.hasChildren ? '文件夹 · 包含内容' : '空文件夹'
  if (entry.entryType === 'file') return `文件 · ${formatBytes(entry.sizeBytes)}`
  return '不支持操作的特殊目录项'
}

export default function WebInstanceFileManagerDialog({
  instance,
  onClose,
}: WebInstanceFileManagerDialogProps) {
  const [listing, setListing] = useState<InstanceDirectoryListing | null>(null)
  const [clipboard, setClipboard] = useState<InstanceFileEntry | null>(null)
  const [editor, setEditor] = useState<EntryEditorState>(null)
  const [loading, setLoading] = useState(false)
  const [mutating, setMutating] = useState(false)
  const [error, setError] = useState('')
  const [messageApi, contextHolder] = message.useMessage()

  const loadDirectory = async (path?: string | null) => {
    if (!instance) return
    setLoading(true)
    setError('')
    try {
      setListing(await listInstanceFiles(instance.id, path))
    } catch (loadError) {
      setError(String(loadError))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (!instance) {
      setListing(null)
      setClipboard(null)
      setEditor(null)
      setError('')
      return
    }
    setClipboard(null)
    void loadDirectory()
  }, [instance?.id])

  const refreshCurrentDirectory = async () => {
    await loadDirectory(listing?.currentPath)
  }

  const openEditor = (mode: 'create-directory' | 'create-file' | 'rename', entry?: InstanceFileEntry) => {
    if (mode === 'rename') {
      if (!entry) return
      setEditor({ mode: 'rename', value: entry.name, entry })
      return
    }
    if (mode === 'create-directory') {
      setEditor({ mode: 'create-directory', value: '新建文件夹' })
      return
    }
    setEditor({ mode: 'create-file', value: '新建文件.txt' })
  }

  const submitEditor = async () => {
    if (!instance || !listing || !editor) return
    const name = editor.value.trim()
    if (!name) return
    setMutating(true)
    try {
      if (editor.mode === 'rename') {
        await renameInstanceFileEntry(instance.id, editor.entry.path, name)
        if (clipboard?.path === editor.entry.path) setClipboard(null)
        messageApi.success(`已重命名为 ${name}`)
      } else {
        await createInstanceFileEntry(
          instance.id,
          listing.currentPath,
          name,
          editor.mode === 'create-directory' ? 'directory' : 'file',
        )
        messageApi.success(editor.mode === 'create-directory' ? '文件夹已创建' : '文件已创建')
      }
      setEditor(null)
      await refreshCurrentDirectory()
    } catch (mutationError) {
      messageApi.error(`操作失败：${String(mutationError)}`)
    } finally {
      setMutating(false)
    }
  }

  const pasteEntry = async (targetDirectory: string) => {
    if (!instance || !clipboard) return
    setMutating(true)
    try {
      await copyInstanceFileEntry(instance.id, clipboard.path, targetDirectory)
      messageApi.success(`已粘贴 ${clipboard.name}`)
      await refreshCurrentDirectory()
    } catch (mutationError) {
      messageApi.error(`粘贴失败：${String(mutationError)}`)
    } finally {
      setMutating(false)
    }
  }

  const deleteEntry = async (entry: InstanceFileEntry) => {
    if (!instance) return
    setMutating(true)
    try {
      await deleteInstanceFileEntry(instance.id, entry.path)
      if (clipboard?.path === entry.path) setClipboard(null)
      messageApi.success(`${entry.name} 已删除`)
      await refreshCurrentDirectory()
    } catch (mutationError) {
      const text = String(mutationError)
      if (text.includes('已取消高风险 Web 管理操作')) {
        messageApi.info('已取消删除')
      } else {
        messageApi.error(`删除失败：${text}`)
      }
    } finally {
      setMutating(false)
    }
  }

  const entryMenu = (entry: InstanceFileEntry): MenuProps => ({
    items: [
      {
        key: 'open',
        label: '打开',
        icon: <FolderOpenOutlined />,
        disabled: entry.entryType !== 'directory',
      },
      { type: 'divider' },
      {
        key: 'rename',
        label: '重命名',
        icon: <EditOutlined />,
        disabled: entry.entryType === 'other',
      },
      {
        key: 'copy',
        label: '复制',
        icon: <CopyOutlined />,
        disabled: entry.entryType === 'other',
      },
      {
        key: 'paste',
        label: '粘贴到此文件夹',
        icon: <SnippetsOutlined />,
        disabled: !clipboard || entry.entryType !== 'directory',
      },
      { type: 'divider' },
      {
        key: 'delete',
        label: '删除',
        icon: <DeleteOutlined />,
        danger: true,
        disabled: entry.entryType === 'other',
      },
    ],
    onClick: ({ key }) => {
      if (key === 'open') void loadDirectory(entry.path)
      if (key === 'rename') openEditor('rename', entry)
      if (key === 'copy') {
        setClipboard(entry)
        messageApi.success(`已复制 ${entry.name}，请选择目标目录后粘贴`)
      }
      if (key === 'paste') void pasteEntry(entry.path)
      if (key === 'delete') void deleteEntry(entry)
    },
  })

  const directoryMenu: MenuProps = {
    items: [
      { key: 'new-directory', label: '新建文件夹', icon: <FolderAddOutlined /> },
      { key: 'new-file', label: '新建文件', icon: <FileAddOutlined /> },
      {
        key: 'paste',
        label: '粘贴',
        icon: <SnippetsOutlined />,
        disabled: !clipboard || !listing,
      },
      { type: 'divider' },
      { key: 'refresh', label: '刷新', icon: <ReloadOutlined /> },
    ],
    onClick: ({ key }) => {
      if (key === 'new-directory') openEditor('create-directory')
      if (key === 'new-file') openEditor('create-file')
      if (key === 'paste' && listing) void pasteEntry(listing.currentPath)
      if (key === 'refresh') void refreshCurrentDirectory()
    },
  }

  const editorTitle = editor?.mode === 'rename'
    ? '重命名目录项'
    : editor?.mode === 'create-directory'
      ? '新建文件夹'
      : '新建文件'

  return (
    <>
      {contextHolder}
      <Modal
        className="web-instance-file-manager"
        footer={null}
        onCancel={onClose}
        open={instance !== null}
        title={(
          <span className="web-instance-file-manager__title">
            <FolderOpenOutlined />
            {instance ? `${instance.name} · 实例目录` : '实例目录'}
          </span>
        )}
        width={980}
      >
        <div className="web-instance-file-manager__intro">
          当前展示的是运行 Web 管理服务的主机目录。所有浏览和文件操作均限制在此实例根目录内；双击文件夹可进入，右键目录项或空白区域可执行管理操作。
        </div>

        <div className="web-instance-file-manager__toolbar">
          <Input
            prefix={<FolderOpenOutlined />}
            readOnly
            value={listing?.currentPath ?? instance?.installPath ?? ''}
          />
          <Space.Compact>
            <Button
              aria-label="返回实例根目录"
              disabled={!listing || loading || mutating}
              icon={<HomeOutlined />}
              onClick={() => void loadDirectory(listing?.rootPath)}
            />
            <Button
              aria-label="返回上一级目录"
              disabled={!listing?.parentPath || loading || mutating}
              icon={<ArrowUpOutlined />}
              onClick={() => void loadDirectory(listing?.parentPath)}
            />
            <Button
              aria-label="刷新目录"
              disabled={!listing || loading || mutating}
              icon={<ReloadOutlined />}
              onClick={() => void refreshCurrentDirectory()}
            />
          </Space.Compact>
          <Space.Compact>
            <Button
              disabled={!listing || loading || mutating}
              icon={<FolderAddOutlined />}
              onClick={() => openEditor('create-directory')}
            >
              新建文件夹
            </Button>
            <Button
              disabled={!listing || loading || mutating}
              icon={<FileAddOutlined />}
              onClick={() => openEditor('create-file')}
            >
              新建文件
            </Button>
            <Button
              disabled={!clipboard || !listing || loading || mutating}
              icon={<SnippetsOutlined />}
              onClick={() => listing && void pasteEntry(listing.currentPath)}
            >
              粘贴
            </Button>
          </Space.Compact>
        </div>

        {error && <Alert showIcon type="error" message="无法读取实例目录" description={error} />}

        <Dropdown menu={directoryMenu} trigger={['contextMenu']}>
          <div className="web-instance-file-manager__list">
            <Spin spinning={loading || mutating}>
              {listing && listing.entries.length > 0 ? (
                <List
                  dataSource={listing.entries}
                  renderItem={(entry) => (
                    <Dropdown key={entry.path} menu={entryMenu(entry)} trigger={['contextMenu']}>
                      <List.Item
                        actions={[
                          <Dropdown key="actions" menu={entryMenu(entry)} trigger={['click']}>
                            <Button
                              aria-label={`管理 ${entry.name}`}
                              icon={<EllipsisOutlined />}
                              size="small"
                              type="text"
                            />
                          </Dropdown>,
                        ]}
                        className={`web-instance-file-manager__entry web-instance-file-manager__entry--${entry.entryType}`}
                        onContextMenu={(event) => event.stopPropagation()}
                        onDoubleClick={() => entry.entryType === 'directory' && void loadDirectory(entry.path)}
                      >
                        <List.Item.Meta
                          avatar={entry.entryType === 'directory'
                            ? <FolderOutlined className="web-instance-file-manager__folder-icon" />
                            : <FileOutlined className="web-instance-file-manager__file-icon" />}
                          description={(
                            <span className="web-instance-file-manager__entry-description">
                              <span>{entryTypeText(entry)}</span>
                              <span>修改时间：{formatModifiedTime(entry.modifiedAt)}</span>
                            </span>
                          )}
                          title={(
                            <span className="web-instance-file-manager__entry-title">
                              {entry.name}
                              {clipboard?.path === entry.path && <Tag color="blue">已复制</Tag>}
                            </span>
                          )}
                        />
                      </List.Item>
                    </Dropdown>
                  )}
                />
              ) : (
                <Empty
                  description={loading ? '正在读取目录…' : '当前目录为空，可右键空白区域新建文件或文件夹'}
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                />
              )}
            </Spin>
          </div>
        </Dropdown>

        <div className="web-instance-file-manager__status">
          <Text type="secondary">
            {listing
              ? `当前显示 ${listing.entries.length} / ${listing.totalEntries} 个目录项`
              : '等待目录加载'}
          </Text>
          <span>
            {clipboard
              ? <><CopyOutlined /> 剪贴板：<strong>{clipboard.name}</strong></>
              : '右键目录项可复制、重命名或删除'}
          </span>
        </div>

        {listing?.truncated && (
          <Alert
            showIcon
            type="info"
            message={`当前目录共有 ${listing.totalEntries} 个目录项，已显示前 1000 个；请进入更精确的子目录后继续操作。`}
          />
        )}
      </Modal>

      <Modal
        cancelButtonProps={{ disabled: mutating }}
        cancelText="取消"
        confirmLoading={mutating}
        destroyOnHidden
        okButtonProps={{ disabled: !editor?.value.trim() }}
        okText="确认"
        onCancel={() => setEditor(null)}
        onOk={() => void submitEditor()}
        open={editor !== null}
        title={editorTitle}
      >
        <Input
          autoFocus
          disabled={mutating}
          maxLength={240}
          onChange={(event) => editor && setEditor({ ...editor, value: event.target.value })}
          onPressEnter={() => void submitEditor()}
          placeholder="请输入名称"
          value={editor?.value ?? ''}
        />
      </Modal>
    </>
  )
}
