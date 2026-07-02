import { useCallback, useEffect, useRef, useState } from 'react'
import {
  CheckCircleOutlined,
  CloudDownloadOutlined,
  FolderOpenOutlined,
  WarningOutlined,
} from '@ant-design/icons'
import { Channel, invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Alert, Button, Modal, Progress, Space, Spin, Typography, message } from 'antd'
import { saveGlobalSettings } from './globalSettings'
import type {
  GlobalSettings,
  SteamCmdCheck,
  SteamCmdInstallResult,
  SteamCmdProgress,
} from './types'

const { Text, Title } = Typography

interface SteamCmdSetupProps {
  settings: GlobalSettings
  onSettingsChange: (settings: GlobalSettings) => void
}

type Availability = 'checking' | 'ready' | 'missing'

const emptyProgress: SteamCmdProgress = {
  phase: 'downloading',
  downloadedBytes: 0,
  totalBytes: null,
  bytesPerSecond: 0,
  message: '准备下载 SteamCMD',
}

function formatBytes(value: number | null) {
  if (value === null) return '未知'
  if (value < 1024) return `${value} B`
  const units = ['KB', 'MB', 'GB', 'TB']
  let size = value / 1024
  let index = 0
  while (size >= 1024 && index < units.length - 1) {
    size /= 1024
    index += 1
  }
  return `${size.toFixed(size >= 100 ? 0 : size >= 10 ? 1 : 2)} ${units[index]}`
}

function phaseLabel(phase: SteamCmdProgress['phase']) {
  if (phase === 'extracting') return '正在解压'
  if (phase === 'initializing') return '正在初始化'
  if (phase === 'completed') return '安装完成'
  return '正在下载'
}

export default function SteamCmdSetup({ settings, onSettingsChange }: SteamCmdSetupProps) {
  const [messageApi, contextHolder] = message.useMessage()
  const [availability, setAvailability] = useState<Availability>('checking')
  const [guideOpen, setGuideOpen] = useState(false)
  const [checkingDirectory, setCheckingDirectory] = useState(false)
  const [installing, setInstalling] = useState(false)
  const [progress, setProgress] = useState<SteamCmdProgress>(emptyProgress)
  const [installError, setInstallError] = useState<string | null>(null)
  const [lastParentPath, setLastParentPath] = useState<string | null>(null)
  const lastCheckedPath = useRef<string | null>(null)
  const initialCheckCompleted = useRef(false)

  const applySteamCmdPath = useCallback((path: string) => {
    const next = { ...settings, steamCmdPath: path }
    void saveGlobalSettings(next).then((saved) => {
      onSettingsChange(saved)
      setAvailability('ready')
      setGuideOpen(false)
      setInstallError(null)
    }).catch((error) => {
      messageApi.error(`保存 SteamCMD 设置失败：${String(error)}`)
    })
  }, [messageApi, onSettingsChange, settings])

  const checkConfiguredPath = useCallback(async (showGuide: boolean) => {
    setAvailability('checking')
    try {
      const result = await invoke<SteamCmdCheck>('check_steamcmd', { path: settings.steamCmdPath })
      if (result.valid) {
        setAvailability('ready')
        setGuideOpen(false)
      } else {
        setAvailability('missing')
        if (showGuide) setGuideOpen(true)
      }
    } catch (error) {
      setAvailability('missing')
      if (showGuide) setGuideOpen(true)
      console.error('检查 SteamCMD 失败', error)
    }
  }, [settings.steamCmdPath])

  useEffect(() => {
    if (lastCheckedPath.current === settings.steamCmdPath) return
    lastCheckedPath.current = settings.steamCmdPath
    const showGuide = !initialCheckCompleted.current
    void checkConfiguredPath(showGuide).finally(() => {
      initialCheckCompleted.current = true
    })
  }, [settings.steamCmdPath, checkConfiguredPath])

  const selectExistingDirectory = async () => {
    setCheckingDirectory(true)
    try {
      const selected = await open({
        defaultPath: settings.steamCmdPath || undefined,
        directory: true,
        multiple: false,
        title: '选择包含 steamcmd.exe 的目录',
      })
      if (!selected) return

      const result = await invoke<SteamCmdCheck>('check_steamcmd', { path: selected })
      if (!result.valid) {
        messageApi.error(result.reason ?? '所选目录不是有效的 SteamCMD 目录')
        return
      }
      applySteamCmdPath(result.path)
      messageApi.success('SteamCMD 目录已保存到全局设置')
    } catch (error) {
      messageApi.error(`无法选择 SteamCMD 目录：${String(error)}`)
    } finally {
      setCheckingDirectory(false)
    }
  }

  const runInstall = async (parentPath: string) => {
    setLastParentPath(parentPath)
    setInstalling(true)
    setInstallError(null)
    setProgress(emptyProgress)

    const progressChannel = new Channel<SteamCmdProgress>()
    progressChannel.onmessage = (event) => setProgress(event)

    try {
      const result = await invoke<SteamCmdInstallResult>('install_steamcmd', {
        parentPath,
        progress: progressChannel,
      })
      applySteamCmdPath(result.path)
      messageApi.success('SteamCMD 已静默安装并完成初始化')
    } catch (error) {
      setInstallError(String(error))
      setAvailability('missing')
    } finally {
      setInstalling(false)
    }
  }

  const selectDownloadParent = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择 SteamCMD 的上级目录（程序将创建 SteamCMD 子目录）',
      })
      if (selected) await runInstall(selected)
    } catch (error) {
      setInstallError(`无法选择安装目录：${String(error)}`)
    }
  }

  const percent = progress.totalBytes && progress.totalBytes > 0
    ? Math.min(100, Math.round(progress.downloadedBytes / progress.totalBytes * 100))
    : null

  const modalFooter = installing ? null : installError ? (
    <Space wrap>
      <Button onClick={() => setGuideOpen(false)}>稍后配置</Button>
      <Button icon={<FolderOpenOutlined />} onClick={() => void selectExistingDirectory()}>选择已有目录</Button>
      <Button onClick={() => void selectDownloadParent()}>重新选择目录</Button>
      <Button type="primary" icon={<CloudDownloadOutlined />} disabled={!lastParentPath} onClick={() => lastParentPath && void runInstall(lastParentPath)}>重试</Button>
    </Space>
  ) : (
    <Space wrap>
      <Button onClick={() => setGuideOpen(false)}>稍后配置</Button>
      <Button loading={checkingDirectory} icon={<FolderOpenOutlined />} onClick={() => void selectExistingDirectory()}>选择已有 SteamCMD</Button>
      <Button type="primary" icon={<CloudDownloadOutlined />} onClick={() => void selectDownloadParent()}>下载 SteamCMD</Button>
    </Space>
  )

  return (
    <>
      {contextHolder}
      {availability === 'missing' && !guideOpen && (
        <Alert
          className="steamcmd-missing-alert"
          type="warning"
          showIcon
          title="SteamCMD 尚未配置，服务器安装与更新功能暂不可用"
          description={`当前检查目录：${settings.steamCmdPath}`}
          action={<Button size="small" type="primary" onClick={() => setGuideOpen(true)}>立即配置</Button>}
        />
      )}

      <Modal
        width={620}
        open={guideOpen}
        closable={!installing}
        maskClosable={false}
        keyboard={!installing}
        onCancel={() => !installing && setGuideOpen(false)}
        footer={modalFooter}
        title={<Space><WarningOutlined className="steamcmd-warning-icon" />未检测到 SteamCMD</Space>}
      >
        {installing ? (
          <div className="steamcmd-install-progress">
            <div className="steamcmd-install-progress__heading">
              <div className="steamcmd-install-progress__icon">
                {progress.phase === 'completed' ? <CheckCircleOutlined /> : <CloudDownloadOutlined />}
              </div>
              <div>
                <Title level={5}>{phaseLabel(progress.phase)}</Title>
                <Text type="secondary">{progress.message}</Text>
              </div>
              {progress.phase !== 'downloading' && progress.phase !== 'completed' && <Spin size="small" />}
            </div>
            <Progress
              percent={percent ?? (progress.phase === 'downloading' ? 0 : 100)}
              showInfo={percent !== null}
              status={progress.phase === 'completed' ? 'success' : 'active'}
              strokeColor="#13b8ff"
            />
            <div className="steamcmd-progress-stats">
              <div><span>下载速度</span><strong>{progress.bytesPerSecond > 0 ? `${formatBytes(progress.bytesPerSecond)}/s` : '—'}</strong></div>
              <div><span>已下载</span><strong>{formatBytes(progress.downloadedBytes)}</strong></div>
              <div><span>总大小</span><strong>{formatBytes(progress.totalBytes)}</strong></div>
            </div>
            <Text className="steamcmd-progress-note" type="secondary">安装期间不会显示 SteamCMD 控制台窗口，请勿关闭应用。</Text>
          </div>
        ) : (
          <div className="steamcmd-guide-content">
            <Text>全局设置中的目录未找到可用的 <code>steamcmd.exe</code>：</Text>
            <div className="steamcmd-current-path">{settings.steamCmdPath}</div>
            {installError ? (
              <Alert type="error" showIcon title="SteamCMD 安装失败" description={installError} />
            ) : (
              <Alert type="info" showIcon title="请选择已有 SteamCMD 目录，或选择一个上级目录，由管理器自动创建 SteamCMD 子目录并静默安装。" />
            )}
          </div>
        )}
      </Modal>
    </>
  )
}
