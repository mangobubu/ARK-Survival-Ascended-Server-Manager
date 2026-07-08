import type { ReactNode } from 'react'
import { Empty } from 'antd'
import type { ServerConfig, ServerStatus } from './types'

export const mapGlyphs: Record<string, string> = {
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

export function Logo() {
  return (
    <div className="brand">
      <img className="brand__emblem" src="/app-icon.png" alt="ASA 服务器管理器" />
      <div><div className="brand__title">方舟进化飞升服务器管理器</div><div className="brand__subtitle">ARK SURVIVAL ASCENDED SERVER MANAGER</div></div>
    </div>
  )
}

export function StatCard({
  icon,
  label,
  value,
  suffix,
  tone = 'blue',
}: {
  icon: ReactNode
  label: string
  value: string | number
  suffix: string
  tone?: 'blue' | 'green'
}) {
  return (
    <div className={`stat-card stat-card--${tone}`}>
      <div className="stat-card__icon">{icon}</div>
      <div><div className="stat-card__label">{label}</div><div className="stat-card__value">{value} <small>{suffix}</small></div></div>
    </div>
  )
}

export function statusMeta(status: ServerStatus) {
  if (status === 'running') return { color: 'success', text: '⊙ 运行中' }
  if (status === 'stopping') return { color: 'processing', text: '◍ 停止中' }
  if (status === 'starting') return { color: 'processing', text: '◌ 启动中' }
  if (status === 'updating') return { color: 'processing', text: '↻ 更新中' }
  if (status === 'backingUp') return { color: 'processing', text: '▣ 备份中' }
  if (status === 'error') return { color: 'error', text: '⊗ 异常' }
  return { color: 'default', text: '⊖ 已停止' }
}

export function canDeleteInstance(status: ServerStatus) {
  return status === 'stopped' || status === 'error'
}

export function formatServerVersion(serverVersion?: string | null, versionState?: string) {
  const normalized = serverVersion?.trim()
  if (normalized) return normalized.toLowerCase().startsWith('v') ? normalized : `v${normalized}`
  return versionState === '未安装' ? '未安装' : '未识别'
}

export function enforceRequiredRconConfig(config: ServerConfig): ServerConfig {
  return {
    ...config,
    rconEnabled: true,
    adminPassword: config.adminPassword.trim(),
  }
}

export function PanelLoading({ text = '正在加载配置面板...' }: { text?: string }) {
  return (
    <section className="surface config-panel">
      <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={text} />
    </section>
  )
}
