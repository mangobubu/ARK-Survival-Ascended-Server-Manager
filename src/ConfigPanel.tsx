import { useEffect, useMemo, useRef, useState } from 'react'
import {
  ApartmentOutlined,
  ArrowDownOutlined,
  ArrowUpOutlined,
  BugOutlined,
  CloudSyncOutlined,
  CodeOutlined,
  DeleteOutlined,
  FileSearchOutlined,
  FolderOpenOutlined,
  HistoryOutlined,
  LinkOutlined,
  PlusOutlined,
  ReloadOutlined,
  SafetyCertificateOutlined,
  SearchOutlined,
  SettingOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons'
import {
  Alert,
  Avatar,
  Button,
  Checkbox,
  Empty,
  Input,
  Modal,
  Pagination,
  Progress,
  Radio,
  Segmented,
  Select,
  Space,
  Spin,
  Switch,
  Tabs,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import { activeEventOptions } from './data'
import { searchCurseForgeMods } from './backendApi'
import { arkStackableItemOptions } from './arkStackableItemOptions'
import {
  AccordionGroup,
  Field,
  NumberField,
  SectionCard,
  filterSearchNode,
  hasRenderableNode,
  normalizeSearchText,
} from './configPanelLayout'
import {
  ItemStackOverrideModal,
  getItemStackItemLabel,
  itemStackLanguageText,
} from './itemStackOverrideEditor'
import type {
  CurseForgeModSearchResult,
  CurseForgeModSummary,
  GlobalSettings,
  ModItem,
  ServerConfig,
  ServerInstance,
} from './types'

const { Text, Paragraph } = Typography

type AppLanguage = GlobalSettings['language']

const CURSEFORGE_PAGE_SIZE = 20
const EMPTY_CURSEFORGE_RESULT: CurseForgeModSearchResult = { items: [], totalCount: 0 }

function formatDownloadCount(value: number) {
  return new Intl.NumberFormat('zh-CN', { notation: 'compact', maximumFractionDigits: 1 }).format(value)
}

function formatModDate(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '未知时间'
  return new Intl.DateTimeFormat('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' }).format(date)
}

interface ConfigPanelProps {
  instance: ServerInstance
  config: ServerConfig
  mods: ModItem[]
  dirty: boolean
  language: AppLanguage
  onConfigChange: <K extends keyof ServerConfig>(key: K, value: ServerConfig[K]) => void
  onModsChange: (mods: ModItem[]) => void
  onSave: () => void
  onApply: () => void
  onCheckModUpdates: () => void
  checkingMods?: boolean
  configOperation?: 'save' | 'apply' | null
  actionsDisabled?: boolean
}

export default function ConfigPanel({ instance, config, mods, dirty, language, onConfigChange, onModsChange, onSave, onApply, onCheckModUpdates, checkingMods = false, configOperation = null, actionsDisabled = false }: ConfigPanelProps) {
  const [modModalOpen, setModModalOpen] = useState(false)
  const [itemStackModalOpen, setItemStackModalOpen] = useState(false)
  const [modSearch, setModSearch] = useState('')
  const [modSearchPage, setModSearchPage] = useState(1)
  const [modCatalog, setModCatalog] = useState<CurseForgeModSearchResult>(EMPTY_CURSEFORGE_RESULT)
  const [modCatalogLoading, setModCatalogLoading] = useState(false)
  const [modCatalogError, setModCatalogError] = useState('')
  const [modCatalogReload, setModCatalogReload] = useState(0)
  const [selectedCatalogMods, setSelectedCatalogMods] = useState<CurseForgeModSummary[]>([])
  const modRequestSequence = useRef(0)
  const [activeTab, setActiveTab] = useState('basic')
  const [configSearch, setConfigSearch] = useState('')
  const set = onConfigChange
  const isAberration = instance.mapCode === 'Aberration_WP'
  const isLostColony = instance.mapCode === 'LostColony_WP'
  const normalizedConfigSearch = normalizeSearchText(configSearch)
  const configSearchActive = normalizedConfigSearch.length > 0
  const hasJoinPassword = config.serverPassword.trim().length > 0
  const hasExclusiveAccess = config.whitelist || config.exclusiveJoin
  const privateVisibilityNeedsAccess = config.visibility === 'private' && !hasJoinPassword && !hasExclusiveAccess
  const itemStackOverrides = config.itemStackOverrides ?? []
  const itemStackText = itemStackLanguageText[language]
  const existingModIds = useMemo(() => new Set(mods.map((mod) => mod.id)), [mods])
  const selectedCatalogModIds = useMemo(
    () => new Set(selectedCatalogMods.map((mod) => mod.id)),
    [selectedCatalogMods],
  )
  const selectableCatalogMods = modCatalog.items.filter((mod) => !existingModIds.has(mod.id))
  const selectedOnCurrentPage = selectableCatalogMods.filter((mod) => selectedCatalogModIds.has(mod.id)).length
  const allCurrentPageSelected = selectableCatalogMods.length > 0
    && selectedOnCurrentPage === selectableCatalogMods.length

  useEffect(() => {
    if (!modModalOpen) return
    const requestId = ++modRequestSequence.current
    let cancelled = false
    setModCatalogLoading(true)
    setModCatalogError('')
    const timer = window.setTimeout(() => {
      void searchCurseForgeMods(
        modSearch.trim(),
        (modSearchPage - 1) * CURSEFORGE_PAGE_SIZE,
        CURSEFORGE_PAGE_SIZE,
      ).then((result) => {
        if (cancelled || requestId !== modRequestSequence.current) return
        setModCatalog(result)
      }).catch((error) => {
        if (cancelled || requestId !== modRequestSequence.current) return
        setModCatalog(EMPTY_CURSEFORGE_RESULT)
        setModCatalogError(String(error))
      }).finally(() => {
        if (!cancelled && requestId === modRequestSequence.current) setModCatalogLoading(false)
      })
    }, modSearch.trim() ? 350 : 0)

    return () => {
      cancelled = true
      window.clearTimeout(timer)
    }
  }, [modCatalogReload, modModalOpen, modSearch, modSearchPage])

  const openItemStackModal = () => {
    setItemStackModalOpen(true)
  }

  const openModModal = () => {
    setModSearch('')
    setModSearchPage(1)
    setModCatalog(EMPTY_CURSEFORGE_RESULT)
    setModCatalogError('')
    setSelectedCatalogMods([])
    setModModalOpen(true)
  }

  const closeModModal = () => {
    modRequestSequence.current += 1
    setModModalOpen(false)
  }

  const toggleCatalogMod = (item: CurseForgeModSummary) => {
    if (existingModIds.has(item.id)) return
    setSelectedCatalogMods((current) => current.some((mod) => mod.id === item.id)
      ? current.filter((mod) => mod.id !== item.id)
      : [...current, item])
  }

  const toggleCurrentCatalogPage = () => {
    const currentPageIds = new Set(selectableCatalogMods.map((mod) => mod.id))
    if (allCurrentPageSelected) {
      setSelectedCatalogMods((current) => current.filter((mod) => !currentPageIds.has(mod.id)))
      return
    }
    setSelectedCatalogMods((current) => {
      const selectedIds = new Set(current.map((mod) => mod.id))
      return [...current, ...selectableCatalogMods.filter((mod) => !selectedIds.has(mod.id))]
    })
  }

  const addSelectedMods = () => {
    const additions: ModItem[] = selectedCatalogMods
      .filter((item) => !existingModIds.has(item.id))
      .map((item) => ({
        id: item.id,
        name: item.name,
        version: item.version,
        size: item.size,
        enabled: true,
      }))
    if (additions.length === 0) return
    onModsChange([...mods, ...additions])
    closeModModal()
  }

  const moveMod = (index: number, direction: -1 | 1) => {
    const target = index + direction
    if (target < 0 || target >= mods.length) return
    const next = [...mods]
    ;[next[index], next[target]] = [next[target], next[index]]
    onModsChange(next)
  }


  const basicTab = (
    <AccordionGroup forceExpand={configSearchActive}>
      <SectionCard title="网络与远程管理" icon={<ApartmentOutlined />}>
        <Field label="服务器名称" tip="[SessionSettings] SessionName" wide><Input value={config.sessionName} onChange={(e) => set('sessionName', e.target.value)} /></Field>
        <div className="field-pair">
          <Field label="游戏端口" tip="[SessionSettings] Port"><NumberField value={config.gamePort} min={1024} max={65535} onChange={(v) => set('gamePort', v)} /></Field>
          <Field label="查询端口" tip="Steam 查询端口"><NumberField value={config.queryPort} min={1024} max={65535} onChange={(v) => set('queryPort', v)} /></Field>
        </div>
        <Field label="RCON 远程控制" tip="管理器需要通过 RCON 判断启动完成并执行保存、停止命令"><Switch checked disabled /></Field>
        <Field label="RCON 端口"><NumberField value={config.rconPort} min={1024} max={65535} onChange={(v) => set('rconPort', v)} /></Field>
        <Field label="加入服务器密码"><Input.Password value={config.serverPassword} onChange={(e) => set('serverPassword', e.target.value)} placeholder="留空表示无密码" /></Field>
        <Field label="管理员密码" tip="必填，用于 RCON 状态探测与安全停服"><Input.Password value={config.adminPassword} status={config.adminPassword.trim() ? undefined : 'error'} onChange={(e) => set('adminPassword', e.target.value)} placeholder="必填" /></Field>
        <Field label="观察者密码"><Input.Password value={config.spectatorPassword} onChange={(e) => set('spectatorPassword', e.target.value)} placeholder="留空表示不启用" /></Field>
        <Field
          label="服务器可见性"
          tip="ASA 官方 Server configuration 没有独立的免密码隐藏服务器参数；私有模式必须配合加入密码或 Exclusive Join 白名单才有真实准入效果。"
        >
          <Radio.Group
            value={config.visibility}
            onChange={(e) => set('visibility', e.target.value)}
            optionType="button"
            buttonStyle="solid"
            options={[{ label: '公开', value: 'public' }, { label: '私有', value: 'private' }]}
          />
        </Field>
        {privateVisibilityNeedsAccess && (
          <Alert
            type="error"
            showIcon
            title="私有可见性需要加入密码或 Exclusive Join"
            description="仅选择“私有”不会产生一个免密码且不可直连的 ASA 服务端；不设置加入密码、不启用白名单时，拿到服务器地址的玩家仍可能加入。"
          />
        )}
        <Field label="集群名称 Cluster ID" wide><Input value={config.clusterId} onChange={(e) => set('clusterId', e.target.value)} /></Field>
        <Field label="允许跨服传送"><Switch checked={config.crossTransfer} onChange={(v) => set('crossTransfer', v)} /></Field>
      </SectionCard>

      <SectionCard title="游戏规则与倍率" icon={<SafetyCertificateOutlined />}>
        <div className="field-pair">
          <Field label="游戏模式"><Segmented value={config.pve ? 'pve' : 'pvp'} onChange={(v) => set('pve', v === 'pve')} options={[{ label: 'PvE', value: 'pve' }, { label: 'PvP', value: 'pvp' }]} /></Field>
          <Field label="最大玩家"><NumberField value={config.maxPlayers} min={1} max={250} onChange={(v) => set('maxPlayers', v)} /></Field>
        </div>
        <Field label="难度覆盖" tip="OverrideOfficialDifficulty；5.0 通常对应野生恐龙最高 150 级"><NumberField value={config.difficulty} min={0.1} max={50} step={0.5} onChange={(v) => set('difficulty', v)} addonAfter="×30级" /></Field>
        <div className="field-pair">
          <Field label="经验倍率"><NumberField value={config.xpMultiplier} min={0.1} max={100} step={0.1} onChange={(v) => set('xpMultiplier', v)} addonAfter="x" /></Field>
          <Field label="驯服倍率"><NumberField value={config.tamingSpeed} min={0.1} max={100} step={0.5} onChange={(v) => set('tamingSpeed', v)} addonAfter="x" /></Field>
        </div>
        <Field label="采集数量倍率" tip="HarvestAmountMultiplier；写入 GameUserSettings.ini，并同步加入启动参数兜底。ASA 官方 1x 若显示 +2，设置 5x 理论应为 +10。"><NumberField value={config.harvestAmount} min={0.1} max={100} step={0.5} onChange={(v) => set('harvestAmount', v)} addonAfter="x" /></Field>
        <Field label="采集物耐久倍率" tip="HarvestHealthMultiplier；越高可采集的次数越多"><NumberField value={config.harvestHealthMultiplier} min={0.1} max={100} step={0.1} onChange={(v) => set('harvestHealthMultiplier', v)} addonAfter="x" /></Field>
        <Field label="硬核模式"><Switch checked={config.hardcore} onChange={(v) => set('hardcore', v)} /></Field>
        <Field label="PvE 禁用友军伤害" tip="bPvEDisableFriendlyFire；仅 PvE 规则使用"><Switch checked={config.disableFriendlyFire} onChange={(v) => set('disableFriendlyFire', v)} /></Field>
        <Field label="PvP/通用禁用友军伤害" tip="bDisableFriendlyFire；用于 PvP 或通用友军伤害规则"><Switch checked={config.disableFriendlyFirePvP} onChange={(v) => set('disableFriendlyFirePvP', v)} /></Field>
        <Field label="允许 PvP Gamma"><Switch checked={config.enablePvPGamma} onChange={(v) => set('enablePvPGamma', v)} /></Field>
        <Field label="显示命中标记"><Switch checked={config.allowHitMarkers} onChange={(v) => set('allowHitMarkers', v)} /></Field>
        <Field label="允许第三人称"><Switch checked={config.thirdPerson} onChange={(v) => set('thirdPerson', v)} /></Field>
        <Field label="显示准星"><Switch checked={config.crosshair} onChange={(v) => set('crosshair', v)} /></Field>
        <Field label="地图显示玩家位置"><Switch checked={config.showMapPlayer} onChange={(v) => set('showMapPlayer', v)} /></Field>
        <Field label="PvE 飞行生物叼取"><Switch checked={config.flyerCarry} onChange={(v) => set('flyerCarry', v)} /></Field>
      </SectionCard>

      <SectionCard title="聊天与玩家体验" icon={<SafetyCertificateOutlined />} note="ServerSettings / Game.ini">
        <Field label="全局语音" tip="GlobalVoiceChat；开启后语音不受距离限制"><Switch checked={config.globalVoiceChat} onChange={(v) => set('globalVoiceChat', v)} /></Field>
        <Field label="附近文字聊天" tip="ProximityChat；开启后文字聊天按附近距离传播"><Switch checked={config.proximityChat} onChange={(v) => set('proximityChat', v)} /></Field>
        <Field label="显示玩家加入通知" tip="反向写入 DontAlwaysNotifyPlayerJoined；开启本项会写入 DontAlwaysNotifyPlayerJoined=False"><Switch checked={config.showPlayerJoinNotifications} onChange={(v) => set('showPlayerJoinNotifications', v)} /></Field>
        <Field label="显示浮动伤害数字" tip="ShowFloatingDamageText"><Switch checked={config.showFloatingDamageText} onChange={(v) => set('showFloatingDamageText', v)} /></Field>
        <Field label="强制隐藏 HUD" tip="ServerForceNoHUD；开启后服务端强制玩家隐藏 HUD"><Switch checked={config.serverForceNoHud} onChange={(v) => set('serverForceNoHud', v)} /></Field>
        <Field label="战斗日志隐藏伤害来源" tip="AllowHideDamageSourceFromLogs"><Switch checked={config.allowHideDamageSourceFromLogs} onChange={(v) => set('allowHideDamageSourceFromLogs', v)} /></Field>
        <Field label="阻止出生动画" tip="PreventSpawnAnimations"><Switch checked={config.preventSpawnAnimations} onChange={(v) => set('preventSpawnAnimations', v)} /></Field>
        <Field label="禁用拍照模式" tip="bDisablePhotoMode"><Switch checked={config.disablePhotoMode} onChange={(v) => set('disablePhotoMode', v)} /></Field>
        <Field label="拍照模式范围上限" tip="PhotoModeRangeLimit；禁用拍照模式时此值不生效">
          <NumberField disabled={config.disablePhotoMode} value={config.photoModeRangeLimit} min={0} max={100000} step={100} onChange={(v) => set('photoModeRangeLimit', v)} />
        </Field>
      </SectionCard>

      <SectionCard title="伤害、抗性与生存消耗" icon={<SafetyCertificateOutlined />} note="GameUserSettings.ini">
        <Field label="玩家伤害倍率"><NumberField value={config.playerDamageMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('playerDamageMultiplier', v)} addonAfter="x" /></Field>
        <Field label="玩家抗性倍率" tip="数值越高，玩家受到的伤害越低"><NumberField value={config.playerResistanceMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('playerResistanceMultiplier', v)} addonAfter="x" /></Field>
        <Field label="野生生物伤害"><NumberField value={config.dinoDamageMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('dinoDamageMultiplier', v)} addonAfter="x" /></Field>
        <Field label="野生生物抗性"><NumberField value={config.dinoResistanceMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('dinoResistanceMultiplier', v)} addonAfter="x" /></Field>
        <Field label="驯养生物伤害"><NumberField value={config.tamedDinoDamageMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('tamedDinoDamageMultiplier', v)} addonAfter="x" /></Field>
        <Field label="驯养生物抗性"><NumberField value={config.tamedDinoResistanceMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('tamedDinoResistanceMultiplier', v)} addonAfter="x" /></Field>
        <Field label="玩家食物消耗"><NumberField value={config.playerFoodDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('playerFoodDrainMultiplier', v)} addonAfter="x" /></Field>
        <Field label="玩家水分消耗"><NumberField value={config.playerWaterDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('playerWaterDrainMultiplier', v)} addonAfter="x" /></Field>
        <Field label="玩家耐力消耗"><NumberField value={config.playerStaminaDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('playerStaminaDrainMultiplier', v)} addonAfter="x" /></Field>
        <Field label="生物食物消耗"><NumberField value={config.dinoFoodDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('dinoFoodDrainMultiplier', v)} addonAfter="x" /></Field>
        <Field label="生物耐力消耗"><NumberField value={config.dinoStaminaDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('dinoStaminaDrainMultiplier', v)} addonAfter="x" /></Field>
        <Field label="玩家生命恢复" tip="PlayerCharacterHealthRecoveryMultiplier"><NumberField value={config.playerHealthRecoveryMultiplier} min={0} max={100} step={0.1} onChange={(v) => set('playerHealthRecoveryMultiplier', v)} addonAfter="x" /></Field>
        <Field label="生物生命恢复" tip="DinoCharacterHealthRecoveryMultiplier"><NumberField value={config.dinoHealthRecoveryMultiplier} min={0} max={100} step={0.1} onChange={(v) => set('dinoHealthRecoveryMultiplier', v)} addonAfter="x" /></Field>
        <Field label="氧气游泳速度属性倍率" tip="OxygenSwimSpeedStatMultiplier"><NumberField value={config.oxygenSwimSpeedStatMultiplier} min={0} max={100} step={0.1} onChange={(v) => set('oxygenSwimSpeedStatMultiplier', v)} addonAfter="x" /></Field>
      </SectionCard>

      <SectionCard title="资源、物品与战利品" icon={<CloudSyncOutlined />} note="常用倍率">
        <Field label="资源玩家刷新半径"><NumberField value={config.resourceNoReplenishRadiusPlayers} min={0.01} max={100} step={0.1} onChange={(v) => set('resourceNoReplenishRadiusPlayers', v)} addonAfter="x" /></Field>
        <Field label="资源建筑刷新半径"><NumberField value={config.resourceNoReplenishRadiusStructures} min={0.01} max={100} step={0.1} onChange={(v) => set('resourceNoReplenishRadiusStructures', v)} addonAfter="x" /></Field>
        <Field label="作物生长速度"><NumberField value={config.cropGrowthSpeedMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('cropGrowthSpeedMultiplier', v)} addonAfter="x" /></Field>
        <Field label="作物腐坏速度"><NumberField value={config.cropDecaySpeedMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('cropDecaySpeedMultiplier', v)} addonAfter="x" /></Field>
        <Field label="补给箱品质"><NumberField value={config.supplyCrateLootQualityMultiplier} min={0.1} max={5} step={0.1} onChange={(v) => set('supplyCrateLootQualityMultiplier', v)} addonAfter="x" /></Field>
        <Field label="钓鱼战利品品质"><NumberField value={config.fishingLootQualityMultiplier} min={0.1} max={5} step={0.1} onChange={(v) => set('fishingLootQualityMultiplier', v)} addonAfter="x" /></Field>
        <Field label="燃料消耗间隔" tip="越高燃料使用越慢"><NumberField value={config.fuelConsumptionIntervalMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('fuelConsumptionIntervalMultiplier', v)} addonAfter="x" /></Field>
        <Field label="钳制物品腐坏时间" tip="ClampItemSpoilingTimes"><Switch checked={config.clampItemSpoilingTimes} onChange={(v) => set('clampItemSpoilingTimes', v)} /></Field>
        <Field label="钳制资源采集伤害" tip="ClampResourceHarvestDamage"><Switch checked={config.clampResourceHarvestDamage} onChange={(v) => set('clampResourceHarvestDamage', v)} /></Field>
        <Field label="随机补给箱落点" tip="RandomSupplyCratePoints"><Switch checked={config.randomSupplyCratePoints} onChange={(v) => set('randomSupplyCratePoints', v)} /></Field>
        <Field label="物品叠加/堆叠倍率" tip="ItemStackSizeMultiplier；全局物品堆叠倍率，1 表示默认叠加数量"><NumberField value={config.itemStackSizeMultiplier} min={0.1} max={1000} step={0.5} onChange={(v) => set('itemStackSizeMultiplier', v)} addonAfter="x" /></Field>
        <Field label="食物腐坏时间"><NumberField value={config.globalSpoilingTimeMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('globalSpoilingTimeMultiplier', v)} addonAfter="x" /></Field>
        <Field label="掉落物分解时间"><NumberField value={config.globalItemDecompositionTimeMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('globalItemDecompositionTimeMultiplier', v)} addonAfter="x" /></Field>
        <Field label="尸体分解时间"><NumberField value={config.globalCorpseDecompositionTimeMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('globalCorpseDecompositionTimeMultiplier', v)} addonAfter="x" /></Field>
      </SectionCard>

      <SectionCard title="物品单独叠加覆盖" icon={<CodeOutlined />} note="Game.ini">
        <div className="item-stack-toolbar item-stack-toolbar--summary">
          <div className="item-stack-toolbar__summary">
            <Text strong>已配置 {itemStackOverrides.length} 条物品覆盖</Text>
            <span>在子窗口中集中搜索、添加与编辑 ConfigOverrideItemMaxQuantity。</span>
          </div>
          <Button type="primary" icon={<PlusOutlined />} onClick={openItemStackModal}>管理物品覆盖</Button>
        </div>
        {itemStackOverrides.length === 0 ? (
          <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="尚未添加单物品叠加覆盖，点击“管理物品覆盖”在子窗口中添加" />
        ) : (
          <div className="item-stack-summary">
            {itemStackOverrides.slice(0, 5).map((override, index) => {
              const selectedItem = arkStackableItemOptions.find((item) => item.classString === override.itemClassString)
              const selectedItemLabel = selectedItem ? getItemStackItemLabel(selectedItem, language) : (override.itemClassString || itemStackText.unselectedItem)

              return (
                <Tag key={(override.itemClassString || 'empty') + '-' + index} color={override.itemClassString ? 'cyan' : 'default'}>
                  {selectedItemLabel} · {Math.max(1, Math.trunc(override.maxItemQuantity || 1))}
                </Tag>
              )
            })}
            {itemStackOverrides.length > 5 && <Tag color="blue">+{itemStackOverrides.length - 5} 条</Tag>}
          </div>
        )}
      </SectionCard>

      <SectionCard title="自动化与维护" icon={<HistoryOutlined />}>
        <Field label="自动重启"><Switch checked={config.autoRestart} onChange={(v) => set('autoRestart', v)} /></Field>
        <Field label="每日重启时间"><Input type="time" value={config.restartTime} onChange={(e) => set('restartTime', e.target.value)} /></Field>
        <Field label="定时保存间隔"><NumberField value={config.saveInterval} min={5} max={120} onChange={(v) => set('saveInterval', v)} addonAfter="分钟" /></Field>
        <Field label="备份保留天数"><NumberField value={config.backupRetention} min={1} max={90} onChange={(v) => set('backupRetention', v)} addonAfter="天" /></Field>
        <Field label="启动前更新服务端"><Switch checked={config.autoUpdateServer} onChange={(v) => set('autoUpdateServer', v)} /></Field>
        <Field label="启动前更新 MOD"><Switch checked={config.autoUpdateMods} onChange={(v) => set('autoUpdateMods', v)} /></Field>
        <Field label="崩溃后自动重启"><Switch checked={config.restartOnCrash} onChange={(v) => set('restartOnCrash', v)} /></Field>
        <Field label="停止前强制保存"><Switch checked={config.saveOnStop} onChange={(v) => set('saveOnStop', v)} /></Field>
        <Alert type="info" showIcon title="运行中的实例保存配置后，需要重启才能使大多数参数生效。" />
      </SectionCard>
    </AccordionGroup>
  )

  const advancedTab = (
    <AccordionGroup forceExpand={configSearchActive}>
      <SectionCard title="世界节奏与生态" icon={<CloudSyncOutlined />} note="GameUserSettings.ini">
        <div className="field-pair">
          <Field label="昼夜周期速度"><NumberField value={config.dayCycleSpeed} min={0.1} max={10} step={0.1} onChange={(v) => set('dayCycleSpeed', v)} addonAfter="x" /></Field>
          <Field label="白天时间速度"><NumberField value={config.dayTimeSpeed} min={0.1} max={10} step={0.1} onChange={(v) => set('dayTimeSpeed', v)} addonAfter="x" /></Field>
        </div>
        <div className="field-pair">
          <Field label="夜间时间速度"><NumberField value={config.nightTimeSpeed} min={0.1} max={10} step={0.1} onChange={(v) => set('nightTimeSpeed', v)} addonAfter="x" /></Field>
          <Field label="资源刷新周期" tip="数值越小，资源刷新越快"><NumberField value={config.resourceRespawn} min={0.01} max={10} step={0.1} onChange={(v) => set('resourceRespawn', v)} addonAfter="x" /></Field>
        </div>
        <Field label="野生恐龙数量"><NumberField value={config.dinoCount} min={0.1} max={10} step={0.1} onChange={(v) => set('dinoCount', v)} addonAfter="x" /></Field>
        <Field label="最大驯养数量"><NumberField value={config.maxTamedDinos} min={0} max={50000} step={100} onChange={(v) => set('maxTamedDinos', v)} /></Field>
        <Field label="下次启动清除野生恐龙" tip="仅执行一次 -ForceRespawnDinos"><Switch checked={config.destroyWildDinos} onChange={(v) => set('destroyWildDinos', v)} /></Field>
      </SectionCard>

      <SectionCard title="繁殖与成长" icon={<BugOutlined />} note="Game.ini">
        <Field label="交配间隔" tip="数值越小，下一次交配越快"><NumberField value={config.matingInterval} min={0.001} max={100} step={0.05} onChange={(v) => set('matingInterval', v)} addonAfter="x" /></Field>
        <Field label="交配过程速度"><NumberField value={config.matingSpeedMultiplier} min={0.01} max={1000} step={0.1} onChange={(v) => set('matingSpeedMultiplier', v)} addonAfter="x" /></Field>
        <div className="field-pair">
          <Field label="孵化速度"><NumberField value={config.eggHatchSpeed} min={0.1} max={1000} step={1} onChange={(v) => set('eggHatchSpeed', v)} addonAfter="x" /></Field>
          <Field label="幼崽成长速度"><NumberField value={config.babyMatureSpeed} min={0.1} max={1000} step={1} onChange={(v) => set('babyMatureSpeed', v)} addonAfter="x" /></Field>
        </div>
        <Field label="留痕互动间隔" tip="数值越小，留痕互动越频繁"><NumberField value={config.cuddleInterval} min={0.001} max={10} step={0.05} onChange={(v) => set('cuddleInterval', v)} addonAfter="x" /></Field>
        <Field label="幼崽食物消耗"><NumberField value={config.babyFoodConsumption} min={0.01} max={100} step={0.1} onChange={(v) => set('babyFoodConsumption', v)} addonAfter="x" /></Field>
        <Field label="产蛋间隔"><NumberField value={config.layEggIntervalMultiplier} min={0.001} max={100} step={0.1} onChange={(v) => set('layEggIntervalMultiplier', v)} addonAfter="x" /></Field>
        <Field label="留痕宽限期"><NumberField value={config.babyCuddleGracePeriodMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('babyCuddleGracePeriodMultiplier', v)} addonAfter="x" /></Field>
        <Field label="错过留痕损失速度"><NumberField value={config.babyCuddleLoseImprintQualitySpeedMultiplier} min={0} max={100} step={0.1} onChange={(v) => set('babyCuddleLoseImprintQualitySpeedMultiplier', v)} addonAfter="x" /></Field>
        <Field label="留痕属性加成"><NumberField value={config.babyImprintingStatScaleMultiplier} min={0} max={100} step={0.1} onChange={(v) => set('babyImprintingStatScaleMultiplier', v)} addonAfter="x" /></Field>
        <Field
          label="单次留痕量（一次满留痕）"
          tip="对应 Game.ini 的 BabyImprintAmountMultiplier；倍率越高，每次留痕增加越多。想一次留痕留满可设为 100，并将留痕互动间隔同步压到 0.01。"
        >
          <Space wrap>
            <NumberField value={config.babyImprintAmountMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('babyImprintAmountMultiplier', v)} addonAfter="x" />
            <Button
              size="small"
              onClick={() => {
                set('babyImprintAmountMultiplier', 100)
                set('cuddleInterval', 0.01)
              }}
            >
              设为一次满留痕
            </Button>
          </Space>
        </Field>
        <Field label="任何人可照料幼崽"><Switch checked={config.allowAnyoneBabyImprintCuddle} onChange={(v) => set('allowAnyoneBabyImprintCuddle', v)} /></Field>
        <Field label="禁用留痕属性加成" tip="DisableImprintDinoBuff；开启后即使完成留痕，也不应用留痕骑乘属性加成"><Switch checked={config.disableImprintDinoBuff} onChange={(v) => set('disableImprintDinoBuff', v)} /></Field>
        <Field label="阻止配偶加成" tip="PreventMateBoost"><Switch checked={config.preventMateBoost} onChange={(v) => set('preventMateBoost', v)} /></Field>
        <Field label="排泄间隔" tip="PoopIntervalMultiplier；数值越小排泄越频繁"><NumberField value={config.poopIntervalMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('poopIntervalMultiplier', v)} addonAfter="x" /></Field>
        <Field label="野生生物食物消耗" tip="WildDinoCharacterFoodDrainMultiplier"><NumberField value={config.wildDinoFoodDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('wildDinoFoodDrainMultiplier', v)} addonAfter="x" /></Field>
      </SectionCard>

      <SectionCard title="经验与制作" icon={<SettingOutlined />} note="Game.ini">
        <Field label="通用经验倍率" tip="GenericXPMultiplier"><NumberField value={config.genericXpMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('genericXpMultiplier', v)} addonAfter="x" /></Field>
        <Field label="采集经验倍率" tip="HarvestXPMultiplier"><NumberField value={config.harvestXpMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('harvestXpMultiplier', v)} addonAfter="x" /></Field>
        <Field label="击杀经验倍率" tip="KillXPMultiplier"><NumberField value={config.killXpMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('killXpMultiplier', v)} addonAfter="x" /></Field>
        <Field label="特殊经验倍率" tip="SpecialXPMultiplier"><NumberField value={config.specialXpMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('specialXpMultiplier', v)} addonAfter="x" /></Field>
        <Field label="制作经验倍率" tip="CraftXPMultiplier"><NumberField value={config.craftXpMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('craftXpMultiplier', v)} addonAfter="x" /></Field>
        <Field label="制作技能加成倍率" tip="CraftingSkillBonusMultiplier"><NumberField value={config.craftingSkillBonusMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('craftingSkillBonusMultiplier', v)} addonAfter="x" /></Field>
        <Field label="自定义食谱效果倍率" tip="CustomRecipeEffectivenessMultiplier"><NumberField value={config.customRecipeEffectivenessMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('customRecipeEffectivenessMultiplier', v)} addonAfter="x" /></Field>
        <Field label="自定义食谱技能倍率" tip="CustomRecipeSkillMultiplier"><NumberField value={config.customRecipeSkillMultiplier} min={0} max={1000} step={0.1} onChange={(v) => set('customRecipeSkillMultiplier', v)} addonAfter="x" /></Field>
        <Field label="允许无限洗点" tip="bAllowUnlimitedRespecs"><Switch checked={config.allowUnlimitedRespecs} onChange={(v) => set('allowUnlimitedRespecs', v)} /></Field>
        <Field label="显示创造模式" tip="bShowCreativeMode"><Switch checked={config.showCreativeMode} onChange={(v) => set('showCreativeMode', v)} /></Field>
        <Field label="头发生长速度" tip="HairGrowthSpeedMultiplier"><NumberField value={config.hairGrowthSpeedMultiplier} min={0} max={100} step={0.1} onChange={(v) => set('hairGrowthSpeedMultiplier', v)} addonAfter="x" /></Field>
        <Field label="最大坠落速度倍率" tip="MaxFallSpeedMultiplier"><NumberField value={config.maxFallSpeedMultiplier} min={0} max={100} step={0.1} onChange={(v) => set('maxFallSpeedMultiplier', v)} addonAfter="x" /></Field>
      </SectionCard>

      <SectionCard title="建筑、部落与衰减" icon={<ApartmentOutlined />} note="Game.ini / ServerSettings">
        <div className="field-pair">
          <Field label="范围内建筑上限"><NumberField value={config.structureLimit} min={100} max={100000} step={100} onChange={(v) => set('structureLimit', v)} /></Field>
          <Field label="平台建筑倍率"><NumberField value={config.platformStructureMultiplier} min={0.1} max={10} step={0.1} onChange={(v) => set('platformStructureMultiplier', v)} addonAfter="x" /></Field>
        </div>
        <Field label="平台鞍建造区域边界" tip="PlatformSaddleBuildAreaBoundsMultiplier"><NumberField value={config.platformSaddleBuildAreaBoundsMultiplier} min={0.1} max={100} step={0.1} onChange={(v) => set('platformSaddleBuildAreaBoundsMultiplier', v)} addonAfter="x" /></Field>
        <Field label="建筑承伤倍率" tip="StructureResistanceMultiplier；数值越高建筑受到的伤害越高，0.5 表示承受一半伤害"><NumberField value={config.structureResistanceMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('structureResistanceMultiplier', v)} addonAfter="x" /></Field>
        <Field label="禁用建筑碰撞检测"><Switch checked={config.disablePlacementCollision} onChange={(v) => set('disablePlacementCollision', v)} /></Field>
        <Field label="PvE 允许在补给箱附近建筑" tip="仅在 PvE 模式生效；允许玩家在补给箱投放点附近放置建筑">
          <Switch checked={config.pveAllowStructuresAtSupplyDrops} onChange={(v) => set('pveAllowStructuresAtSupplyDrops', v)} />
        </Field>
        <Field label="启用重点资源区建筑禁区" tip="开启后禁止在地图预设的资源富集区建筑，不是禁止在所有资源节点周围建筑">
          <Switch checked={config.enableExtraStructurePreventionVolumes} onChange={(v) => set('enableExtraStructurePreventionVolumes', v)} />
        </Field>
        <Field label="最大部落人数"><NumberField value={config.maxTribeSize} min={0} max={500} onChange={(v) => set('maxTribeSize', v)} /></Field>
        <Field label="允许部落联盟"><Switch checked={config.tribeAlliances} onChange={(v) => set('tribeAlliances', v)} /></Field>
        <Field label="PvE 洞穴建造"><Switch checked={config.allowCaveBuildingPvE} onChange={(v) => set('allowCaveBuildingPvE', v)} /></Field>
        <Field label="PvP 洞穴建造"><Switch checked={config.allowCaveBuildingPvP} onChange={(v) => set('allowCaveBuildingPvP', v)} /></Field>
        <Field label="建筑维修冷却"><NumberField value={config.structureDamageRepairCooldown} min={0} max={86400} onChange={(v) => set('structureDamageRepairCooldown', v)} addonAfter="秒" /></Field>
        <Field label="始终可拾起建筑物" tip="对应 AlwaysAllowStructurePickup；开启后不再限制建筑放置后的快速拾取时间窗口">
          <Switch checked={config.alwaysAllowStructurePickup} onChange={(v) => set('alwaysAllowStructurePickup', v)} />
        </Field>
        <Field label="建筑可拾取时间" tip={config.alwaysAllowStructurePickup ? '已开启始终可拾起建筑物，此时间窗口不再生效' : '建筑放置后允许快速拾取的时间窗口'}>
          <NumberField disabled={config.alwaysAllowStructurePickup} value={config.structurePickupTimeAfterPlacement} min={0} max={86400} onChange={(v) => set('structurePickupTimeAfterPlacement', v)} addonAfter="秒" />
        </Field>
        <Field label="拾取长按时间"><NumberField value={config.structurePickupHoldDuration} min={0} max={60} step={0.1} onChange={(v) => set('structurePickupHoldDuration', v)} addonAfter="秒" /></Field>
        <Field label="旧建筑摧毁周期"><NumberField value={config.autoDestroyOldStructuresMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('autoDestroyOldStructuresMultiplier', v)} addonAfter="x" /></Field>
        <Field label="孤立建筑快速衰减"><Switch checked={config.fastDecayUnsnappedCoreStructures} onChange={(v) => set('fastDecayUnsnappedCoreStructures', v)} /></Field>
        <Field label="区域发电机上限"><NumberField value={config.limitGeneratorsNum} min={0} max={1000} onChange={(v) => set('limitGeneratorsNum', v)} /></Field>
        <Field label="发电机限制范围"><NumberField value={config.limitGeneratorsRange} min={0} max={1000000} step={100} onChange={(v) => set('limitGeneratorsRange', v)} /></Field>
        <Field label="允许附着多个 C4" tip="AllowMultipleAttachedC4"><Switch checked={config.allowMultipleAttachedC4} onChange={(v) => set('allowMultipleAttachedC4', v)} /></Field>
        <Field label="强制锁定所有建筑" tip="ForceAllStructureLocking"><Switch checked={config.forceAllStructureLocking} onChange={(v) => set('forceAllStructureLocking', v)} /></Field>
        <Field label="禁用无线制作" tip="bDisableWirelessCrafting"><Switch checked={config.disableWirelessCrafting} onChange={(v) => set('disableWirelessCrafting', v)} /></Field>
        <Field label="无线制作范围覆盖" tip="WirelessCraftingRangeOverride；禁用无线制作时此值不生效"><NumberField disabled={config.disableWirelessCrafting} value={config.wirelessCraftingRangeOverride} min={0} max={100000} step={100} onChange={(v) => set('wirelessCraftingRangeOverride', v)} /></Field>
        <Field label="平台鞍允许低温冰箱"><Switch checked={config.allowCryoFridgeOnSaddle} onChange={(v) => set('allowCryoFridgeOnSaddle', v)} /></Field>
        <Field label="关闭冷冻舱敌人检测"><Switch checked={config.disableCryopodEnemyCheck} onChange={(v) => set('disableCryopodEnemyCheck', v)} /></Field>
        <Field label="取消低温冰箱要求"><Switch checked={config.disableCryopodFridgeRequirement} onChange={(v) => set('disableCryopodFridgeRequirement', v)} /></Field>
        <Field label="取消冷冻舱冷却"><Switch checked={config.disableCryopodCooldown} onChange={(v) => set('disableCryopodCooldown', v)} /></Field>
      </SectionCard>

      <SectionCard title="生物、飞行与冷冻舱" icon={<BugOutlined />} note="GameUserSettings.ini / Game.ini / 启动参数">
        <Field label="玩家/非飞行生物速度升级" tip="bAllowSpeedLeveling；允许玩家与非飞行生物将属性点投入移动速度，ASA 中飞行生物速度升级也依赖此开关">
          <Switch checked={config.allowSpeedLeveling} onChange={(v) => set('allowSpeedLeveling', v)} />
        </Field>
        <Field label="飞行生物速度升级" tip="AllowFlyerSpeedLeveling；ASA 中还需要同时开启“玩家/非飞行生物速度升级”才会生效">
          <Switch checked={config.allowFlyerSpeedLeveling} onChange={(v) => set('allowFlyerSpeedLeveling', v)} />
        </Field>
        <Field label="强制允许洞穴飞行"><Switch checked={config.forceAllowCaveFlyers} onChange={(v) => set('forceAllowCaveFlyers', v)} /></Field>
        <Field label="骑乘飞行生物恢复耐力"><Switch checked={config.allowFlyingStaminaRecovery} onChange={(v) => set('allowFlyingStaminaRecovery', v)} /></Field>
        <Field label="突袭生物食物消耗"><NumberField value={config.raidDinoFoodDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('raidDinoFoodDrainMultiplier', v)} addonAfter="x" /></Field>
        <Field label="允许喂养突袭生物" tip="AllowRaidDinoFeeding"><Switch checked={config.allowRaidDinoFeeding} onChange={(v) => set('allowRaidDinoFeeding', v)} /></Field>
        <Field label="个人驯养数量上限" tip="MaxPersonalTamedDinos；0 表示不设置个人上限"><NumberField value={config.maxPersonalTamedDinos} min={0} max={100000} step={100} onChange={(v) => set('maxPersonalTamedDinos', v)} /></Field>
        <Field label="驯养生物软上限" tip="MaxTamedDinos_SoftTameLimit"><NumberField value={config.maxTamedDinosSoftTameLimit} min={0} max={100000} step={100} onChange={(v) => set('maxTamedDinosSoftTameLimit', v)} /></Field>
        <Field label="软上限删除倒计时" tip="MaxTamedDinos_SoftTameLimit_CountdownForDeletionDuration；单位秒"><NumberField value={config.maxTamedDinosSoftTameLimitCountdown} min={0} max={31536000} step={3600} onChange={(v) => set('maxTamedDinosSoftTameLimitCountdown', v)} addonAfter="秒" /></Field>
        <Field label="超软上限销毁驯养生物" tip="DestroyTamesOverTheSoftTameLimit；危险：达到软上限后会按倒计时删除驯养生物"><Switch checked={config.destroyTamesOverSoftTameLimit} onChange={(v) => set('destroyTamesOverSoftTameLimit', v)} /></Field>
        <Field label="生物升级动画" tip="bUseDinoLevelUpAnimations"><Switch checked={config.useDinoLevelUpAnimations} onChange={(v) => set('useDinoLevelUpAnimations', v)} /></Field>
      </SectionCard>

      <SectionCard title="玩家、疾病与部落" icon={<SafetyCertificateOutlined />} note="服务器规则">
        <Field label="踢出挂机玩家"><Switch checked={config.enableIdlePlayerKick} onChange={(v) => set('enableIdlePlayerKick', v)} /></Field>
        <Field label="挂机踢出时间"><NumberField value={config.kickIdlePlayersPeriod} min={60} max={86400} step={60} onChange={(v) => set('kickIdlePlayersPeriod', v)} addonAfter="秒" /></Field>
        <Field label="启用疾病"><Switch checked={config.enableDiseases} onChange={(v) => set('enableDiseases', v)} /></Field>
        <Field label="疾病不永久保留"><Switch checked={config.nonPermanentDiseases} onChange={(v) => set('nonPermanentDiseases', v)} /></Field>
        <Field label="部落改名冷却"><NumberField value={config.tribeNameChangeCooldown} min={0} max={365} onChange={(v) => set('tribeNameChangeCooldown', v)} addonAfter="天" /></Field>
        <Field label="每部落联盟上限" tip="0 表示不限制"><NumberField value={config.maxAlliancesPerTribe} min={0} max={1000} onChange={(v) => set('maxAlliancesPerTribe', v)} /></Field>
        <Field label="每联盟部落上限" tip="0 表示不限制"><NumberField value={config.maxTribesPerAlliance} min={0} max={1000} onChange={(v) => set('maxTribesPerAlliance', v)} /></Field>
      </SectionCard>

      <SectionCard title="离线保护与衰减" icon={<HistoryOutlined />} note="ServerSettings">
        <Field label="启用离线突袭保护" tip="PreventOfflinePvP"><Switch checked={config.preventOfflinePvP} onChange={(v) => set('preventOfflinePvP', v)} /></Field>
        <Field label="离线保护生效延迟" tip="PreventOfflinePvPInterval；部落最后一名玩家离线后等待的秒数">
          <NumberField disabled={!config.preventOfflinePvP} value={config.preventOfflinePvPInterval} min={0} max={86400} step={60} onChange={(v) => set('preventOfflinePvPInterval', v)} addonAfter="秒" />
        </Field>
        <Field label="启用 PvE 建筑衰减" tip="反向写入 DisableStructureDecayPvE；开启本项会写入 DisableStructureDecayPvE=False"><Switch checked={config.pveStructureDecay} onChange={(v) => set('pveStructureDecay', v)} /></Field>
        <Field label="启用 PvE 驯养生物衰减" tip="反向写入 DisableDinoDecayPvE；开启本项会写入 DisableDinoDecayPvE=False"><Switch checked={config.pveDinoDecay} onChange={(v) => set('pveDinoDecay', v)} /></Field>
        <Field label="PvE 生物衰减周期" tip="PvEDinoDecayPeriodMultiplier；启用 PvE 驯养生物衰减时生效">
          <NumberField disabled={!config.pveDinoDecay} value={config.pveDinoDecayPeriodMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('pveDinoDecayPeriodMultiplier', v)} addonAfter="x" />
        </Field>
        <Field label="启用 PvP 驯养生物衰减" tip="PvPDinoDecay"><Switch checked={config.pvpDinoDecay} onChange={(v) => set('pvpDinoDecay', v)} /></Field>
      </SectionCard>

      <SectionCard title="准入与跨服传输" icon={<SafetyCertificateOutlined />} note="集群安全">
        <Field label="仅白名单玩家"><Switch checked={config.whitelist} onChange={(v) => set('whitelist', v)} /></Field>
        <Field label="启用 Exclusive Join"><Switch checked={config.exclusiveJoin} onChange={(v) => set('exclusiveJoin', v)} /></Field>
        <div className="transfer-grid">
          <Text type="secondary">下载限制（进入本服）</Text>
          <Checkbox checked={config.preventDownloadItems} onChange={(e) => set('preventDownloadItems', e.target.checked)}>禁止物品</Checkbox>
          <Checkbox checked={config.preventDownloadDinos} onChange={(e) => set('preventDownloadDinos', e.target.checked)}>禁止恐龙</Checkbox>
          <Checkbox checked={config.preventDownloadSurvivors} onChange={(e) => set('preventDownloadSurvivors', e.target.checked)}>禁止角色</Checkbox>
          <Text type="secondary">上传限制（离开本服）</Text>
          <Checkbox checked={config.preventUploadItems} onChange={(e) => set('preventUploadItems', e.target.checked)}>禁止物品</Checkbox>
          <Checkbox checked={config.preventUploadDinos} onChange={(e) => set('preventUploadDinos', e.target.checked)}>禁止恐龙</Checkbox>
          <Checkbox checked={config.preventUploadSurvivors} onChange={(e) => set('preventUploadSurvivors', e.target.checked)}>禁止角色</Checkbox>
        </div>
        <Field label="禁止所有贡品下载"><Switch checked={config.noTributeDownloads} onChange={(v) => set('noTributeDownloads', v)} /></Field>
        <Field label="生物重新上传冷却"><NumberField value={config.minimumDinoReuploadInterval} min={0} max={31536000} step={60} onChange={(v) => set('minimumDinoReuploadInterval', v)} addonAfter="秒" /></Field>
        <Field label="角色上传过期"><NumberField value={config.tributeCharacterExpirationSeconds} min={0} max={31536000} step={3600} onChange={(v) => set('tributeCharacterExpirationSeconds', v)} addonAfter="秒" /></Field>
        <Field label="生物上传过期"><NumberField value={config.tributeDinoExpirationSeconds} min={0} max={31536000} step={3600} onChange={(v) => set('tributeDinoExpirationSeconds', v)} addonAfter="秒" /></Field>
        <Field label="物品上传过期"><NumberField value={config.tributeItemExpirationSeconds} min={0} max={31536000} step={3600} onChange={(v) => set('tributeItemExpirationSeconds', v)} addonAfter="秒" /></Field>
        <Field label="贡品生物数量上限" tip="MaxTributeDinos；低于官方默认值 20 会被服务端回退，高于社区验证上限 273 可能损坏集群数据"><NumberField value={config.maxTributeDinos} min={20} max={273} onChange={(v) => set('maxTributeDinos', v)} /></Field>
        <Field label="贡品物品数量上限" tip="MaxTributeItems；低于官方默认值 50 会被服务端回退，高于社区验证上限 154 可能损坏集群数据"><NumberField value={config.maxTributeItems} min={50} max={154} onChange={(v) => set('maxTributeItems', v)} /></Field>
        <Field label="集群共享目录" wide><Input value={config.clusterDirOverride} onChange={(e) => set('clusterDirOverride', e.target.value)} /></Field>
        <Field label="禁用非集群传输路径"><Switch checked={config.noTransferFromFiltering} onChange={(v) => set('noTransferFromFiltering', v)} /></Field>
        <Alert type="warning" showIcon title="跨服集群应统一 Cluster ID，并显式设置上传/下载规则。" />
      </SectionCard>

      {(isAberration || isLostColony) && (
        <SectionCard title={`${instance.map} 专属设置`} icon={<SettingOutlined />} note="仅当前地图生效">
          {isAberration && (
            <Field label="允许下载非原生生物" tip="CrossARKAllowForeignDinoDownloads；仅 Aberration 的原生生物限制使用">
              <Switch checked={config.crossArkAllowForeignDinoDownloads} onChange={(v) => set('crossArkAllowForeignDinoDownloads', v)} />
            </Field>
          )}
          {isLostColony && <>
            <Field label="限制每部落地堡"><Switch checked={config.limitBunkersPerTribe} onChange={(v) => set('limitBunkersPerTribe', v)} /></Field>
            <Field label="每部落地堡上限"><NumberField value={config.limitBunkersPerTribeNum} min={0} max={100} onChange={(v) => set('limitBunkersPerTribeNum', v)} /></Field>
            <Field label="地堡允许建于限制区"><Switch checked={config.allowBunkersInPreventionZones} onChange={(v) => set('allowBunkersInPreventionZones', v)} /></Field>
            <Field label="允许在地堡内骑乘"><Switch checked={config.allowRidingDinosInsideBunkers} onChange={(v) => set('allowRidingDinosInsideBunkers', v)} /></Field>
            <Field label="地堡模块允许露出地面"><Switch checked={config.allowBunkerModulesAboveGround} onChange={(v) => set('allowBunkerModulesAboveGround', v)} /></Field>
            <Field label="允许地堡内生物 AI"><Switch checked={config.allowDinoAIInsideBunkers} onChange={(v) => set('allowDinoAIInsideBunkers', v)} /></Field>
            <Field label="模块允许建于限制区"><Switch checked={config.allowBunkerModulesInPreventionZones} onChange={(v) => set('allowBunkerModulesInPreventionZones', v)} /></Field>
            <Field label="地堡最小间距"><NumberField value={config.minDistanceBetweenBunkers} min={0} max={100000} step={100} onChange={(v) => set('minDistanceBetweenBunkers', v)} /></Field>
            <Field label="敌方进入血量阈值"><NumberField value={config.enemyAccessBunkerHPThreshold} min={0} max={1} step={0.05} onChange={(v) => set('enemyAccessBunkerHPThreshold', v)} /></Field>
            <Field label="低血量伤害倍率"><NumberField value={config.bunkerUnderHPThresholdDmgMultiplier} min={0} max={10} step={0.05} onChange={(v) => set('bunkerUnderHPThresholdDmgMultiplier', v)} addonAfter="x" /></Field>
          </>}
        </SectionCard>
      )}
    </AccordionGroup>
  )

  const launchArgs = useMemo(() => [
    `-port=${config.gamePort}`,
    `-WinLiveMaxPlayers=${config.maxPlayers}`,
    config.useAllCores && '-USEALLAVAILABLECORES',
    config.noBattlEye && '-NoBattlEye',
    config.noDinos && '-NoDinos',
    config.noWildBabies && '-NoWildBabies',
    config.disableCustomCosmetics && '-DisableCustomCosmetics',
    config.unstasisDinoObstructionCheck && '-UnstasisDinoObstructionCheck',
    config.useServerNetSpeedCheck && '-UseServerNetSpeedCheck',
    config.noSound && '-nosound',
    config.allowFlyerSpeedLeveling && '-AllowFlyerSpeedLeveling',
    config.forceAllowCaveFlyers && '-ForceAllowCaveFlyers',
    config.enableIdlePlayerKick && '-EnableIdlePlayerKick',
    config.preventHibernation && '-preventhibernation',
    config.stasisKeepControllers && '-StasisKeepControllers',
    config.useStructureStasisGrid && '-UseStructureStasisGrid',
    config.alwaysTickDedicatedSkeletalMeshes && '-AlwaysTickDedicatedSkeletalMeshes',
    config.noTransferFromFiltering && '-NoTransferFromFiltering',
    config.gbUsageToForceRestart > 0 && `-GBUsageToForceRestart=${config.gbUsageToForceRestart}`,
    config.serverPlatform && `-ServerPlatform=${config.serverPlatform}`,
    config.activeEvent && `-ActiveEvent=${config.activeEvent}`,
    config.clusterId && `-clusterid=${config.clusterId}`,
    config.clusterDirOverride && `-ClusterDirOverride="${config.clusterDirOverride}"`,
    config.useDynamicConfig && '-UseDynamicConfig',
    config.useDynamicConfig && config.customDynamicConfigUrl && `-CustomDynamicConfigUrl="${config.customDynamicConfigUrl}"`,
    config.serverGameLog && '-servergamelog',
    config.serverGameLogIncludeTribe && '-servergamelogincludetribelogs',
    config.serverGameLogIncludeTribe && '-ServerRCONOutputTribeLogs',
    config.destroyWildDinos && '-ForceRespawnDinos',
    (config.whitelist || config.exclusiveJoin) && '-exclusivejoin',
    config.customLaunchArgs,
  ].filter(Boolean).join(' '), [config])

  const performanceTab = (
    <AccordionGroup forceExpand={configSearchActive}>
      <SectionCard title="进程资源调度" icon={<ThunderboltOutlined />} note="管理器级设置">
        <Field label="进程优先级"><Select value={config.processPriority} onChange={(v) => set('processPriority', v)} options={[{ label: '正常', value: 'normal' }, { label: '高于正常', value: 'aboveNormal' }, { label: '高', value: 'high' }]} /></Field>
        <Field label="CPU 核心亲和性" tip="自动或填写核心编号，例如 0-7"><Input value={config.cpuAffinity} onChange={(e) => set('cpuAffinity', e.target.value)} /></Field>
        <Field label="内存告警阈值"><NumberField value={config.memoryWarningGb} min={4} max={256} onChange={(v) => set('memoryWarningGb', v)} addonAfter="GB" /></Field>
        <Field label="使用全部可用核心"><Switch checked={config.useAllCores} onChange={(v) => set('useAllCores', v)} /></Field>
        <Field label="禁用 BattlEye"><Switch checked={config.noBattlEye} onChange={(v) => set('noBattlEye', v)} /></Field>
        <Alert type="info" showIcon title="资源阈值由管理器监控，不会写入 ARK 的 INI 文件。" />
      </SectionCard>

      <SectionCard title="网络与 RCON 调优" icon={<CloudSyncOutlined />} note="谨慎修改">
        <Field label="网络 Tick Rate" tip="NetServerMaxTickRate；过高会增加 CPU 与带宽占用"><NumberField value={config.networkTickRate} min={10} max={120} step={5} onChange={(v) => set('networkTickRate', v)} addonAfter="Hz" /></Field>
        <Field label="客户端速率上限" tip="MaxClientRate"><NumberField value={config.maxClientRate} min={15000} max={1000000} step={5000} onChange={(v) => set('maxClientRate', v)} addonAfter="B/s" /></Field>
        <Field label="RCON 输出缓冲"><NumberField value={config.rconBufferSize} min={1000} max={64000} step={1000} onChange={(v) => set('rconBufferSize', v)} addonAfter="行" /></Field>
        <div className="resource-meter">
          <div><span>CPU 预算</span><span>62%</span></div><Progress percent={62} showInfo={false} strokeColor="#00b8ff" railColor="#10273a" />
          <div><span>内存预算</span><span>13.6 / {config.memoryWarningGb} GB</span></div><Progress percent={Math.round(13.6 / config.memoryWarningGb * 100)} showInfo={false} strokeColor="#18cf7a" railColor="#10273a" />
        </div>
      </SectionCard>

      <SectionCard title="存档与世界内存" icon={<HistoryOutlined />} note="稳定性优先">
        <Field label="保存间隔"><NumberField value={config.saveInterval} min={5} max={120} onChange={(v) => set('saveInterval', v)} addonAfter="分钟" /></Field>
        <Field label="压缩历史备份"><Switch checked={config.compressBackups} onChange={(v) => set('compressBackups', v)} /></Field>
        <Field label="重启前生成快照"><Switch checked={config.snapshotBeforeRestart} onChange={(v) => set('snapshotBeforeRestart', v)} /></Field>
        <Field label="禁用冬眠系统" tip="-preventhibernation 会显著增加内存占用"><Switch checked={config.preventHibernation} onChange={(v) => set('preventHibernation', v)} /></Field>
        <Field label="冻结区保留控制器" tip="-StasisKeepControllers，可能增加 CPU 与内存占用"><Switch checked={config.stasisKeepControllers} onChange={(v) => set('stasisKeepControllers', v)} /></Field>
      </SectionCard>

      <SectionCard title="平台、事件与动态配置" icon={<SettingOutlined />} note="ASA 启动参数">
        <Field label="连接平台"><Select value={config.serverPlatform} onChange={(v) => set('serverPlatform', v)} options={[{ label: '全部平台', value: 'ALL' }, { label: 'Steam / PC', value: 'PC' }, { label: 'PlayStation 5', value: 'PS5' }, { label: 'Xbox Series', value: 'XSX' }, { label: 'Windows Store', value: 'WINGDK' }]} /></Field>
        <Field label="激活活动" tip="对应 ASA 启动参数 -ActiveEvent=<eventname>；留空不追加参数，None 会显式禁用默认活动引用。ARK Wiki 标注目前仅 WinterWonderland 仍完整可用，已废弃活动不可新选。">
          <Select
            value={config.activeEvent}
            onChange={(value) => set('activeEvent', value)}
            options={activeEventOptions}
            showSearch
            optionFilterProp="label"
          />
        </Field>
        <Field label="内存强制重启阈值"><NumberField value={config.gbUsageToForceRestart} min={0} max={512} onChange={(v) => set('gbUsageToForceRestart', v)} addonAfter="GB" /></Field>
        <Field label="启用结构停滞网格"><Switch checked={config.useStructureStasisGrid} onChange={(v) => set('useStructureStasisGrid', v)} /></Field>
        <Field label="持续更新骨骼网格"><Switch checked={config.alwaysTickDedicatedSkeletalMeshes} onChange={(v) => set('alwaysTickDedicatedSkeletalMeshes', v)} /></Field>
        <Field label="启用动态配置"><Switch checked={config.useDynamicConfig} onChange={(v) => set('useDynamicConfig', v)} /></Field>
        {config.useDynamicConfig && <Field label="动态配置 URL" wide><Input value={config.customDynamicConfigUrl} onChange={(e) => set('customDynamicConfigUrl', e.target.value)} placeholder="https://example.com/dynamicconfig.ini" /></Field>}
      </SectionCard>

      <SectionCard title="服务端启动行为" icon={<ThunderboltOutlined />} note="ASA 启动参数">
        <Field label="禁用全部生物" tip="-NoDinos；危险：启用后世界不会生成任何恐龙"><Switch checked={config.noDinos} onChange={(v) => set('noDinos', v)} /></Field>
        <Field label="禁用野生幼崽" tip="-NoWildBabies"><Switch checked={config.noWildBabies} onChange={(v) => set('noWildBabies', v)} /></Field>
        <Field label="禁用自定义装饰" tip="-DisableCustomCosmetics"><Switch checked={config.disableCustomCosmetics} onChange={(v) => set('disableCustomCosmetics', v)} /></Field>
        <Field label="解除停滞时检查生物阻挡" tip="-UnstasisDinoObstructionCheck"><Switch checked={config.unstasisDinoObstructionCheck} onChange={(v) => set('unstasisDinoObstructionCheck', v)} /></Field>
        <Field label="启用服务端网速检查" tip="-UseServerNetSpeedCheck"><Switch checked={config.useServerNetSpeedCheck} onChange={(v) => set('useServerNetSpeedCheck', v)} /></Field>
        <Field label="禁用服务端声音" tip="-nosound"><Switch checked={config.noSound} onChange={(v) => set('noSound', v)} /></Field>
      </SectionCard>

      <SectionCard title="启动参数预览" icon={<CodeOutlined />} note="实时生成">
        <Field label="额外启动参数" wide><Input.TextArea rows={3} value={config.customLaunchArgs} onChange={(e) => set('customLaunchArgs', e.target.value)} /></Field>
        <div className="code-preview">
          <span>ShooterGameServer.exe {instance.mapCode}</span>
          <span>?SessionName={encodeURIComponent(config.sessionName)}?QueryPort={config.queryPort}?RCONPort={config.rconPort}</span>
          <span>{launchArgs}</span>
        </div>
        <Alert type="warning" showIcon title="性能参数没有通用最优值。禁用冬眠等选项可能显著提高资源占用。" />
      </SectionCard>

      <SectionCard title="高级自定义 INI" icon={<CodeOutlined />} note="保留未结构化配置">
        <Field label="ServerSettings 自定义项" tip="写入 GameUserSettings.ini 的 [ServerSettings]；每行只填 Key=Value，不要填写 section。与表单托管键冲突时以表单设置为准。" wide>
          <Input.TextArea rows={4} value={config.customServerSettings} onChange={(e) => set('customServerSettings', e.target.value)} placeholder={'Key=Value\nAnotherKey=Value'} />
        </Field>
        <Field label="ShooterGameMode 自定义项" tip="写入 Game.ini 的 [/Script/ShooterGame.ShooterGameMode]；每行只填 Key=Value，不要填写 section。与表单托管键冲突时以表单设置为准。" wide>
          <Input.TextArea rows={4} value={config.customGameIniSettings} onChange={(e) => set('customGameIniSettings', e.target.value)} placeholder={'Key=Value\nAnotherKey=Value'} />
        </Field>
        <Field label="IpNetDriver 自定义项" tip="写入 Engine.ini 的 [/Script/OnlineSubsystemUtils.IpNetDriver]；每行只填 Key=Value，不要填写 section。与表单托管键冲突时以表单设置为准。" wide>
          <Input.TextArea rows={4} value={config.customEngineIniSettings} onChange={(e) => set('customEngineIniSettings', e.target.value)} placeholder={'Key=Value\nAnotherKey=Value'} />
        </Field>
      </SectionCard>
    </AccordionGroup>
  )

  const modsTab = (
    <AccordionGroup className="mod-layout" forceExpand={configSearchActive}>
      <SectionCard title="MOD 加载列表" icon={<BugOutlined />} note={`已启用 ${mods.filter((m) => m.enabled).length} / ${mods.length}`}>
        <div className="mod-toolbar">
          <div className="mod-toolbar__actions">
            <Button type="primary" icon={<PlusOutlined />} onClick={openModModal}>添加 MOD</Button>
            <Button loading={checkingMods} icon={<ReloadOutlined />} onClick={onCheckModUpdates}>检查更新</Button>
          </div>
          <Text type="secondary">按列表顺序加载，靠上的 MOD 优先加载</Text>
        </div>
        {mods.length === 0 ? <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="尚未添加 MOD" /> : (
          <div className="mod-list">
            {mods.map((item, index) => (
              <div className="mod-item" key={item.id}>
                <div className="mod-item__main">
                  <span className="mod-order">#{String(index + 1).padStart(2, '0')}</span>
                  <div className="mod-name"><strong>{item.name}</strong><span>CurseForge ID · {item.id}</span></div>
                  <Switch size="small" checked={item.enabled} onChange={(enabled) => onModsChange(mods.map((mod) => mod.id === item.id ? { ...mod, enabled } : mod))} />
                </div>
                <div className="mod-item__meta">
                  <span>版本 {item.version}</span>
                  {item.updateAvailable && <Tag color="orange">可更新</Tag>}
                  <span>{item.size}</span>
                  <div className="mod-item__actions">
                    <Tooltip title="上移"><Button type="text" size="small" icon={<ArrowUpOutlined />} disabled={index === 0} onClick={() => moveMod(index, -1)} /></Tooltip>
                    <Tooltip title="下移"><Button type="text" size="small" icon={<ArrowDownOutlined />} disabled={index === mods.length - 1} onClick={() => moveMod(index, 1)} /></Tooltip>
                    <Tooltip title="移除"><Button type="text" danger size="small" icon={<DeleteOutlined />} onClick={() => onModsChange(mods.filter((mod) => mod.id !== item.id))} /></Tooltip>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </SectionCard>
      <SectionCard title="自动更新策略" icon={<CloudSyncOutlined />}>
        <Field label="启动前检查更新"><Switch checked={config.autoUpdateMods} onChange={(v) => set('autoUpdateMods', v)} /></Field>
        <Field label="有更新时自动下载"><Switch checked={config.autoUpdateMods} onChange={(v) => set('autoUpdateMods', v)} /></Field>
        <Field label="运行中仅通知"><Switch defaultChecked /></Field>
      </SectionCard>
      <SectionCard title="生成的 ActiveMods" icon={<CodeOutlined />} note="GameUserSettings.ini">
        <div className="code-preview code-preview--single">ActiveMods={mods.filter((mod) => mod.enabled).map((mod) => mod.id).join(',') || '(未启用 MOD)'}</div>
        <Paragraph type="secondary">ASA 使用 CurseForge MOD ID。管理器保存时会按当前顺序写入 <code>[ServerSettings]</code>。</Paragraph>
      </SectionCard>
    </AccordionGroup>
  )

  const logsTab = (
    <AccordionGroup className="log-settings-layout" forceExpand={configSearchActive}>
      <SectionCard title="服务端日志参数" icon={<FileSearchOutlined />} note="启动参数">
        <Field label="启用 Server Game Log"><Switch checked={config.serverGameLog} onChange={(v) => set('serverGameLog', v)} /></Field>
        <Field label="输出部落日志到 RCON"><Switch checked={config.serverGameLogIncludeTribe} onChange={(v) => set('serverGameLogIncludeTribe', v)} /></Field>
        <Field label="管理员命令审计"><Switch checked={config.adminLogging} onChange={(v) => set('adminLogging', v)} /></Field>
        <Field label="聊天记录"><Switch checked={config.chatLogging} onChange={(v) => set('chatLogging', v)} /></Field>
      </SectionCard>
      <SectionCard title="轮转与保留" icon={<HistoryOutlined />} note="管理器级设置">
        <Field label="日志详细级别"><Select value={config.logLevel} onChange={(v) => set('logLevel', v)} options={[{ label: '普通', value: 'normal' }, { label: '详细', value: 'verbose' }, { label: '调试', value: 'debug' }]} /></Field>
        <Field label="单文件轮转大小"><NumberField value={config.rotateSizeMb} min={10} max={2048} step={10} onChange={(v) => set('rotateSizeMb', v)} addonAfter="MB" /></Field>
        <Field label="日志保留天数"><NumberField value={config.logRetentionDays} min={1} max={365} onChange={(v) => set('logRetentionDays', v)} addonAfter="天" /></Field>
        <Field label="日志目录" wide><Input value={config.logPath} onChange={(e) => set('logPath', e.target.value)} suffix={<FolderOpenOutlined />} /></Field>
      </SectionCard>
    </AccordionGroup>
  )

  const tabs = [
    { key: 'basic', label: '基础设置', children: basicTab },
    { key: 'advanced', label: '高级设置', children: advancedTab },
    { key: 'performance', label: '性能设置', children: performanceTab },
    { key: 'mods', label: 'MOD 设置', children: modsTab },
    { key: 'logs', label: '日志参数', children: logsTab },
  ]
  const filteredTabs = configSearchActive
    ? tabs.map((tab) => {
      const tabMatches = normalizeSearchText(tab.label).includes(normalizedConfigSearch)
      const children = tabMatches ? tab.children : filterSearchNode(tab.children, normalizedConfigSearch)
      return hasRenderableNode(children) ? { ...tab, children } : null
    }).filter((tab): tab is typeof tabs[number] => tab !== null)
    : tabs
  const tabsWithSearchState = filteredTabs.length > 0 ? filteredTabs : [{
    key: 'config-search-empty',
    label: '无结果',
    disabled: true,
    children: (
      <div className="config-panel__search-empty">
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="没有匹配的配置项" />
      </div>
    ),
  }]
  const activeTabKey = tabsWithSearchState.some((tab) => tab.key === activeTab) ? activeTab : tabsWithSearchState[0].key

  return (
    <div className="config-panel">
      <div className="config-panel__title">
        <div className="config-title-text"><span className="ark-mark">✣</span><span>实例配置编辑</span><span className="config-title-separator">/</span><strong>{instance.name}</strong><span>·</span><span>{instance.map}</span></div>
        <Space>
          {dirty && <Tag color="gold">有未保存修改</Tag>}
          <Tag color={instance.status === 'running' ? 'success' : instance.status === 'error' ? 'error' : ['starting', 'stopping', 'updating', 'backingUp'].includes(instance.status) ? 'processing' : 'default'}>
            {instance.status === 'running' ? '● 运行中' : instance.status === 'stopping' ? '● 停止中' : instance.status === 'starting' ? '● 启动中' : instance.status === 'updating' ? '● 更新中' : instance.status === 'backingUp' ? '● 备份中' : instance.status === 'error' ? '● 异常' : '● 已停止'}
          </Tag>
        </Space>
      </div>
      <div className="config-panel__search">
        <Input
          allowClear
          aria-label="搜索配置项"
          placeholder="搜索配置项、分组或 INI 参数"
          prefix={<SearchOutlined />}
          value={configSearch}
          onChange={(event) => setConfigSearch(event.target.value)}
        />
        {configSearchActive && (
          <div className="config-panel__search-meta">
            <span>{filteredTabs.length > 0 ? `已筛选 ${filteredTabs.length} 个页签` : '未找到匹配配置'}</span>
            <span>搜索时自动展开匹配分组</span>
          </div>
        )}
      </div>
      <Tabs activeKey={activeTabKey} onChange={setActiveTab} items={tabsWithSearchState} />
      <div className="panel-footer-actions">
        <div className="panel-footer-actions__buttons">
          <Button loading={configOperation === 'save'} disabled={actionsDisabled} onClick={onSave}>仅保存配置</Button>
          <Button type="primary" icon={<ReloadOutlined />} loading={configOperation === 'apply'} disabled={actionsDisabled} onClick={onApply}>保存并应用重启</Button>
        </div>
      </div>

      <ItemStackOverrideModal
        language={language}
        open={itemStackModalOpen}
        overrides={itemStackOverrides}
        onChange={(next) => set('itemStackOverrides', next)}
        onClose={() => setItemStackModalOpen(false)}
      />
      <Modal
        className="curseforge-mod-dialog"
        centered
        destroyOnHidden
        open={modModalOpen}
        title={(
          <div className="curseforge-mod-dialog__title">
            <BugOutlined />
            <span>添加 CurseForge MOD</span>
            <Tag color="blue">ARK: Survival Ascended</Tag>
          </div>
        )}
        width={860}
        onCancel={closeModModal}
        footer={[
          <Button key="cancel" onClick={closeModModal}>取消</Button>,
          <Button
            key="add"
            type="primary"
            icon={<PlusOutlined />}
            disabled={selectedCatalogMods.length === 0}
            onClick={addSelectedMods}
          >
            添加 {selectedCatalogMods.length > 0 ? `${selectedCatalogMods.length} 个 MOD` : '所选 MOD'}
          </Button>,
        ]}
      >
        <div className="curseforge-mod-dialog__body">
          <Input
            allowClear
            autoFocus
            aria-label="搜索 CurseForge MOD"
            size="large"
            prefix={<SearchOutlined />}
            placeholder="搜索 MOD 名称或关键词"
            value={modSearch}
            onChange={(event) => {
              setModSearch(event.target.value)
              setModSearchPage(1)
            }}
          />

          <div className="curseforge-mod-dialog__toolbar">
            <Checkbox
              checked={allCurrentPageSelected}
              disabled={selectableCatalogMods.length === 0}
              indeterminate={selectedOnCurrentPage > 0 && !allCurrentPageSelected}
              onChange={toggleCurrentCatalogPage}
            >
              选择当前页
            </Checkbox>
            <Text type="secondary">
              已选 {selectedCatalogMods.length} 个 · 共 {modCatalog.totalCount.toLocaleString('zh-CN')} 个结果
            </Text>
          </div>

          <div className="curseforge-mod-dialog__results" aria-busy={modCatalogLoading}>
            {modCatalogError ? (
              <Alert
                showIcon
                type="error"
                message="无法加载 CurseForge 官方 MOD"
                description={modCatalogError}
                action={<Button size="small" onClick={() => setModCatalogReload((value) => value + 1)}>重试</Button>}
              />
            ) : modCatalogLoading && modCatalog.items.length === 0 ? (
              <div className="curseforge-mod-dialog__loading"><Spin /><Text type="secondary">正在获取官方 MOD 数据</Text></div>
            ) : modCatalog.items.length === 0 ? (
              <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={modSearch.trim() ? '没有找到匹配的 MOD' : '官方目录暂无可用 MOD'} />
            ) : (
              <div className="curseforge-catalog-list" role="list">
                {modCatalog.items.map((item) => {
                  const alreadyAdded = existingModIds.has(item.id)
                  const selected = selectedCatalogModIds.has(item.id)
                  return (
                    <div
                      className={`curseforge-catalog-item${selected ? ' curseforge-catalog-item--selected' : ''}${alreadyAdded ? ' curseforge-catalog-item--disabled' : ''}`}
                      key={item.id}
                      role="listitem"
                    >
                      <Checkbox
                        aria-label={`选择 ${item.name}`}
                        checked={alreadyAdded || selected}
                        disabled={alreadyAdded}
                        onChange={() => toggleCatalogMod(item)}
                      />
                      <Avatar
                        className="curseforge-catalog-item__image"
                        shape="square"
                        size={52}
                        src={item.thumbnailUrl ?? undefined}
                        icon={<BugOutlined />}
                      />
                      <div className="curseforge-catalog-item__content">
                        <div className="curseforge-catalog-item__heading">
                          <strong title={item.name}>{item.name}</strong>
                          {alreadyAdded && <Tag>已添加</Tag>}
                        </div>
                        <p title={item.summary}>{item.summary || '暂无简介'}</p>
                        <div className="curseforge-catalog-item__meta">
                          <span>{item.author}</span>
                          <span>ID {item.id}</span>
                          <span>{formatDownloadCount(item.downloadCount)} 次下载</span>
                          <span>更新于 {formatModDate(item.dateModified)}</span>
                        </div>
                      </div>
                      <div className="curseforge-catalog-item__release">
                        <strong title={item.version}>{item.version}</strong>
                        <span>{item.size}</span>
                        <Tooltip title="在 CurseForge 查看">
                          <Button
                            aria-label={`在 CurseForge 查看 ${item.name}`}
                            type="text"
                            size="small"
                            icon={<LinkOutlined />}
                            href={item.websiteUrl}
                            target="_blank"
                          />
                        </Tooltip>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
            {modCatalogLoading && modCatalog.items.length > 0 && (
              <div className="curseforge-mod-dialog__loading-overlay"><Spin size="small" /></div>
            )}
          </div>

          {modCatalog.totalCount > CURSEFORGE_PAGE_SIZE && !modCatalogError && (
            <Pagination
              align="center"
              current={modSearchPage}
              pageSize={CURSEFORGE_PAGE_SIZE}
              showSizeChanger={false}
              total={Math.min(modCatalog.totalCount, 10_000)}
              onChange={setModSearchPage}
            />
          )}
        </div>
      </Modal>
    </div>
  )
}
