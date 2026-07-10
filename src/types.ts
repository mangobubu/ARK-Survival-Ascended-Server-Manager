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
  enablePvPGamma: boolean
  allowHitMarkers: boolean
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
  structureLimit: number
  platformStructureMultiplier: number
  disablePlacementCollision: boolean
  pveAllowStructuresAtSupplyDrops: boolean
  enableExtraStructurePreventionVolumes: boolean
  maxTribeSize: number
  tribeAlliances: boolean
  pveStructureDecay: boolean
  allowCaveBuildingPvE: boolean
  allowCaveBuildingPvP: boolean
  structureDamageRepairCooldown: number
  structurePickupTimeAfterPlacement: number
  structurePickupHoldDuration: number
  autoDestroyOldStructuresMultiplier: number
  fastDecayUnsnappedCoreStructures: boolean
  limitGeneratorsNum: number
  limitGeneratorsRange: number
  allowCryoFridgeOnSaddle: boolean
  disableCryopodEnemyCheck: boolean
  disableCryopodFridgeRequirement: boolean
  disableCryopodCooldown: boolean
  allowSpeedLeveling: boolean
  allowFlyerSpeedLeveling: boolean
  forceAllowCaveFlyers: boolean
  allowFlyingStaminaRecovery: boolean
  raidDinoFoodDrainMultiplier: number
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
  clusterDirOverride: string
  noTransferFromFiltering: boolean
  enableIdlePlayerKick: boolean
  kickIdlePlayersPeriod: number
  enableDiseases: boolean
  nonPermanentDiseases: boolean
  tribeNameChangeCooldown: number
  maxAlliancesPerTribe: number
  maxTribesPerAlliance: number
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
  customLaunchArgs: string
  serverGameLog: boolean
  serverGameLogIncludeTribe: boolean
  adminLogging: boolean
  chatLogging: boolean
  logTimestamp: boolean
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
