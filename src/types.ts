export type ServerStatus = 'running' | 'stopped' | 'starting'

export interface GlobalSettings {
  steamCmdPath: string
  serverStoragePath: string
  backupStoragePath: string
  language: 'zh-CN' | 'en-US'
  theme: 'dark' | 'light' | 'system'
  autoUpdateOnStart: boolean
  autoRestartOnCrash: boolean
  maxBackupRetention: number
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
}

export interface ModItem {
  id: string
  name: string
  version: string
  size: string
  enabled: boolean
  updateAvailable?: boolean
}

export interface LogLine {
  id: number
  time: string
  instance: string
  level: 'info' | 'success' | 'warn' | 'error'
  message: string
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
