import type { KeyboardEvent } from 'react'
import type { WebIpWhitelistEntry } from './types'

export const closeBehaviorOptions = [
  { value: 'askEveryTime', label: '每次询问' },
  { value: 'minimizeToTray', label: '最小化托盘' },
  { value: 'exitApp', label: '退出应用' },
]

export const chinaMainlandIpToken = 'CN_MAINLAND'

export const defaultWebIpWhitelistEntry: WebIpWhitelistEntry = {
  value: chinaMainlandIpToken,
  group: '默认',
  note: '内置中国大陆 IPv4 CIDR',
}

export function normalizeWebIpWhitelist(value: unknown): WebIpWhitelistEntry[] {
  if (!Array.isArray(value)) return [defaultWebIpWhitelistEntry]
  const seen = new Set<string>()
  const normalized = value
    .map((item): WebIpWhitelistEntry | null => {
      if (typeof item === 'string') {
        const raw = item.trim()
        if (!raw) return null
        return {
          value: raw.toUpperCase() === chinaMainlandIpToken ? chinaMainlandIpToken : raw,
          group: '',
          note: '',
        }
      }
      if (item && typeof item === 'object') {
        const candidate = item as Partial<WebIpWhitelistEntry>
        const raw = String(candidate.value ?? '').trim()
        if (!raw) return null
        return {
          value: raw.toUpperCase() === chinaMainlandIpToken ? chinaMainlandIpToken : raw,
          group: String(candidate.group ?? '').trim(),
          note: String(candidate.note ?? '').trim(),
        }
      }
      return null
    })
    .filter((item): item is WebIpWhitelistEntry => Boolean(item))
    .filter((item) => {
      if (seen.has(item.value)) return false
      seen.add(item.value)
      return true
    })
  return normalized.length > 0 ? normalized : [defaultWebIpWhitelistEntry]
}

function isValidIpv4(ip: string) {
  const parts = ip.split('.')
  return parts.length === 4 && parts.every((part) => {
    if (!/^\d{1,3}$/.test(part)) return false
    const value = Number(part)
    return value >= 0 && value <= 255
  })
}

export function isValidWebIpWhitelistEntry(entry: string) {
  if (entry.toUpperCase() === chinaMainlandIpToken) return true
  if (isValidIpv4(entry)) return true
  const [ip, prefix, extra] = entry.split('/')
  if (extra !== undefined || !ip || prefix === undefined) return false
  const prefixValue = Number(prefix)
  return isValidIpv4(ip) && /^\d{1,2}$/.test(prefix) && prefixValue >= 0 && prefixValue <= 32
}

export function webIpWhitelistSignature(value: unknown) {
  return JSON.stringify(normalizeWebIpWhitelist(value))
}

export function formatBanSource(source: string) {
  const labels: Record<string, string> = {
    login: '登录失败',
    ua: '异常 UA',
    body: '危险请求体',
    path: '路径探测',
    rate: '频率限制',
    security: '安全策略',
  }
  return labels[source] ?? source
}

export function formatRemainingSeconds(seconds: number) {
  if (seconds <= 0) return '即将过期'
  if (seconds < 60) return `${seconds} 秒`
  const minutes = Math.floor(seconds / 60)
  const restSeconds = seconds % 60
  if (minutes < 60) return restSeconds > 0 ? `${minutes} 分 ${restSeconds} 秒` : `${minutes} 分`
  const hours = Math.floor(minutes / 60)
  const restMinutes = minutes % 60
  return restMinutes > 0 ? `${hours} 小时 ${restMinutes} 分` : `${hours} 小时`
}

export function formatBanTime(timestamp: number) {
  if (!timestamp) return '未知时间'
  return new Date(timestamp).toLocaleString('zh-CN', { hour12: false })
}

export type ShortcutParseResult =
  | { type: 'key'; value: string }
  | { type: 'modifier' }
  | { type: 'unsupported' }

export function normalizeShortcutEvent(event: KeyboardEvent<HTMLInputElement>): ShortcutParseResult {
  const code = event.code || ''
  const key = event.key || ''
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(key)) return { type: 'modifier' }
  if (/^Key[A-Z]$/.test(code)) return { type: 'key', value: code.replace('Key', '') }
  if (/^Digit[0-9]$/.test(code)) return { type: 'key', value: code.replace('Digit', '') }
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(key.toUpperCase())) return { type: 'key', value: key.toUpperCase() }

  const namedKeys: Record<string, string> = {
    Escape: 'ESC',
    ' ': 'SPACE',
    Spacebar: 'SPACE',
    ArrowUp: 'UP',
    ArrowDown: 'DOWN',
    ArrowLeft: 'LEFT',
    ArrowRight: 'RIGHT',
    Home: 'HOME',
    End: 'END',
    PageUp: 'PAGEUP',
    PageDown: 'PAGEDOWN',
    Insert: 'INSERT',
    Delete: 'DELETE',
  }
  const namedKey = namedKeys[key]
  return namedKey ? { type: 'key', value: namedKey } : { type: 'unsupported' }
}

export function formatShortcutKey(key?: string) {
  if (!key) return 'A'
  const labels: Record<string, string> = {
    ESC: 'Esc',
    SPACE: 'Space',
    UP: '↑',
    DOWN: '↓',
    LEFT: '←',
    RIGHT: '→',
    PAGEUP: 'PageUp',
    PAGEDOWN: 'PageDown',
    INSERT: 'Insert',
    DELETE: 'Delete',
  }
  return labels[key] ?? key
}
