import type { GlobalSettings, LogLine, ModItem, ServerConfig, ServerInstance } from './types'

export const defaultGlobalSettings: GlobalSettings = {
  steamCmdPath: 'C:\\SteamCMD',
  serverStoragePath: 'D:\\ASA-Server',
  backupStoragePath: 'D:\\ASA-Backups',
  language: 'zh-CN',
  theme: 'dark',
  autoUpdateOnStart: true,
  autoRestartOnCrash: true,
  maxBackupRetention: 7,
}

export const initialInstances: ServerInstance[] = []

export const serverMapOptions = [
  { name: 'The Island', zhName: '孤岛', code: 'TheIsland_WP' },
  { name: 'Scorched Earth', zhName: '焦土', code: 'ScorchedEarth_WP' },
  { name: 'The Center', zhName: '中心岛', code: 'TheCenter_WP' },
  { name: 'Aberration', zhName: '畸变', code: 'Aberration_WP' },
  { name: 'Extinction', zhName: '灭绝', code: 'Extinction_WP' },
  { name: 'Astraeos', zhName: '阿斯特拉奥斯', code: 'Astraeos_WP' },
  { name: 'Ragnarok', zhName: '仙境', code: 'Ragnarok_WP' },
  { name: 'Valguero', zhName: '瓦尔盖罗', code: 'Valguero_WP' },
  { name: 'Lost Colony', zhName: '失落殖民地', code: 'LostColony_WP' },
]

export const initialLogs: LogLine[] = []

export const initialMods: ModItem[] = []

export const defaultConfig: ServerConfig = {
  sessionName: '方舟进化 · 孤岛生存', serverPassword: '', spectatorPassword: '', adminPassword: 'ark-admin-2026', gamePort: 7777, queryPort: 27015,
  rconEnabled: true, rconPort: 32330, visibility: 'public', clusterId: 'Boat-ASA-Cluster', crossTransfer: true, maxPlayers: 30,
  pve: true, hardcore: false, disableFriendlyFire: false, enablePvPGamma: true, allowHitMarkers: true,
  difficulty: 5, xpMultiplier: 1.5, tamingSpeed: 3, harvestAmount: 2, harvestHealthMultiplier: 1,
  playerDamageMultiplier: 1, playerResistanceMultiplier: 1, dinoDamageMultiplier: 1, dinoResistanceMultiplier: 1,
  tamedDinoDamageMultiplier: 1, tamedDinoResistanceMultiplier: 1, playerFoodDrainMultiplier: 1, playerWaterDrainMultiplier: 1,
  playerStaminaDrainMultiplier: 1, dinoFoodDrainMultiplier: 1, dinoStaminaDrainMultiplier: 1,
  thirdPerson: true, crosshair: true, showMapPlayer: true, flyerCarry: true,
  autoRestart: true, restartTime: '04:00', saveInterval: 15, backupRetention: 7, autoUpdateServer: true, autoUpdateMods: true, restartOnCrash: true, saveOnStop: true,
  dayCycleSpeed: 1, dayTimeSpeed: 1, nightTimeSpeed: 1.5, resourceRespawn: 0.7,
  resourceNoReplenishRadiusPlayers: 1, resourceNoReplenishRadiusStructures: 1, dinoCount: 1, maxTamedDinos: 5000, destroyWildDinos: false,
  cropGrowthSpeedMultiplier: 1, cropDecaySpeedMultiplier: 1, supplyCrateLootQualityMultiplier: 1, fishingLootQualityMultiplier: 1,
  fuelConsumptionIntervalMultiplier: 1, itemStackSizeMultiplier: 1, globalSpoilingTimeMultiplier: 1,
  globalItemDecompositionTimeMultiplier: 1, globalCorpseDecompositionTimeMultiplier: 1,
  matingInterval: 0.25, matingSpeedMultiplier: 1, eggHatchSpeed: 10, babyMatureSpeed: 20, cuddleInterval: 0.1, babyFoodConsumption: 0.5,
  layEggIntervalMultiplier: 1, babyCuddleGracePeriodMultiplier: 1, babyCuddleLoseImprintQualitySpeedMultiplier: 1,
  babyImprintingStatScaleMultiplier: 1, babyImprintAmountMultiplier: 1, allowAnyoneBabyImprintCuddle: false,
  structureLimit: 10500, platformStructureMultiplier: 1.5, disablePlacementCollision: true, maxTribeSize: 8, tribeAlliances: true, pveStructureDecay: false,
  allowCaveBuildingPvE: false, allowCaveBuildingPvP: true, structureDamageRepairCooldown: 180,
  structurePickupTimeAfterPlacement: 30, structurePickupHoldDuration: 0.5, autoDestroyOldStructuresMultiplier: 1,
  fastDecayUnsnappedCoreStructures: false, limitGeneratorsNum: 3, limitGeneratorsRange: 15000,
  allowCryoFridgeOnSaddle: false, disableCryopodEnemyCheck: false, disableCryopodFridgeRequirement: false, disableCryopodCooldown: false,
  allowFlyerSpeedLeveling: false, forceAllowCaveFlyers: false, allowFlyingStaminaRecovery: false, raidDinoFoodDrainMultiplier: 1,
  whitelist: false, exclusiveJoin: false, preventDownloadItems: false, preventDownloadDinos: false, preventDownloadSurvivors: false,
  preventUploadItems: false, preventUploadDinos: false, preventUploadSurvivors: false, noTributeDownloads: false,
  minimumDinoReuploadInterval: 0, tributeCharacterExpirationSeconds: 0, tributeDinoExpirationSeconds: 0, tributeItemExpirationSeconds: 0,
  clusterDirOverride: 'ShooterGame/Saved/clusters', noTransferFromFiltering: true,
  enableIdlePlayerKick: false, kickIdlePlayersPeriod: 3600, enableDiseases: true, nonPermanentDiseases: false,
  tribeNameChangeCooldown: 15, maxAlliancesPerTribe: 0, maxTribesPerAlliance: 0,
  processPriority: 'aboveNormal', cpuAffinity: '自动', memoryWarningGb: 24, useAllCores: true, noBattlEye: true,
  networkTickRate: 30, maxClientRate: 100000, rconBufferSize: 6000, compressBackups: true, snapshotBeforeRestart: true,
  preventHibernation: false, stasisKeepControllers: false, useStructureStasisGrid: true, alwaysTickDedicatedSkeletalMeshes: false,
  gbUsageToForceRestart: 35, serverPlatform: 'ALL', activeEvent: '', useDynamicConfig: false, customDynamicConfigUrl: '',
  customLaunchArgs: '-culture=zh',
  serverGameLog: true, serverGameLogIncludeTribe: true, adminLogging: true, chatLogging: true, logTimestamp: true,
  logLevel: 'normal', rotateSizeMb: 100, logRetentionDays: 14, logPath: 'ShooterGame/Saved/Logs',
  crossArkAllowForeignDinoDownloads: false,
  limitBunkersPerTribe: true, limitBunkersPerTribeNum: 3, allowBunkersInPreventionZones: false,
  allowRidingDinosInsideBunkers: true, allowBunkerModulesAboveGround: false, allowDinoAIInsideBunkers: true,
  allowBunkerModulesInPreventionZones: false, minDistanceBetweenBunkers: 3000,
  enemyAccessBunkerHPThreshold: 0.25, bunkerUnderHPThresholdDmgMultiplier: 0.05,
}
