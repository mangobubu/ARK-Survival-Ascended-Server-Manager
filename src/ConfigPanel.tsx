import { createContext, useContext, useMemo, useState } from 'react'
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
  InfoCircleOutlined,
  PlusOutlined,
  ReloadOutlined,
  RightOutlined,
  SafetyCertificateOutlined,
  SettingOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons'
import {
  Alert,
  Button,
  Checkbox,
  Empty,
  Input,
  InputNumber,
  Modal,
  Progress,
  Radio,
  Segmented,
  Select,
  Space,
  Switch,
  Tabs,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import type { ModItem, ServerConfig, ServerInstance } from './types'

const { Text, Paragraph } = Typography

interface ConfigPanelProps {
  instance: ServerInstance
  config: ServerConfig
  mods: ModItem[]
  dirty: boolean
  onConfigChange: <K extends keyof ServerConfig>(key: K, value: ServerConfig[K]) => void
  onModsChange: (mods: ModItem[]) => void
  onSave: () => void
  onApply: () => void
  onCheckModUpdates: () => void
  checkingMods?: boolean
}

const AccordionContext = createContext<{
  activeSection: string | null
  setActiveSection: (section: string | null) => void
} | null>(null)

function AccordionGroup({ children, className = '' }: { children: React.ReactNode; className?: string }) {
  const [activeSection, setActiveSection] = useState<string | null>(null)

  return (
    <AccordionContext.Provider value={{ activeSection, setActiveSection }}>
      <div className={`settings-accordion ${className}`}>{children}</div>
    </AccordionContext.Provider>
  )
}

function SectionCard({ title, icon, note, children, className = '' }: { title: string; icon?: React.ReactNode; note?: string; children: React.ReactNode; className?: string }) {
  const accordion = useContext(AccordionContext)
  const expanded = !accordion || accordion.activeSection === title

  return (
    <section className={`setting-card ${expanded ? 'setting-card--expanded' : ''} ${className}`}>
      <button
        type="button"
        className="setting-card__header"
        aria-expanded={expanded}
        onClick={() => accordion?.setActiveSection(expanded ? null : title)}
      >
        <span className="setting-card__icon">{icon}</span>
        <span>{title}</span>
        {note && <span className="setting-card__note">{note}</span>}
        {accordion && <RightOutlined className="setting-card__chevron" />}
      </button>
      {expanded && <div className="setting-card__body">{children}</div>}
    </section>
  )
}

function Field({ label, tip, children, wide = false }: { label: string; tip?: string; children: React.ReactNode; wide?: boolean }) {
  return (
    <div className={`config-field ${wide ? 'config-field--wide' : ''}`}>
      <div className="config-field__label">
        <span>{label}</span>
        {tip && <Tooltip title={tip}><InfoCircleOutlined /></Tooltip>}
      </div>
      <div className="config-field__control">{children}</div>
    </div>
  )
}

function NumberField({ value, onChange, min = 0, max, step = 1, addonAfter }: { value: number; onChange: (value: number) => void; min?: number; max?: number; step?: number; addonAfter?: string }) {
  if (addonAfter) {
    return (
      <Space.Compact>
        <InputNumber value={value} min={min} max={max} step={step} onChange={(next) => onChange(next ?? min)} />
        <Input value={addonAfter} disabled style={{ width: '48px', textAlign: 'center', padding: '4px 0' }} />
      </Space.Compact>
    )
  }
  return <InputNumber value={value} min={min} max={max} step={step} onChange={(next) => onChange(next ?? min)} />
}

export default function ConfigPanel({ instance, config, mods, dirty, onConfigChange, onModsChange, onSave, onApply, onCheckModUpdates, checkingMods = false }: ConfigPanelProps) {
  const [modModalOpen, setModModalOpen] = useState(false)
  const [modId, setModId] = useState('')
  const [activeTab, setActiveTab] = useState('basic')
  const set = onConfigChange
  const isAberration = instance.mapCode === 'Aberration_WP'
  const isLostColony = instance.mapCode === 'LostColony_WP'

  const addMod = () => {
    const id = modId.trim()
    if (!id || mods.some((mod) => mod.id === id)) return
    onModsChange([...mods, { id, name: `CurseForge 模组 ${id}`, version: '等待检测', size: '—', enabled: true }])
    setModId('')
    setModModalOpen(false)
  }

  const moveMod = (index: number, direction: -1 | 1) => {
    const target = index + direction
    if (target < 0 || target >= mods.length) return
    const next = [...mods]
    ;[next[index], next[target]] = [next[target], next[index]]
    onModsChange(next)
  }

  const basicTab = (
    <AccordionGroup>
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
        <Field label="服务器可见性"><Radio.Group value={config.visibility} onChange={(e) => set('visibility', e.target.value)} optionType="button" buttonStyle="solid" options={[{ label: '公开', value: 'public' }, { label: '私有', value: 'private' }]} /></Field>
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
        <Field label="采集数量倍率"><NumberField value={config.harvestAmount} min={0.1} max={100} step={0.5} onChange={(v) => set('harvestAmount', v)} addonAfter="x" /></Field>
        <Field label="采集物耐久倍率" tip="HarvestHealthMultiplier；越高可采集的次数越多"><NumberField value={config.harvestHealthMultiplier} min={0.1} max={100} step={0.1} onChange={(v) => set('harvestHealthMultiplier', v)} addonAfter="x" /></Field>
        <Field label="硬核模式"><Switch checked={config.hardcore} onChange={(v) => set('hardcore', v)} /></Field>
        <Field label="禁用友军伤害"><Switch checked={config.disableFriendlyFire} onChange={(v) => set('disableFriendlyFire', v)} /></Field>
        <Field label="允许 PvP Gamma"><Switch checked={config.enablePvPGamma} onChange={(v) => set('enablePvPGamma', v)} /></Field>
        <Field label="显示命中标记"><Switch checked={config.allowHitMarkers} onChange={(v) => set('allowHitMarkers', v)} /></Field>
        <Field label="允许第三人称"><Switch checked={config.thirdPerson} onChange={(v) => set('thirdPerson', v)} /></Field>
        <Field label="显示准星"><Switch checked={config.crosshair} onChange={(v) => set('crosshair', v)} /></Field>
        <Field label="地图显示玩家位置"><Switch checked={config.showMapPlayer} onChange={(v) => set('showMapPlayer', v)} /></Field>
        <Field label="PvE 飞行生物叼取"><Switch checked={config.flyerCarry} onChange={(v) => set('flyerCarry', v)} /></Field>
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
      </SectionCard>

      <SectionCard title="资源、物品与战利品" icon={<CloudSyncOutlined />} note="常用倍率">
        <Field label="资源玩家刷新半径"><NumberField value={config.resourceNoReplenishRadiusPlayers} min={0.01} max={100} step={0.1} onChange={(v) => set('resourceNoReplenishRadiusPlayers', v)} addonAfter="x" /></Field>
        <Field label="资源建筑刷新半径"><NumberField value={config.resourceNoReplenishRadiusStructures} min={0.01} max={100} step={0.1} onChange={(v) => set('resourceNoReplenishRadiusStructures', v)} addonAfter="x" /></Field>
        <Field label="作物生长速度"><NumberField value={config.cropGrowthSpeedMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('cropGrowthSpeedMultiplier', v)} addonAfter="x" /></Field>
        <Field label="作物腐坏速度"><NumberField value={config.cropDecaySpeedMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('cropDecaySpeedMultiplier', v)} addonAfter="x" /></Field>
        <Field label="补给箱品质"><NumberField value={config.supplyCrateLootQualityMultiplier} min={0.1} max={5} step={0.1} onChange={(v) => set('supplyCrateLootQualityMultiplier', v)} addonAfter="x" /></Field>
        <Field label="钓鱼战利品品质"><NumberField value={config.fishingLootQualityMultiplier} min={0.1} max={5} step={0.1} onChange={(v) => set('fishingLootQualityMultiplier', v)} addonAfter="x" /></Field>
        <Field label="燃料消耗间隔" tip="越高燃料使用越慢"><NumberField value={config.fuelConsumptionIntervalMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('fuelConsumptionIntervalMultiplier', v)} addonAfter="x" /></Field>
        <Field label="物品堆叠倍率"><NumberField value={config.itemStackSizeMultiplier} min={0.1} max={1000} step={0.5} onChange={(v) => set('itemStackSizeMultiplier', v)} addonAfter="x" /></Field>
        <Field label="食物腐坏时间"><NumberField value={config.globalSpoilingTimeMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('globalSpoilingTimeMultiplier', v)} addonAfter="x" /></Field>
        <Field label="掉落物分解时间"><NumberField value={config.globalItemDecompositionTimeMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('globalItemDecompositionTimeMultiplier', v)} addonAfter="x" /></Field>
        <Field label="尸体分解时间"><NumberField value={config.globalCorpseDecompositionTimeMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('globalCorpseDecompositionTimeMultiplier', v)} addonAfter="x" /></Field>
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
    <AccordionGroup>
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
        <Field label="单次留痕量"><NumberField value={config.babyImprintAmountMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('babyImprintAmountMultiplier', v)} addonAfter="x" /></Field>
        <Field label="任何人可照料幼崽"><Switch checked={config.allowAnyoneBabyImprintCuddle} onChange={(v) => set('allowAnyoneBabyImprintCuddle', v)} /></Field>
      </SectionCard>

      <SectionCard title="建筑、部落与衰减" icon={<ApartmentOutlined />} note="Game.ini / ServerSettings">
        <div className="field-pair">
          <Field label="范围内建筑上限"><NumberField value={config.structureLimit} min={100} max={100000} step={100} onChange={(v) => set('structureLimit', v)} /></Field>
          <Field label="平台建筑倍率"><NumberField value={config.platformStructureMultiplier} min={0.1} max={10} step={0.1} onChange={(v) => set('platformStructureMultiplier', v)} addonAfter="x" /></Field>
        </div>
        <Field label="禁用建筑碰撞检测"><Switch checked={config.disablePlacementCollision} onChange={(v) => set('disablePlacementCollision', v)} /></Field>
        <Field label="最大部落人数"><NumberField value={config.maxTribeSize} min={0} max={500} onChange={(v) => set('maxTribeSize', v)} /></Field>
        <Field label="允许部落联盟"><Switch checked={config.tribeAlliances} onChange={(v) => set('tribeAlliances', v)} /></Field>
        <Field label="启用 PvE 建筑衰减"><Switch checked={config.pveStructureDecay} onChange={(v) => set('pveStructureDecay', v)} /></Field>
        <Field label="PvE 洞穴建造"><Switch checked={config.allowCaveBuildingPvE} onChange={(v) => set('allowCaveBuildingPvE', v)} /></Field>
        <Field label="PvP 洞穴建造"><Switch checked={config.allowCaveBuildingPvP} onChange={(v) => set('allowCaveBuildingPvP', v)} /></Field>
        <Field label="建筑维修冷却"><NumberField value={config.structureDamageRepairCooldown} min={0} max={86400} onChange={(v) => set('structureDamageRepairCooldown', v)} addonAfter="秒" /></Field>
        <Field label="建筑可拾取时间"><NumberField value={config.structurePickupTimeAfterPlacement} min={0} max={86400} onChange={(v) => set('structurePickupTimeAfterPlacement', v)} addonAfter="秒" /></Field>
        <Field label="拾取长按时间"><NumberField value={config.structurePickupHoldDuration} min={0} max={60} step={0.1} onChange={(v) => set('structurePickupHoldDuration', v)} addonAfter="秒" /></Field>
        <Field label="旧建筑摧毁周期"><NumberField value={config.autoDestroyOldStructuresMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('autoDestroyOldStructuresMultiplier', v)} addonAfter="x" /></Field>
        <Field label="孤立建筑快速衰减"><Switch checked={config.fastDecayUnsnappedCoreStructures} onChange={(v) => set('fastDecayUnsnappedCoreStructures', v)} /></Field>
        <Field label="区域发电机上限"><NumberField value={config.limitGeneratorsNum} min={0} max={1000} onChange={(v) => set('limitGeneratorsNum', v)} /></Field>
        <Field label="发电机限制范围"><NumberField value={config.limitGeneratorsRange} min={0} max={1000000} step={100} onChange={(v) => set('limitGeneratorsRange', v)} /></Field>
        <Field label="平台鞍允许低温冰箱"><Switch checked={config.allowCryoFridgeOnSaddle} onChange={(v) => set('allowCryoFridgeOnSaddle', v)} /></Field>
        <Field label="关闭冷冻舱敌人检测"><Switch checked={config.disableCryopodEnemyCheck} onChange={(v) => set('disableCryopodEnemyCheck', v)} /></Field>
        <Field label="取消低温冰箱要求"><Switch checked={config.disableCryopodFridgeRequirement} onChange={(v) => set('disableCryopodFridgeRequirement', v)} /></Field>
        <Field label="取消冷冻舱冷却"><Switch checked={config.disableCryopodCooldown} onChange={(v) => set('disableCryopodCooldown', v)} /></Field>
      </SectionCard>

      <SectionCard title="生物、飞行与冷冻舱" icon={<BugOutlined />} note="GameUserSettings.ini / 启动参数">
        <Field label="飞行生物速度升级"><Switch checked={config.allowFlyerSpeedLeveling} onChange={(v) => set('allowFlyerSpeedLeveling', v)} /></Field>
        <Field label="强制允许洞穴飞行"><Switch checked={config.forceAllowCaveFlyers} onChange={(v) => set('forceAllowCaveFlyers', v)} /></Field>
        <Field label="骑乘飞行生物恢复耐力"><Switch checked={config.allowFlyingStaminaRecovery} onChange={(v) => set('allowFlyingStaminaRecovery', v)} /></Field>
        <Field label="突袭生物食物消耗"><NumberField value={config.raidDinoFoodDrainMultiplier} min={0.01} max={100} step={0.1} onChange={(v) => set('raidDinoFoodDrainMultiplier', v)} addonAfter="x" /></Field>
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
    config.serverGameLogIncludeTribe && '-ServerRCONOutputTribeLogs',
    config.destroyWildDinos && '-ForceRespawnDinos',
    (config.whitelist || config.exclusiveJoin) && '-exclusivejoin',
    config.customLaunchArgs,
  ].filter(Boolean).join(' '), [config])

  const performanceTab = (
    <AccordionGroup>
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
        <Field label="激活活动"><Input value={config.activeEvent} onChange={(e) => set('activeEvent', e.target.value)} placeholder="留空表示不启用" /></Field>
        <Field label="内存强制重启阈值"><NumberField value={config.gbUsageToForceRestart} min={0} max={512} onChange={(v) => set('gbUsageToForceRestart', v)} addonAfter="GB" /></Field>
        <Field label="启用结构停滞网格"><Switch checked={config.useStructureStasisGrid} onChange={(v) => set('useStructureStasisGrid', v)} /></Field>
        <Field label="持续更新骨骼网格"><Switch checked={config.alwaysTickDedicatedSkeletalMeshes} onChange={(v) => set('alwaysTickDedicatedSkeletalMeshes', v)} /></Field>
        <Field label="启用动态配置"><Switch checked={config.useDynamicConfig} onChange={(v) => set('useDynamicConfig', v)} /></Field>
        {config.useDynamicConfig && <Field label="动态配置 URL" wide><Input value={config.customDynamicConfigUrl} onChange={(e) => set('customDynamicConfigUrl', e.target.value)} placeholder="https://example.com/dynamicconfig.ini" /></Field>}
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
    </AccordionGroup>
  )

  const modsTab = (
    <AccordionGroup className="mod-layout">
      <SectionCard title="MOD 加载列表" icon={<BugOutlined />} note={`已启用 ${mods.filter((m) => m.enabled).length} / ${mods.length}`}>
        <div className="mod-toolbar">
          <div className="mod-toolbar__actions">
            <Button type="primary" icon={<PlusOutlined />} onClick={() => setModModalOpen(true)}>添加 MOD</Button>
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
    <AccordionGroup className="log-settings-layout">
      <SectionCard title="服务端日志参数" icon={<FileSearchOutlined />} note="启动参数">
        <Field label="启用 Server Game Log"><Switch checked={config.serverGameLog} onChange={(v) => set('serverGameLog', v)} /></Field>
        <Field label="输出部落日志到 RCON"><Switch checked={config.serverGameLogIncludeTribe} onChange={(v) => set('serverGameLogIncludeTribe', v)} /></Field>
        <Field label="管理员命令审计"><Switch checked={config.adminLogging} onChange={(v) => set('adminLogging', v)} /></Field>
        <Field label="聊天记录"><Switch checked={config.chatLogging} onChange={(v) => set('chatLogging', v)} /></Field>
        <Field label="时间戳"><Switch checked={config.logTimestamp} onChange={(v) => set('logTimestamp', v)} /></Field>
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
      <Tabs activeKey={activeTab} onChange={setActiveTab} items={tabs} />
      <div className="panel-footer-actions">
        <div className="panel-footer-actions__buttons">
          <Button onClick={onSave}>仅保存配置</Button>
          <Button type="primary" icon={<ReloadOutlined />} onClick={onApply}>保存并应用重启</Button>
        </div>
      </div>

      <Modal title="添加 CurseForge MOD" open={modModalOpen} onCancel={() => setModModalOpen(false)} onOk={addMod} okText="添加到列表" cancelText="取消">
        <Paragraph type="secondary">输入 ASA CurseForge 项目 ID。原型会将其加入 ActiveMods 队列，正式接入后可从 CurseForge API 获取名称、版本与文件大小。</Paragraph>
        <Input autoFocus value={modId} onChange={(e) => setModId(e.target.value.replace(/\D/g, ''))} onPressEnter={addMod} placeholder="例如：928708" prefix={<BugOutlined />} />
      </Modal>
    </div>
  )
}
