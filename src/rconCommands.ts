export interface RconCommandDoc {
  command: string
  signature: string
  insertText: string
  description: string
  example: string
  category: '状态查询' | '服务器公告' | '运维控制' | '玩家管理'
  tone?: 'danger' | 'warning' | 'default'
}

export const RCON_COMMANDS: RconCommandDoc[] = [
  {
    command: 'ListPlayers',
    signature: 'ListPlayers',
    insertText: 'ListPlayers',
    description: '列出当前在线玩家及其玩家 ID，用于确认服务器在线与玩家列表。',
    example: 'ListPlayers',
    category: '状态查询',
  },
  {
    command: 'GetGameLog',
    signature: 'GetGameLog',
    insertText: 'GetGameLog',
    description: '读取 ASA 游戏日志缓冲区，适合快速查看最近聊天、加入退出等服务端事件。',
    example: 'GetGameLog',
    category: '状态查询',
  },
  {
    command: 'Broadcast',
    signature: 'Broadcast <消息>',
    insertText: 'Broadcast ',
    description: '向服务器内所有玩家发送屏幕广播消息。',
    example: 'Broadcast 服务器将在 10 分钟后维护',
    category: '服务器公告',
  },
  {
    command: 'ServerChat',
    signature: 'ServerChat <消息>',
    insertText: 'ServerChat ',
    description: '以服务器身份向聊天频道发送消息。',
    example: 'ServerChat 欢迎来到 ASA 服务器',
    category: '服务器公告',
  },
  {
    command: 'SetMessageOfTheDay',
    signature: 'SetMessageOfTheDay <消息>',
    insertText: 'SetMessageOfTheDay ',
    description: '设置玩家进入服务器时显示的每日消息。',
    example: 'SetMessageOfTheDay 欢迎，请遵守服务器规则',
    category: '服务器公告',
  },
  {
    command: 'ShowMessageOfTheDay',
    signature: 'ShowMessageOfTheDay',
    insertText: 'ShowMessageOfTheDay',
    description: '向在线玩家展示当前每日消息。',
    example: 'ShowMessageOfTheDay',
    category: '服务器公告',
  },
  {
    command: 'SaveWorld',
    signature: 'SaveWorld',
    insertText: 'SaveWorld',
    description: '立即保存世界存档；维护、重启或停服前建议先执行。',
    example: 'SaveWorld',
    category: '运维控制',
  },
  {
    command: 'DestroyWildDinos',
    signature: 'DestroyWildDinos',
    insertText: 'DestroyWildDinos',
    description: '清理野生恐龙并触发生态刷新，常用于更新后重刷野生生物。',
    example: 'DestroyWildDinos',
    category: '运维控制',
    tone: 'warning',
  },
  {
    command: 'SetTimeOfDay',
    signature: 'SetTimeOfDay <HH:MM[:SS]>',
    insertText: 'SetTimeOfDay ',
    description: '设置游戏内时间，格式示例 12:00 或 18:30:00。',
    example: 'SetTimeOfDay 12:00',
    category: '运维控制',
  },
  {
    command: 'DoExit',
    signature: 'DoExit',
    insertText: 'DoExit',
    description: '请求服务端正常退出。建议先执行 SaveWorld，确认无人操作后再使用。',
    example: 'SaveWorld -> DoExit',
    category: '运维控制',
    tone: 'danger',
  },
  {
    command: 'KickPlayer',
    signature: 'KickPlayer <玩家ID>',
    insertText: 'KickPlayer ',
    description: '按 ListPlayers 返回的玩家 ID 踢出玩家。',
    example: 'KickPlayer 1234567890',
    category: '玩家管理',
    tone: 'warning',
  },
  {
    command: 'BanPlayer',
    signature: 'BanPlayer <玩家ID>',
    insertText: 'BanPlayer ',
    description: '按玩家 ID 封禁玩家。',
    example: 'BanPlayer 1234567890',
    category: '玩家管理',
    tone: 'danger',
  },
  {
    command: 'UnBanPlayer',
    signature: 'UnBanPlayer <玩家ID>',
    insertText: 'UnBanPlayer ',
    description: '解除玩家 ID 的封禁。',
    example: 'UnBanPlayer 1234567890',
    category: '玩家管理',
  },
  {
    command: 'AllowPlayerToJoinNoCheck',
    signature: 'AllowPlayerToJoinNoCheck <玩家ID>',
    insertText: 'AllowPlayerToJoinNoCheck ',
    description: '将玩家加入白名单/免检查加入列表。',
    example: 'AllowPlayerToJoinNoCheck 1234567890',
    category: '玩家管理',
  },
  {
    command: 'DisallowPlayerToJoinNoCheck',
    signature: 'DisallowPlayerToJoinNoCheck <玩家ID>',
    insertText: 'DisallowPlayerToJoinNoCheck ',
    description: '将玩家从白名单/免检查加入列表中移除。',
    example: 'DisallowPlayerToJoinNoCheck 1234567890',
    category: '玩家管理',
  },
]

export function normalizeRconInput(input: string) {
  const trimmed = input.trim()
  return trimmed.startsWith('/') ? trimmed.slice(1).trim() : trimmed
}
