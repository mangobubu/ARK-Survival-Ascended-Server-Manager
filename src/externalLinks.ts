import { openUrl } from '@tauri-apps/plugin-opener'
import { isTauriRuntime } from './runtime'

export async function openExternalLink(url: string): Promise<void> {
  const parsedUrl = new URL(url)
  if (parsedUrl.protocol !== 'https:') {
    throw new Error('仅允许打开 HTTPS 外部链接')
  }

  if (isTauriRuntime()) {
    await openUrl(parsedUrl.toString())
    return
  }

  window.open(parsedUrl.toString(), '_blank', 'noopener,noreferrer')
}
