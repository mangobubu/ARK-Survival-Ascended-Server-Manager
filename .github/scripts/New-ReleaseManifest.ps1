param(
  [string]$PackageLockPath = "package-lock.json",
  [string]$CargoLockPath = "src-tauri/Cargo.lock",
  [string]$DistPath = "dist",
  [string]$OutputPath = "release-sbom.json"
)

$ErrorActionPreference = "Stop"

function Resolve-RequiredPath {
  param(
    [string]$Path,
    [string]$Label
  )

  if (-not (Test-Path -LiteralPath $Path)) {
    throw "$Label 不存在：$Path"
  }
  (Resolve-Path -LiteralPath $Path).Path
}

function Get-RelativePathText {
  param(
    [string]$Root,
    [string]$Path
  )

  $rootFull = [System.IO.Path]::GetFullPath($Root).TrimEnd([char[]]@('\', '/'))
  $pathFull = [System.IO.Path]::GetFullPath($Path)
  if ($pathFull.StartsWith($rootFull, [System.StringComparison]::OrdinalIgnoreCase)) {
    return $pathFull.Substring($rootFull.Length).TrimStart([char[]]@('\', '/')).Replace([string][char]92, "/")
  }
  $pathFull.Replace([string][char]92, "/")
}

function Get-Sha256Text {
  param([string]$Path)
  (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-ToolVersionText {
  param(
    [string]$Command,
    [string[]]$Arguments = @("--version")
  )

  try {
    $output = & $Command @Arguments 2>$null
    if ($LASTEXITCODE -eq 0 -and $output) {
      return [string]($output | Select-Object -First 1)
    }
  } catch {
    return ""
  }
  ""
}

function Read-NpmPackages {
  param([string]$Path)

  $nodeScript = @"
const fs = require('fs');
const lockPath = process.argv[1];
const lock = JSON.parse(fs.readFileSync(lockPath, 'utf8'));
const packages = Object.entries(lock.packages || {})
  .filter(([path, pkg]) => path && pkg && pkg.version)
  .map(([path, pkg]) => ({
    name: pkg.name || path.replace(/^node_modules[\\/]/, ''),
    version: String(pkg.version),
    dev: Boolean(pkg.dev),
    integrity: pkg.integrity ? String(pkg.integrity) : '',
    path,
  }))
  .sort((a, b) =>
    a.name.localeCompare(b.name) ||
    a.version.localeCompare(b.version) ||
    a.path.localeCompare(b.path)
  );
process.stdout.write(JSON.stringify({
  lockfileVersion: lock.lockfileVersion,
  packageCount: packages.length,
  packages,
}));
"@

  $json = & node -e $nodeScript $Path
  if ($LASTEXITCODE -ne 0) {
    throw "读取 package-lock.json 失败：$Path"
  }
  $lockSummary = $json | ConvertFrom-Json
  $packages = @()
  foreach ($package in $lockSummary.packages) {
    $packages += [ordered]@{
      name      = [string]$package.name
      version   = [string]$package.version
      dev       = [bool]$package.dev
      integrity = [string]$package.integrity
      path      = [string]$package.path
    }
  }

  [ordered]@{
    lockfileVersion = $lockSummary.lockfileVersion
    packageCount    = $packages.Count
    packages        = $packages
  }
}

function Read-CargoPackages {
  param([string]$Path)

  $packages = @()
  $current = $null

  function Flush-CargoPackage {
    if ($null -eq $script:currentCargoPackage) {
      return
    }
    if (-not $script:currentCargoPackage.ContainsKey("name")) {
      return
    }
    $source = ""
    if ($script:currentCargoPackage.ContainsKey("source")) {
      $source = [string]$script:currentCargoPackage["source"]
    }
    $checksum = ""
    if ($script:currentCargoPackage.ContainsKey("checksum")) {
      $checksum = [string]$script:currentCargoPackage["checksum"]
    }

    $script:cargoPackages += [ordered]@{
      name     = [string]$script:currentCargoPackage["name"]
      version  = [string]$script:currentCargoPackage["version"]
      source   = $source
      checksum = $checksum
    }
  }

  $script:cargoPackages = @()
  $script:currentCargoPackage = $null

  foreach ($line in Get-Content -Encoding UTF8 -LiteralPath $Path) {
    if ($line -match "^\[\[package\]\]") {
      Flush-CargoPackage
      $script:currentCargoPackage = @{}
      continue
    }
    if ($null -eq $script:currentCargoPackage) {
      continue
    }
    if ($line -match '^([A-Za-z0-9_-]+)\s*=\s*"(.*)"\s*$') {
      $script:currentCargoPackage[$matches[1]] = $matches[2]
    }
  }
  Flush-CargoPackage

  $packages = @($script:cargoPackages | Sort-Object name, version, source)
  [ordered]@{
    packageCount = $packages.Count
    packages     = $packages
  }
}

$packageLockFullPath = Resolve-RequiredPath -Path $PackageLockPath -Label "前端 package-lock.json"
$cargoLockFullPath = Resolve-RequiredPath -Path $CargoLockPath -Label "Rust Cargo.lock"
$distFullPath = Resolve-RequiredPath -Path $DistPath -Label "前端生产构建目录"

$distFiles = @(
  Get-ChildItem -LiteralPath $distFullPath -Recurse -File |
    Sort-Object FullName |
    ForEach-Object {
      $relativePath = Get-RelativePathText -Root $distFullPath -Path $_.FullName
      $sha256 = Get-Sha256Text -Path $_.FullName
      [ordered]@{
        path   = $relativePath
        bytes  = $_.Length
        sha256 = $sha256
      }
    }
)

if ($distFiles.Count -eq 0) {
  throw "前端生产构建目录为空，无法生成发布清单"
}

$generatedAtUtc = ([DateTime]::UtcNow).ToString("o")
$nodeVersion = Get-ToolVersionText -Command "node"
$npmVersion = Get-ToolVersionText -Command "npm"
$rustcVersion = Get-ToolVersionText -Command "rustc"
$cargoVersion = Get-ToolVersionText -Command "cargo"
$toolVersions = [ordered]@{
  node  = $nodeVersion
  npm   = $npmVersion
  rustc = $rustcVersion
  cargo = $cargoVersion
}
$npmInfo = Read-NpmPackages -Path $packageLockFullPath
$cargoInfo = Read-CargoPackages -Path $cargoLockFullPath
$frontendDistInfo = [ordered]@{
  fileCount = $distFiles.Count
  files     = $distFiles
}

$manifest = [ordered]@{
  schemaVersion    = 1
  generatedAtUtc   = $generatedAtUtc
  repository       = $env:GITHUB_REPOSITORY
  ref              = $env:GITHUB_REF_NAME
  commitSha        = $env:GITHUB_SHA
  project          = "ARK: Survival Ascended Server Manager"
  tools            = $toolVersions
  npm              = $npmInfo
  cargo            = $cargoInfo
  frontendDist     = $frontendDistInfo
  securityControls = @(
    "发布前执行 npm audit --audit-level=high",
    "发布前执行 cargo audit --deny warnings",
    "发布前执行 cargo deny advisories/bans/sources",
    "发布前校验 Tauri CSP、生产 index.html 无内联脚本、无远程资源",
    "发布产物提供 SHA256SUMS.txt 供用户校验完整性"
  )
}

$outputFullPath = [System.IO.Path]::GetFullPath($OutputPath)
$manifest | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath $outputFullPath -Encoding UTF8
Write-Host "已生成发布 SBOM 与前端产物哈希清单：$outputFullPath"


