import type { JobProgress } from './types'

export function isActiveJobProgress(progress?: JobProgress) {
  return Boolean(progress && !['completed', 'cancelled', 'failed'].includes(progress.phase))
}

function formatJobBytes(value: number | null | undefined) {
  if (!Number.isFinite(value) || !value || value <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = value
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex += 1
  }
  const digits = size >= 100 || unitIndex === 0 ? 0 : size >= 10 ? 1 : 2
  return `${size.toFixed(digits)} ${units[unitIndex]}`
}

function phaseText(phase: string) {
  const labels: Record<string, string> = {
    preparing: '准备中',
    running: '下载/更新',
    downloading: '下载/更新',
    verifying: '校验中',
    preallocating: '预分配',
    committing: '写入中',
  }
  return labels[phase] ?? phase
}

export function parseSteamCmdProgressLog(message: string) {
  const match = message.match(/progress\s*:\s*([\d.]+)\s*\(\s*([\d,]+)\s*\/\s*([\d,]+)\s*\)/i)
  if (!match) return null
  const percent = Number.parseFloat(match[1])
  const downloadedBytes = Number.parseInt(match[2].replaceAll(',', ''), 10)
  const totalBytes = Number.parseInt(match[3].replaceAll(',', ''), 10)
  if (!Number.isFinite(percent) || !Number.isFinite(downloadedBytes)) return null

  const lowerMessage = message.toLowerCase()
  const phase = lowerMessage.includes('download')
    ? 'downloading'
    : lowerMessage.includes('validat') || lowerMessage.includes('verify')
      ? 'verifying'
      : lowerMessage.includes('prealloc')
        ? 'preallocating'
        : lowerMessage.includes('commit')
          ? 'committing'
          : 'running'

  return {
    phase,
    percent,
    downloadedBytes,
    totalBytes: totalBytes > 0 ? totalBytes : null,
  }
}

export function InstanceJobProgress({ progress }: { progress?: JobProgress }) {
  if (!progress) return null
  const totalKnown = Number.isFinite(progress.totalBytes) && (progress.totalBytes ?? 0) > 0
  const downloadedBytes = progress.downloadedBytes ?? 0
  const bytesPerSecond = progress.bytesPerSecond ?? 0
  const hasTransferInfo = downloadedBytes > 0 || totalKnown
  const percent = progress.percent != null ? `${Math.max(0, Math.min(100, progress.percent)).toFixed(1)}%` : '--'
  const speedText = bytesPerSecond > 0 ? `${formatJobBytes(bytesPerSecond)}/s` : hasTransferInfo ? '0 B/s' : '--'
  const downloadedText = downloadedBytes > 0 ? formatJobBytes(downloadedBytes) : '--'
  const totalText = totalKnown ? formatJobBytes(progress.totalBytes) : '--'

  return (
    <div className="instance-job-progress" onClick={(event) => event.stopPropagation()}>
      <div className="instance-job-progress__line">
        <span className="instance-job-progress__phase">{phaseText(progress.phase)}</span>
        <span>进度 <b>{percent}</b></span>
        <span>速度 <b>{speedText}</b></span>
        <span>已下载 <b>{downloadedText}</b></span>
        <span>总大小 <b>{totalText}</b></span>
      </div>
    </div>
  )
}

export function mergeJobProgress(previous: JobProgress | undefined, next: JobProgress): JobProgress {
  const previousHasBytes = (previous?.downloadedBytes ?? 0) > 0 || (previous?.totalBytes ?? 0) > 0
  const nextHasBytes = (next.downloadedBytes ?? 0) > 0 || (next.totalBytes ?? 0) > 0

  if (previous && previousHasBytes && !nextHasBytes) {
    return {
      ...previous,
      jobId: next.jobId,
      phase: next.phase,
      message: next.message,
      detail: next.detail ?? previous.detail,
    }
  }

  return {
    ...next,
    percent: next.percent ?? previous?.percent ?? null,
    downloadedBytes: next.downloadedBytes > 0 ? next.downloadedBytes : previous?.downloadedBytes ?? 0,
    totalBytes: next.totalBytes && next.totalBytes > 0 ? next.totalBytes : previous?.totalBytes ?? null,
    bytesPerSecond: nextHasBytes ? Math.max(0, next.bytesPerSecond) : previous?.bytesPerSecond ?? 0,
  }
}
