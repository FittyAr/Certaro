# build-installer.ps1
# Construye el instalador de ElectroObra para Windows usando Tauri.
#
# Uso:
#   .\build-installer.ps1              # Build release normal
#   .\build-installer.ps1 -Clean       # Limpia y rebuild completo
#   .\build-installer.ps1 -SkipTests   # Salta tests (útil para CI)
#
# Requisitos:
#   - Rust toolchain estable
#   - Node.js >= 20.11
#   - pnpm 9.x
#   - Dependencias de Tauri para Windows (WebView2, etc.)

param(
    [switch]$Clean,
    [switch]$SkipTests,
    [switch]$SkipVersionSync
)

$ErrorActionPreference = "Stop"

# ── Utilidades ──────────────────────────────────────────────────────────────

function Write-Step($msg) { Write-Host "`n>>> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "    $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "    $msg" -ForegroundColor Yellow }
function Write-Err($msg)  { Write-Host "    $msg" -ForegroundColor Red }

function Invoke-OrFail($cmd) {
    Write-Host "    $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    if ($LASTEXITCODE -ne 0) {
        Write-Err "Fallo con código de salida $LASTEXITCODE"
        exit $LASTEXITCODE
    }
}

# ── Versión ─────────────────────────────────────────────────────────────────

$versionFile = ".\VERSION"
if (-not (Test-Path $versionFile)) {
    Write-Err "Archivo VERSION no encontrado."
    exit 1
}
$version = (Get-Content $versionFile).Trim()
Write-Host "ElectroObra v$version" -ForegroundColor White
Write-Host ("=" * 50) -ForegroundColor DarkGray

# ── Verificaciones previas ──────────────────────────────────────────────────

Write-Step "Verificando requisitos"

# Rust
$rustVersion = rustc --version 2>$null
if (-not $rustVersion) {
    Write-Err "Rust no encontrado. Instalá con: winget install Rustlang.Rustup"
    exit 1
}
Write-Ok "Rust: $rustVersion"

# Node
$nodeVersion = node --version 2>$null
if (-not $nodeVersion) {
    Write-Err "Node.js no encontrado. Instalá con: winget install OpenJS.NodeJS.LTS"
    exit 1
}
Write-Ok "Node: $nodeVersion"

# pnpm
$pnpmVersion = pnpm --version 2>$null
if (-not $pnpmVersion) {
    Write-Err "pnpm no encontrado. Instalá con: npm install -g pnpm"
    exit 1
}
Write-Ok "pnpm: v$pnpmVersion"

# ── Sincronización de versión ───────────────────────────────────────────────

if (-not $SkipVersionSync) {
    Write-Step "Sincronizando versión en manifiestos"
    Invoke-OrFail "node scripts/sync-version.mjs --check"
    Write-Ok "VERSION, Cargo.toml, package.json y tauri.conf.json coinciden"
}

# ── Limpieza ────────────────────────────────────────────────────────────────

if ($Clean) {
    Write-Step "Limpieza completa"
    $targets = @("target", "dist", "node_modules\.vite")
    foreach ($dir in $targets) {
        if (Test-Path $dir) {
            Write-Host "    Eliminando $dir..." -ForegroundColor Gray
            Remove-Item -Path $dir -Recurse -Force
        }
    }
    Write-Ok "Limpieza completada"
}

# ── Dependencias ────────────────────────────────────────────────────────────

Write-Step "Instalando dependencias del frontend"
Invoke-OrFail "pnpm install --frozen-lockfile"

# ── Tests ───────────────────────────────────────────────────────────────────

if (-not $SkipTests) {
    Write-Step "Ejecutando tests del backend"
    Invoke-OrFail "cargo test --workspace"

    Write-Step "Ejecutando tests del frontend"
    Invoke-OrFail "pnpm test"

    Write-Step "Verificando i18n"
    Invoke-OrFail "pnpm i18n:check"

    Write-Step "Lint del frontend"
    Invoke-OrFail "pnpm lint"

    Write-Step "Clippy (warnings como errores)"
    Invoke-OrFail "cargo clippy --workspace --all-targets -- -D warnings"
}

# ── Build ───────────────────────────────────────────────────────────────────

Write-Step "Construyendo instalador con Tauri"
Invoke-OrFail "pnpm tauri build"

# ── Resultado ───────────────────────────────────────────────────────────────

Write-Step "Build completado"

$outputDir = "src-tauri\target\release\bundle"
if (Test-Path $outputDir) {
    Write-Host "`nArtefactos generados:" -ForegroundColor White
    Get-ChildItem -Path $outputDir -Recurse -File | ForEach-Object {
        $size = if ($_.Length -gt 1MB) { "$([math]::Round($_.Length / 1MB, 1)) MB" } else { "$([math]::Round($_.Length / 1KB, 1)) KB" }
        Write-Host "    $($_.FullName) ($size)" -ForegroundColor Gray
    }
} else {
    Write-Warn "No se encontró el directorio de salida en $outputDir"
}

Write-Host "`n" -NoNewline
Write-Ok "ElectroObra v$version lista para distribuir."
