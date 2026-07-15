export type ServerStatus = 'running' | 'stopped' | 'stopping' | 'starting' | 'updating' | 'backingUp' | 'error'

export interface WebIpWhitelistEntry {
  value: string
  group: string
  note: string
}

export interface WebSecurityBanRecord {
  ip: string
  reason: string
  source: 'login' | 'ua' | 'body' | 'path' | 'rate' | 'security' | string
  bannedAtMs: number
  remainingSeconds: number
}

export interface WebSecurityUnbanResult {
  ip: string
  existed: boolean
}

export interface WebAcmeCertificateStatus {
  domain: string
  issuedAtUnix: number | null
  renewAfterUnix: number
  expiresAtUnix: number
  fullchainPem: string
  privateKeyPem: string
}

export type AsaConfigTarget =
  | 'managerOnly'
  | 'launchArgument'
  | 'gameUserSettingsServerSettings'
  | 'gameIniShooterGameMode'
  | 'engineIniIpNetDriver'

export interface AsaConfigFieldMetadata {
  key: keyof ServerConfig | string
  target: AsaConfigTarget
  defaultValue: unknown
  sensitiveExport: boolean
}

export interface AsaConfigMetadataDocument {
  fields: AsaConfigFieldMetadata[]
  sensitiveExportKeys: string[]
  dynamicInstanceKeys: string[]
}

export interface GlobalSettings {
  steamCmdPath: string
  serverStoragePath: string
  backupStoragePath: string
  language: 'zh-CN' | 'en-US'
  theme: 'dark' | 'light' | 'system'
  windowCloseBehavior: 'askEveryTime' | 'minimizeToTray' | 'exitApp'
  globalToggleShortcutKey: string
  hideTrayIconWhenMinimized: boolean
  autoUpdateOnStart: boolean
  autoRestartOnCrash: boolean
  maxBackupRetention: number
  curseforgeApiKey: string
  curseforgeApiKeyConfigured: boolean
  webManagementEnabled: boolean
  webServerPort: number
  webAdminUsername: string
  webAdminPassword: string
  webAdminPasswordConfigured: boolean
  webReverseProxyEnabled: boolean
  webReverseProxyDomain: string
  webReverseProxyPort: number
  webReverseProxyOpenRestyPath: string
  webHttpsEnabled: boolean
  webAcmeAutoIssueEnabled: boolean
  webAcmeDirectoryUrl: string
  webAcmeAccountEmail: string
  webAcmeTencentSecretId: string
  webAcmeTencentSecretKey: string
  webAcmeTencentSecretKeyConfigured: boolean
  webLoginFailureBanThreshold: number
  webLoginFailureBanSeconds: number
  webCaptchaCharset: string
  webCaptchaFontSize: number
  webCaptchaNoisePoints: number
  webCaptchaLength: number
  webIpWhitelist: WebIpWhitelistEntry[]
}

export interface SteamCmdCheck {
  path: string
  executablePath: string
  valid: boolean
  reason: string | null
}

export interface SteamCmdProgress {
  phase: 'downloading' | 'extracting' | 'initializing' | 'completed'
  downloadedBytes: number
  totalBytes: number | null
  bytesPerSecond: number
  message: string
}

export interface SteamCmdInstallResult {
  path: string
  executablePath: string
}

export interface ServerInstance {
  id: string
  name: string
  map: string
  mapCode: string
  mode: 'PvE' | 'PvP'
  status: ServerStatus
  gamePort: number
  queryPort: number
  players: number
  maxPlayers: number
  installPath: string
  rconPort: number
  clusterId: string
  description: string
  pid: number | null
  lastStartedAt: string | null
  lastStoppedAt: string | null
  serverVersion: string
  versionState: string
  lastError: string | null
  skipAutoUpdateOnStartOnce?: boolean
}

export interface AddInstancePayload {
  id?: string
  name: string
  map: string
  mapCode: string
  mode: 'PvE' | 'PvP'
  status?: ServerStatus
  gamePort: number
  queryPort: number
  players?: number
  maxPlayers: number
  installPath: string
  rconPort: number
  clusterId: string
  serverPassword: string
  adminPassword: string
  autoInstall: boolean
  description: string
  importedConfig?: Partial<ServerConfig>
  importedMods?: ModItem[]
}

export type InstancePortKind = 'gamePort' | 'queryPort' | 'rconPort'

export interface PortCheckResult {
  port: number
  available: boolean
  exists: boolean
  suggestedPort: number | null
  reason: string | null
}

export interface InstanceCreatedEvent {
  instance: ServerInstance
  autoInstall: boolean
}

export interface ModItem {
  id: string
  name: string
  version: string
  size: string
  enabled: boolean
  updateAvailable?: boolean
}

export interface CurseForgeModSummary {
  id: string
  name: string
  summary: string
  author: string
  version: string
  size: string
  downloadCount: number
  dateModified: string
  thumbnailUrl: string | null
  websiteUrl: string
}

export interface CurseForgeModSearchResult {
  items: CurseForgeModSummary[]
  totalCount: number
}

export interface ItemStackOption {
  label: string
  zhLabel?: string
  classString: string
  category: string
  zhCategory?: string
  defaultStackSize: number
}

export interface ItemStackOverride {
  itemClassString: string
  maxItemQuantity: number
  ignoreMultiplier: boolean
}

export interface LogLine {
  id: number
  time: string
  source: 'application' | 'server'
  serverLogKind?: 'console' | 'file' | null
  instance: string
  level: 'info' | 'success' | 'warn' | 'error'
  message: string
}

export interface LogClearScope {
  source: LogLine['source']
  instance?: string | null
  serverLogKind?: NonNullable<LogLine['serverLogKind']> | null
}

export interface JobProgress {
  jobId: string
  instanceId: string | null
  phase: string
  percent: number | null
  message: string
  detail: string | null
  downloadedBytes: number
  totalBytes: number | null
  bytesPerSecond: number
}

export interface BackupItem {
  instanceId: string
  instanceName: string
  path: string
  sizeBytes: number
  createdAt: string
}

export interface ExportResult {
  path: string
  exportedInstances: number
}

export interface ImportResult {
  importedInstances: number
  skippedInstances: number
}

export type InstanceFileEntryType = 'directory' | 'file' | 'other'

export interface InstanceFileEntry {
  name: string
  path: string
  entryType: InstanceFileEntryType
  sizeBytes: number | null
  modifiedAt: number | null
  hasChildren: boolean
}

export interface InstanceDirectoryListing {
  rootPath: string
  currentPath: string
  parentPath: string | null
  entries: InstanceFileEntry[]
  totalEntries: number
  truncated: boolean
}

export interface ServerConfig {
  sessionName: string
  serverPassword: string
  spectatorPassword: string
  adminPassword: string
  gamePort: number
  queryPort: number
  rconEnabled: boolean
  rconPort: number
  visibility: 'public' | 'private'
  clusterId: string
  crossTransfer: boolean
  maxPlayers: number
  pve: boolean
  hardcore: boolean
  disableFriendlyFire: boolean
  disableFriendlyFirePvP: boolean
  enablePvPGamma: boolean
  allowHitMarkers: boolean
  allowHideDamageSourceFromLogs: boolean
  globalVoiceChat: boolean
  proximityChat: boolean
  preventSpawnAnimations: boolean
  serverForceNoHud: boolean
  showFloatingDamageText: boolean
  showPlayerJoinNotifications: boolean
  difficulty: number
  xpMultiplier: number
  tamingSpeed: number
  harvestAmount: number
  harvestHealthMultiplier: number
  playerDamageMultiplier: number
  playerResistanceMultiplier: number
  dinoDamageMultiplier: number
  dinoResistanceMultiplier: number
  tamedDinoDamageMultiplier: number
  tamedDinoResistanceMultiplier: number
  playerFoodDrainMultiplier: number
  playerWaterDrainMultiplier: number
  playerStaminaDrainMultiplier: number
  dinoFoodDrainMultiplier: number
  dinoStaminaDrainMultiplier: number
  dinoHealthRecoveryMultiplier: number
  oxygenSwimSpeedStatMultiplier: number
  playerHealthRecoveryMultiplier: number
  thirdPerson: boolean
  crosshair: boolean
  showMapPlayer: boolean
  flyerCarry: boolean
  autoRestart: boolean
  restartTime: string
  saveInterval: number
  backupRetention: number
  autoUpdateServer: boolean
  autoUpdateMods: boolean
  restartOnCrash: boolean
  saveOnStop: boolean
  dayCycleSpeed: number
  dayTimeSpeed: number
  nightTimeSpeed: number
  resourceRespawn: number
  resourceNoReplenishRadiusPlayers: number
  resourceNoReplenishRadiusStructures: number
  dinoCount: number
  maxTamedDinos: number
  destroyWildDinos: boolean
  cropGrowthSpeedMultiplier: number
  cropDecaySpeedMultiplier: number
  supplyCrateLootQualityMultiplier: number
  fishingLootQualityMultiplier: number
  fuelConsumptionIntervalMultiplier: number
  clampItemSpoilingTimes: boolean
  clampResourceHarvestDamage: boolean
  randomSupplyCratePoints: boolean
  itemStackSizeMultiplier: number
  itemStackOverrides: ItemStackOverride[]
  globalSpoilingTimeMultiplier: number
  globalItemDecompositionTimeMultiplier: number
  globalCorpseDecompositionTimeMultiplier: number
  matingInterval: number
  matingSpeedMultiplier: number
  eggHatchSpeed: number
  babyMatureSpeed: number
  cuddleInterval: number
  babyFoodConsumption: number
  layEggIntervalMultiplier: number
  babyCuddleGracePeriodMultiplier: number
  babyCuddleLoseImprintQualitySpeedMultiplier: number
  babyImprintingStatScaleMultiplier: number
  babyImprintAmountMultiplier: number
  allowAnyoneBabyImprintCuddle: boolean
  disableImprintDinoBuff: boolean
  preventMateBoost: boolean
  poopIntervalMultiplier: number
  wildDinoFoodDrainMultiplier: number
  structureLimit: number
  platformStructureMultiplier: number
  platformSaddleBuildAreaBoundsMultiplier: number
  structureResistanceMultiplier: number
  disablePlacementCollision: boolean
  pveAllowStructuresAtSupplyDrops: boolean
  enableExtraStructurePreventionVolumes: boolean
  maxTribeSize: number
  tribeAlliances: boolean
  pveStructureDecay: boolean
  allowCaveBuildingPvE: boolean
  allowCaveBuildingPvP: boolean
  structureDamageRepairCooldown: number
  alwaysAllowStructurePickup: boolean
  structurePickupTimeAfterPlacement: number
  structurePickupHoldDuration: number
  autoDestroyOldStructuresMultiplier: number
  fastDecayUnsnappedCoreStructures: boolean
  limitGeneratorsNum: number
  limitGeneratorsRange: number
  allowMultipleAttachedC4: boolean
  forceAllStructureLocking: boolean
  disableWirelessCrafting: boolean
  wirelessCraftingRangeOverride: number
  allowCryoFridgeOnSaddle: boolean
  disableCryopodEnemyCheck: boolean
  disableCryopodFridgeRequirement: boolean
  disableCryopodCooldown: boolean
  allowSpeedLeveling: boolean
  allowFlyerSpeedLeveling: boolean
  forceAllowCaveFlyers: boolean
  allowFlyingStaminaRecovery: boolean
  raidDinoFoodDrainMultiplier: number
  allowRaidDinoFeeding: boolean
  maxPersonalTamedDinos: number
  maxTamedDinosSoftTameLimit: number
  maxTamedDinosSoftTameLimitCountdown: number
  destroyTamesOverSoftTameLimit: boolean
  useDinoLevelUpAnimations: boolean
  whitelist: boolean
  exclusiveJoin: boolean
  preventDownloadItems: boolean
  preventDownloadDinos: boolean
  preventDownloadSurvivors: boolean
  preventUploadItems: boolean
  preventUploadDinos: boolean
  preventUploadSurvivors: boolean
  noTributeDownloads: boolean
  minimumDinoReuploadInterval: number
  tributeCharacterExpirationSeconds: number
  tributeDinoExpirationSeconds: number
  tributeItemExpirationSeconds: number
  maxTributeDinos: number
  maxTributeItems: number
  clusterDirOverride: string
  noTransferFromFiltering: boolean
  enableIdlePlayerKick: boolean
  kickIdlePlayersPeriod: number
  enableDiseases: boolean
  nonPermanentDiseases: boolean
  tribeNameChangeCooldown: number
  maxAlliancesPerTribe: number
  maxTribesPerAlliance: number
  preventOfflinePvP: boolean
  preventOfflinePvPInterval: number
  pveDinoDecay: boolean
  pveDinoDecayPeriodMultiplier: number
  pvpDinoDecay: boolean
  allowUnlimitedRespecs: boolean
  craftingSkillBonusMultiplier: number
  craftXpMultiplier: number
  customRecipeEffectivenessMultiplier: number
  customRecipeSkillMultiplier: number
  genericXpMultiplier: number
  harvestXpMultiplier: number
  killXpMultiplier: number
  specialXpMultiplier: number
  hairGrowthSpeedMultiplier: number
  maxFallSpeedMultiplier: number
  disablePhotoMode: boolean
  photoModeRangeLimit: number
  showCreativeMode: boolean
  processPriority: 'normal' | 'aboveNormal' | 'high'
  cpuAffinity: string
  memoryWarningGb: number
  useAllCores: boolean
  noBattlEye: boolean
  networkTickRate: number
  maxClientRate: number
  rconBufferSize: number
  compressBackups: boolean
  snapshotBeforeRestart: boolean
  preventHibernation: boolean
  stasisKeepControllers: boolean
  useStructureStasisGrid: boolean
  alwaysTickDedicatedSkeletalMeshes: boolean
  gbUsageToForceRestart: number
  serverPlatform: 'ALL' | 'PC' | 'PS5' | 'XSX' | 'WINGDK'
  activeEvent: string
  useDynamicConfig: boolean
  customDynamicConfigUrl: string
  noDinos: boolean
  noWildBabies: boolean
  disableCustomCosmetics: boolean
  unstasisDinoObstructionCheck: boolean
  useServerNetSpeedCheck: boolean
  noSound: boolean
  customLaunchArgs: string
  customServerSettings: string
  customGameIniSettings: string
  customEngineIniSettings: string
  serverGameLog: boolean
  serverGameLogIncludeTribe: boolean
  adminLogging: boolean
  chatLogging: boolean
  logLevel: 'normal' | 'verbose' | 'debug'
  rotateSizeMb: number
  logRetentionDays: number
  logPath: string
  crossArkAllowForeignDinoDownloads: boolean
  limitBunkersPerTribe: boolean
  limitBunkersPerTribeNum: number
  allowBunkersInPreventionZones: boolean
  allowRidingDinosInsideBunkers: boolean
  allowBunkerModulesAboveGround: boolean
  allowDinoAIInsideBunkers: boolean
  allowBunkerModulesInPreventionZones: boolean
  minDistanceBetweenBunkers: number
  enemyAccessBunkerHPThreshold: number
  bunkerUnderHPThresholdDmgMultiplier: number
}

export interface ImportedServerConfigPreview {
  installPath: string
  name: string | null
  map: string | null
  mapCode: string | null
  mode: 'PvE' | 'PvP' | null
  gamePort: number | null
  queryPort: number | null
  rconPort: number | null
  maxPlayers: number | null
  clusterId: string | null
  serverPassword: string | null
  adminPassword: string | null
  config: Partial<ServerConfig>
  mods: ModItem[]
  foundFiles: string[]
  warnings: string[]
}

export interface HostDirectoryEntry {
  name: string
  path: string
  hasChildren: boolean
  serverConfigDetected: boolean
  serverExecutableDetected: boolean
}

export interface HostDirectoryListing {
  rootPath: string
  currentPath: string
  parentPath: string | null
  entries: HostDirectoryEntry[]
  totalEntries: number
  truncated: boolean
}
