import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import ts from 'typescript'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

function readProjectFile(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function readProjectFiles(relativePaths) {
  return relativePaths.map((relativePath) => readProjectFile(relativePath)).join('\n')
}

function listProjectFilesRecursive(relativeDir, extension) {
  const absoluteDir = path.join(repoRoot, relativeDir)
  if (!fs.existsSync(absoluteDir)) return []

  const result = []
  for (const entry of fs.readdirSync(absoluteDir, { withFileTypes: true })) {
    const relativePath = path.join(relativeDir, entry.name).replaceAll(path.sep, '/')
    if (entry.isDirectory()) {
      result.push(...listProjectFilesRecursive(relativePath, extension))
    } else if (entry.isFile() && entry.name.endsWith(extension)) {
      result.push(relativePath)
    }
  }
  return sorted(result)
}

function parseTypeScript(relativePath) {
  return ts.createSourceFile(
    relativePath,
    readProjectFile(relativePath),
    ts.ScriptTarget.Latest,
    true,
    relativePath.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  )
}

function sorted(values) {
  return [...values].sort((a, b) => a.localeCompare(b))
}

function asSet(values, label) {
  const set = new Set()
  const duplicated = []
  for (const value of values) {
    if (set.has(value)) duplicated.push(value)
    set.add(value)
  }
  if (duplicated.length > 0) {
    throw new Error(`${label} 存在重复字段：${sorted(duplicated).join(', ')}`)
  }
  return set
}

function propertyNameText(name) {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) return name.text
  return null
}

function getInterfaceKeys(sourceFile, interfaceName) {
  const keys = []
  sourceFile.forEachChild((node) => {
    if (!ts.isInterfaceDeclaration(node) || node.name.text !== interfaceName) return
    for (const member of node.members) {
      if (!ts.isPropertySignature(member) || !member.name) continue
      const key = propertyNameText(member.name)
      if (key) keys.push(key)
    }
  })
  if (keys.length === 0) throw new Error(`未找到 TypeScript 接口：${interfaceName}`)
  return asSet(keys, `${interfaceName} 接口`)
}

function getExportedObjectKeys(sourceFile, constName) {
  const keys = []
  sourceFile.forEachChild((node) => {
    if (!ts.isVariableStatement(node)) return
    for (const declaration of node.declarationList.declarations) {
      if (!ts.isIdentifier(declaration.name) || declaration.name.text !== constName) continue
      const initializer = declaration.initializer
      if (!initializer || !ts.isObjectLiteralExpression(initializer)) {
        throw new Error(`${constName} 必须是对象字面量，方便进行配置元数据一致性校验`)
      }
      for (const property of initializer.properties) {
        if (ts.isPropertyAssignment(property) || ts.isShorthandPropertyAssignment(property)) {
          const key = propertyNameText(property.name)
          if (key) keys.push(key)
        }
      }
    }
  })
  if (keys.length === 0) throw new Error(`未找到对象常量：${constName}`)
  return asSet(keys, `${constName} 对象`)
}

function getStaticSetCallKeys(sourceFile) {
  const keys = new Set()
  const dynamicCalls = []

  function visit(node) {
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression) && node.expression.text === 'set') {
      const keyArgument = node.arguments[0]
      if (keyArgument && (ts.isStringLiteral(keyArgument) || ts.isNoSubstitutionTemplateLiteral(keyArgument))) {
        keys.add(keyArgument.text)
      } else {
        const position = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile))
        dynamicCalls.push(`${sourceFile.fileName}:${position.line + 1}`)
      }
    }
    ts.forEachChild(node, visit)
  }

  visit(sourceFile)
  if (dynamicCalls.length > 0) {
    throw new Error(
      `ConfigPanel 中的 set 调用必须使用字符串字面量配置键，无法静态校验的位置：${dynamicCalls.join(', ')}`,
    )
  }
  return keys
}

function extractRustStringArray(sourceText, constName) {
  const pattern = new RegExp(`pub\\s+const\\s+${constName}\\s*:[^=]+=\\s*&\\[(?<body>[\\s\\S]*?)\\];`, 'm')
  const match = sourceText.match(pattern)
  if (!match?.groups?.body) throw new Error(`未找到 Rust 字符串数组常量：${constName}`)
  return asSet([...match.groups.body.matchAll(/"([^"]+)"/g)].map((item) => item[1]), constName)
}

function extractRustDefaultEntries(sourceText) {
  const entries = [...sourceText.matchAll(/default\(\s*"([^"]+)"\s*,[\s\S]*?\bTarget::([A-Za-z][A-Za-z0-9_]*)\s*,?\s*\)/g)]
    .map((match) => ({ key: match[1], target: match[2] }))
  if (entries.length === 0) throw new Error('未从 asa_config_defaults 提取到任何配置默认项及 target')
  asSet(entries.map((entry) => entry.key), 'ASA_CONFIG_DEFAULTS')
  return entries
}

function withoutRustTests(sourceText) {
  const testModuleIndex = sourceText.search(/^\s*#\[cfg\(test\)\]/m)
  return testModuleIndex >= 0 ? sourceText.slice(0, testModuleIndex) : sourceText
}

function extractRustFunctions(relativePath) {
  const sourceText = withoutRustTests(readProjectFile(relativePath))
  const matches = [...sourceText.matchAll(/^\s*(?:pub(?:\([^\r\n)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/gm)]
  return matches.map((match, index) => ({
    id: `${relativePath}:${match.index}`,
    name: match[1],
    relativePath,
    source: sourceText.slice(match.index, matches[index + 1]?.index ?? sourceText.length),
  }))
}

function isExcludedRustConsumerPath(relativePath) {
  const normalized = relativePath.replaceAll('\\', '/')
  return normalized.includes('/asa_config_defaults/')
    || normalized.endsWith('/asa_config_defaults.rs')
    || normalized.includes('/instance_config_import/')
    || normalized.endsWith('/instance_config_import.rs')
    || normalized.endsWith('/asa_config_metadata.rs')
    || /(?:^|\/)tests?(?:\/|\.rs$)/.test(normalized)
}

function rustConsumerFiles() {
  const coreFiles = [
    'src-tauri/src/ark_config_launch.rs',
    'src-tauri/src/ark_config_game_user_settings.rs',
    ...listProjectFilesRecursive('src-tauri/src/ark_config_game_user_settings', '.rs'),
    'src-tauri/src/ark_config_ini.rs',
    'src-tauri/src/ark_config_values.rs',
    ...listProjectFilesRecursive('src-tauri/src/ark_config_launch', '.rs'),
    ...listProjectFilesRecursive('src-tauri/src/ark_config_ini', '.rs'),
  ]
  const customHelpers = listProjectFilesRecursive('src-tauri/src', '.rs')
    .filter((relativePath) => /ark_config.*custom|custom.*ark_config/i.test(relativePath.replaceAll('\\', '/')))

  return sorted(new Set([...coreFiles, ...customHelpers]))
    .filter((relativePath) => fs.existsSync(path.join(repoRoot, relativePath)))
    .filter((relativePath) => !isExcludedRustConsumerPath(relativePath))
}

function buildRustFunctionIndex(relativePaths) {
  const functions = relativePaths.flatMap((relativePath) => extractRustFunctions(relativePath))
  const byName = new Map()
  for (const item of functions) {
    const sameName = byName.get(item.name) ?? []
    sameName.push(item)
    byName.set(item.name, sameName)
  }
  return { functions, byName }
}

function collectRustCallGraphSource(functionIndex, rootFunctionName) {
  const roots = functionIndex.byName.get(rootFunctionName) ?? []
  if (roots.length !== 1) {
    throw new Error(
      roots.length === 0
        ? `未在实际配置渲染模块中找到 Rust 根函数：${rootFunctionName}`
        : `Rust 配置渲染根函数重名，无法确定调用链：${rootFunctionName}`,
    )
  }

  const visited = new Set()
  const queue = [...roots]
  const sources = []
  while (queue.length > 0) {
    const current = queue.shift()
    if (visited.has(current.id)) continue
    visited.add(current.id)
    sources.push(`// ${current.relativePath}::${current.name}\n${current.source}`)

    const calledNames = new Set(
      [...current.source.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/g)].map((match) => match[1]),
    )
    for (const calledName of calledNames) {
      for (const called of functionIndex.byName.get(calledName) ?? []) queue.push(called)
    }
  }
  return sources.join('\n')
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function hasQuotedRustKey(sourceText, key) {
  return new RegExp(`"${escapeRegExp(key)}"`).test(sourceText)
}

function difference(left, right) {
  return sorted([...left].filter((value) => !right.has(value)))
}

function assertSameSet(actual, expected, actualLabel, expectedLabel) {
  const missing = difference(expected, actual)
  const extra = difference(actual, expected)
  if (missing.length === 0 && extra.length === 0) return

  const lines = [`${actualLabel} 与 ${expectedLabel} 不一致：`]
  if (missing.length > 0) lines.push(`- ${actualLabel} 缺少：${missing.join(', ')}`)
  if (extra.length > 0) lines.push(`- ${actualLabel} 多出：${extra.join(', ')}`)
  throw new Error(lines.join('\n'))
}

function validateConfigPanelCoverage(panelKeys, serverConfigKeys) {
  const expectedEditableKeys = new Set([...serverConfigKeys].filter((key) => key !== 'rconEnabled'))
  const missing = difference(expectedEditableKeys, panelKeys)
  const unknown = difference(panelKeys, serverConfigKeys)
  const nonEditable = sorted([...panelKeys].filter((key) => serverConfigKeys.has(key) && !expectedEditableKeys.has(key)))
  if (missing.length === 0 && unknown.length === 0 && nonEditable.length === 0) return []

  const lines = ['ConfigPanel 配置编辑覆盖不完整：']
  if (missing.length > 0) lines.push(`- 缺少 ServerConfig 字段的 set('key', ...) 调用：${missing.join(', ')}`)
  if (unknown.length > 0) lines.push(`- set('key', ...) 使用了未知字段：${unknown.join(', ')}`)
  if (nonEditable.length > 0) lines.push(`- 不应由 ConfigPanel 编辑的字段：${nonEditable.join(', ')}`)
  return [lines.join('\n')]
}

const rustTargetRoots = new Map([
  ['LaunchArgument', 'build_launch_arguments'],
  ['GameUserSettingsServerSettings', 'render_game_user_settings'],
  ['GameIniShooterGameMode', 'render_game_ini'],
  ['EngineIniIpNetDriver', 'render_engine_ini'],
])

const rustTargetLabels = new Map([
  ['LaunchArgument', '启动参数'],
  ['GameUserSettingsServerSettings', 'GameUserSettings.ini [ServerSettings]'],
  ['GameIniShooterGameMode', 'Game.ini ShooterGameMode'],
  ['EngineIniIpNetDriver', 'Engine.ini IpNetDriver'],
])

const fixedRustConsumerEvidence = new Map([
  ['GameUserSettingsServerSettings:rconEnabled', /RCONEnabled\s*=\s*True/],
])

const dynamicInstanceConsumerTargets = new Map([
  ['sessionName', ['GameUserSettingsServerSettings', 'LaunchArgument']],
  ['serverPassword', ['GameUserSettingsServerSettings', 'LaunchArgument']],
  ['adminPassword', ['GameUserSettingsServerSettings', 'LaunchArgument']],
  ['gamePort', ['GameUserSettingsServerSettings', 'LaunchArgument']],
  ['queryPort', ['GameUserSettingsServerSettings', 'LaunchArgument']],
  ['rconPort', ['GameUserSettingsServerSettings', 'LaunchArgument']],
  ['clusterId', ['LaunchArgument']],
  ['maxPlayers', ['LaunchArgument']],
  ['pve', ['GameUserSettingsServerSettings', 'LaunchArgument']],
  // 管理器在启动生命周期中消费，不写入 ASA 配置文件或启动参数。
  ['autoUpdateServer', []],
])

function validateRustConfigConsumers(staticDefaultEntries, dynamicInstanceKeys) {
  const consumerFiles = rustConsumerFiles()
  const functionIndex = buildRustFunctionIndex(consumerFiles)
  const sourceByTarget = new Map(
    [...rustTargetRoots].map(([target, root]) => [target, collectRustCallGraphSource(functionIndex, root)]),
  )
  const missingByTarget = new Map()

  function requireConsumer(key, target, origin) {
    const sourceText = sourceByTarget.get(target)
    if (!sourceText) {
      const current = missingByTarget.get(target) ?? []
      current.push(`${key}（${origin}，未知 target）`)
      missingByTarget.set(target, current)
      return
    }
    const fixedEvidence = fixedRustConsumerEvidence.get(`${target}:${key}`)
    if (hasQuotedRustKey(sourceText, key) || fixedEvidence?.test(sourceText)) return
    const current = missingByTarget.get(target) ?? []
    current.push(`${key}（${origin}）`)
    missingByTarget.set(target, current)
  }

  for (const entry of staticDefaultEntries) {
    if (entry.target === 'ManagerOnly') continue
    requireConsumer(entry.key, entry.target, '静态元数据')
  }

  const mappedDynamicKeys = new Set(dynamicInstanceConsumerTargets.keys())
  const missingDynamicMappings = difference(dynamicInstanceKeys, mappedDynamicKeys)
  const staleDynamicMappings = difference(mappedDynamicKeys, dynamicInstanceKeys)
  const errors = []
  if (missingDynamicMappings.length > 0 || staleDynamicMappings.length > 0) {
    const lines = ['动态实例字段的 Rust 消费映射不完整：']
    if (missingDynamicMappings.length > 0) lines.push(`- 缺少明确映射/白名单：${missingDynamicMappings.join(', ')}`)
    if (staleDynamicMappings.length > 0) lines.push(`- 映射中存在已移除字段：${staleDynamicMappings.join(', ')}`)
    errors.push(lines.join('\n'))
  }

  for (const [key, targets] of dynamicInstanceConsumerTargets) {
    if (!dynamicInstanceKeys.has(key)) continue
    for (const target of targets) requireConsumer(key, target, '动态实例字段')
  }

  if (missingByTarget.size > 0) {
    const lines = ['Rust 配置消费校验失败：字段未在其 target 对应的实际渲染/启动调用链中出现：']
    for (const [target, keys] of sorted([...missingByTarget.keys()]).map((target) => [target, missingByTarget.get(target)])) {
      lines.push(`- ${rustTargetLabels.get(target) ?? target}：${sorted(keys).join(', ')}`)
    }
    lines.push(`- 已扫描模块：${consumerFiles.join(', ')}`)
    errors.push(lines.join('\n'))
  }

  return { errors, consumerFiles, sourceByTarget }
}

const typesSource = parseTypeScript('src/types.ts')
const dataSource = parseTypeScript('src/data.ts')
const configPanelSource = parseTypeScript('src/ConfigPanel.tsx')
const defaultsSource = readProjectFiles([
  'src-tauri/src/asa_config_defaults.rs',
  ...listProjectFilesRecursive('src-tauri/src/asa_config_defaults', '.rs'),
])
const metadataSource = readProjectFile('src-tauri/src/asa_config_metadata.rs')

const serverConfigKeys = getInterfaceKeys(typesSource, 'ServerConfig')
const defaultConfigKeys = getExportedObjectKeys(dataSource, 'defaultConfig')
const configPanelSetKeys = getStaticSetCallKeys(configPanelSource)
const staticDefaultEntries = extractRustDefaultEntries(defaultsSource)
const staticDefaultKeys = new Set(staticDefaultEntries.map((entry) => entry.key))
const dynamicInstanceKeys = extractRustStringArray(metadataSource, 'DYNAMIC_INSTANCE_CONFIG_KEYS')
const sensitiveExportKeys = extractRustStringArray(metadataSource, 'CONFIG_EXPORT_SENSITIVE_KEYS')
const backendMetadataKeys = new Set([...staticDefaultKeys, ...dynamicInstanceKeys])

assertSameSet(defaultConfigKeys, serverConfigKeys, 'defaultConfig', 'ServerConfig')
assertSameSet(backendMetadataKeys, serverConfigKeys, '后端配置元数据字段', 'ServerConfig')

const duplicatedBetweenStaticAndDynamic = sorted([...staticDefaultKeys].filter((key) => dynamicInstanceKeys.has(key)))
if (duplicatedBetweenStaticAndDynamic.length > 0) {
  throw new Error(`静态默认字段与动态实例字段不能重复：${duplicatedBetweenStaticAndDynamic.join(', ')}`)
}

const allowedGlobalSensitiveKeys = new Set(['webAdminPassword', 'webAcmeTencentSecretKey'])
const invalidSensitiveKeys = sorted(
  [...sensitiveExportKeys].filter((key) => !serverConfigKeys.has(key) && !allowedGlobalSensitiveKeys.has(key)),
)
if (invalidSensitiveKeys.length > 0) {
  throw new Error(`敏感导出字段既不是 ServerConfig 字段，也不是允许的全局敏感字段：${invalidSensitiveKeys.join(', ')}`)
}

const rustConsumerValidation = validateRustConfigConsumers(staticDefaultEntries, dynamicInstanceKeys)
const extendedValidationErrors = [
  ...validateConfigPanelCoverage(configPanelSetKeys, serverConfigKeys),
  ...rustConsumerValidation.errors,
]
if (extendedValidationErrors.length > 0) {
  throw new Error(`配置字段端到端一致性校验失败：\n\n${extendedValidationErrors.join('\n\n')}`)
}

console.log([
  '配置元数据一致性校验通过',
  `- ServerConfig 字段数：${serverConfigKeys.size}`,
  `- defaultConfig 字段数：${defaultConfigKeys.size}`,
  `- 后端静态默认字段数：${staticDefaultKeys.size}`,
  `- 后端动态实例字段数：${dynamicInstanceKeys.size}`,
  `- 敏感导出字段数：${sensitiveExportKeys.size}`,
  `- ConfigPanel 可编辑字段数：${configPanelSetKeys.size}`,
  `- Rust 消费模块数：${rustConsumerValidation.consumerFiles.length}`,
].join('\n'))
