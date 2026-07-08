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
    ts.ScriptKind.TS,
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

function extractRustStringArray(sourceText, constName) {
  const pattern = new RegExp(`pub\\s+const\\s+${constName}\\s*:[^=]+=\\s*&\\[(?<body>[\\s\\S]*?)\\];`, 'm')
  const match = sourceText.match(pattern)
  if (!match?.groups?.body) throw new Error(`未找到 Rust 字符串数组常量：${constName}`)
  return asSet([...match.groups.body.matchAll(/"([^"]+)"/g)].map((item) => item[1]), constName)
}

function extractRustDefaultKeys(sourceText) {
  return asSet([...sourceText.matchAll(/default\(\s*"([^"]+)"/g)].map((item) => item[1]), 'ASA_CONFIG_DEFAULTS')
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

const typesSource = parseTypeScript('src/types.ts')
const dataSource = parseTypeScript('src/data.ts')
const defaultsSource = readProjectFiles([
  'src-tauri/src/asa_config_defaults.rs',
  ...listProjectFilesRecursive('src-tauri/src/asa_config_defaults', '.rs'),
])
const metadataSource = readProjectFile('src-tauri/src/asa_config_metadata.rs')

const serverConfigKeys = getInterfaceKeys(typesSource, 'ServerConfig')
const defaultConfigKeys = getExportedObjectKeys(dataSource, 'defaultConfig')
const staticDefaultKeys = extractRustDefaultKeys(defaultsSource)
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

console.log([
  '配置元数据一致性校验通过',
  `- ServerConfig 字段数：${serverConfigKeys.size}`,
  `- defaultConfig 字段数：${defaultConfigKeys.size}`,
  `- 后端静态默认字段数：${staticDefaultKeys.size}`,
  `- 后端动态实例字段数：${dynamicInstanceKeys.size}`,
  `- 敏感导出字段数：${sensitiveExportKeys.size}`,
].join('\n'))
