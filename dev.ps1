# dev.ps1
# Asistente de desarrollo para ElectroObra.
#
# Uso:
#   .\dev.ps1 setup              # Instala dependencias y configura el entorno
#   .\dev.ps1 dev / run          # Elige web o desktop (pregunta)
#   .\dev.ps1 dev:web / web      # Solo web (Vite en http://localhost:1420, mock en localStorage)
#   .\dev.ps1 dev:desktop / desktop # Solo desktop (Tauri + Vite, SQLite en %LOCALAPPDATA%\ElectroObra)
#   .\dev.ps1 build          # Build de release
#   .\dev.ps1 test           # Todos los tests (Rust + frontend)
#   .\dev.ps1 test-rust      # Solo tests de Rust
#   .\dev.ps1 test-fe        # Solo tests del frontend
#   .\dev.ps1 lint           # Lint completo (clippy + eslint)
#   .\dev.ps1 lint-fix       # Lint con auto-fix
#   .\dev.ps1 format         # Formatea el código
#   .\dev.ps1 check          # Verificaciones pre-commit (lint + tests + i18n)
#   .\dev.ps1 clean          # Limpia build artifacts
#   .\dev.ps1 clean-all      # Limpia todo incluyendo node_modules
#   .\dev.ps1 i18n           # Verifica sincronización de i18n
#   .\dev.ps1 i18n-fix       # Ordena y sincroniza los locales
#   .\dev.ps1 version        # Muestra la versión actual
#   .\dev.ps1 version-sync   # Sincroniza VERSION a todos los manifiestos
#   .\dev.ps1 installer      # Construye el instalador Windows
#   .\dev.ps1 import         # Ejecuta eo-import-legacy (requiere argumentos)
#   .\dev.ps1 help           # Muestra esta ayuda

param(
    [Parameter(Position = 0)]
    [ValidateSet("setup", "dev", "run", "web", "desktop", "dev:web", "dev:desktop", "build", "test", "test-rust", "test-fe", "lint", "lint-fix",
                 "format", "check", "clean", "clean-all", "i18n", "i18n-fix", "version",
                 "version-sync", "installer", "import", "help")]
    [string]$Command = "help",

    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$ErrorActionPreference = "Stop"

# ── Utilidades ──────────────────────────────────────────────────────────────

function Write-Step($msg) { Write-Host "`n>>> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "    $msg" -ForegroundColor Green }
function Write-Err($msg)  { Write-Host "    $msg" -ForegroundColor Red }

function Invoke-OrFail($cmd) {
    Write-Host "    $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    if ($LASTEXITCODE -ne 0) {
        Write-Err "Fallo con codigo de salida $LASTEXITCODE"
        exit $LASTEXITCODE
    }
}

function Get-Version {
    if (Test-Path ".\VERSION") {
        return (Get-Content ".\VERSION").Trim()
    }
    return "desconocida"
}

# ── Comandos ────────────────────────────────────────────────────────────────

switch ($Command) {
    "help" {
        Write-Host ""
        Write-Host "ElectroObra - Asistente de desarrollo" -ForegroundColor White
        Write-Host ""
        Write-Host "Comandos:" -ForegroundColor White
        Write-Host "  setup          Instala dependencias y configura el entorno" -ForegroundColor Gray
        Write-Host "  dev, run       Elige desktop o web (pregunta)" -ForegroundColor Gray
        Write-Host "  dev:web, web     Solo web (http://localhost:1420, datos en localStorage)" -ForegroundColor Gray
        Write-Host "  dev:desktop, desktop Solo desktop (Tauri, datos en %LOCALAPPDATA%\\ElectroObra\\electroobra.db)" -ForegroundColor Gray
        Write-Host "  build          Build de release" -ForegroundColor Gray
        Write-Host "  test           Todos los tests (Rust + frontend)" -ForegroundColor Gray
        Write-Host "  test-rust      Solo tests de Rust" -ForegroundColor Gray
        Write-Host "  test-fe        Solo tests del frontend" -ForegroundColor Gray
        Write-Host "  lint           Lint completo (clippy + eslint)" -ForegroundColor Gray
        Write-Host "  lint-fix       Lint con auto-fix" -ForegroundColor Gray
        Write-Host "  format         Formatea el codigo" -ForegroundColor Gray
        Write-Host "  check          Verificaciones pre-commit (lint + tests + i18n)" -ForegroundColor Gray
        Write-Host "  clean          Limpia build artifacts" -ForegroundColor Gray
        Write-Host "  clean-all      Limpia todo incluyendo node_modules" -ForegroundColor Gray
        Write-Host "  i18n           Verifica sincronizacion de i18n" -ForegroundColor Gray
        Write-Host "  i18n-fix       Ordena y sincroniza los locales" -ForegroundColor Gray
        Write-Host "  version        Muestra la version actual" -ForegroundColor Gray
        Write-Host "  version-sync   Sincroniza VERSION a todos los manifiestos" -ForegroundColor Gray
        Write-Host "  installer      Construye el instalador Windows" -ForegroundColor Gray
        Write-Host "  import         Ejecuta eo-import-legacy (requiere --source y --target)" -ForegroundColor Gray
        Write-Host "  help           Muestra esta ayuda" -ForegroundColor Gray
        Write-Host ""
    }

    "setup" {
        Write-Step "Instalando dependencias del frontend"
        Invoke-OrFail "pnpm install"

        Write-Step "Verificando herramientas de Rust"
        Invoke-OrFail "rustc --version"
        Invoke-OrFail "cargo --version"

        Write-Step "Verificando sincronizacion de version"
        Invoke-OrFail "node scripts/sync-version.mjs --check"

        Write-Step "Verificando i18n"
        Invoke-OrFail "pnpm i18n:check"

        Write-Host ""
        Write-Ok "Entorno configurado. Ejecuta '.\dev.ps1 dev' o '.\dev.ps1 run' para arrancar."
    }

    { $_ -in @("dev", "run") } {
        $mode = ""
        if ($Args.Count -gt 0) { $mode = $Args[0].ToLower() }
        if ($mode -notin @("web", "desktop", "dev:web", "dev:desktop")) {
            Write-Host ""
            Write-Host "Elige modo:" -ForegroundColor White
            Write-Host "  [1] desktop  Tauri + Vite (ventana nativa, SQLite real)" -ForegroundColor Gray
            Write-Host "  [2] web      Solo Vite (navegador en http://localhost:1420, mock localStorage)" -ForegroundColor Gray
            $choice = Read-Host "Opción [1/2] (default 1)"
            if ($choice -eq "2") { $mode = "web" } else { $mode = "desktop" }
        }
        if ($mode -in @("web", "dev:web")) {
            Write-Step "Arrancando solo web (Vite)"
            Write-Host "    Abre http://localhost:1420 en el navegador." -ForegroundColor Gray
            Write-Host "    Datos: mock en localStorage (electroobra_mock_db_v2), separado del SQLite de desktop." -ForegroundColor Yellow
            Write-Host "    Para ver los mismos datos que en desktop: en desktop Exportar JSON y en web Importar JSON (Configuración > Sistema)." -ForegroundColor Gray
            Write-Host ""
            pnpm dev --host 0.0.0.0
        } else {
            Write-Step "Arrancando desktop (Tauri + Vite)"
            Write-Host "    Ventana nativa + http://localhost:1420 con hot reload." -ForegroundColor Gray
            Write-Host "    Datos: SQLite en %LOCALAPPDATA%\ElectroObra\electroobra.db (real, no mock)." -ForegroundColor Gray
            Write-Host ""
            pnpm tauri dev
        }
    }

    { $_ -in @("web", "dev:web") } {
        Write-Step "Arrancando solo web (Vite)"
        pnpm dev --host 0.0.0.0
    }

    { $_ -in @("desktop", "dev:desktop") } {
        Write-Step "Arrancando desktop (Tauri + Vite)"
        pnpm tauri dev
    }

    "build" {
        Write-Step "Build de release"
        Invoke-OrFail "pnpm tauri build"
        Write-Ok "Build completado. Artefactos en src-tauri\target\release\bundle\"
    }

    "test" {
        Write-Step "Tests de Rust"
        Invoke-OrFail "cargo test --workspace"

        Write-Step "Tests del frontend"
        Invoke-OrFail "pnpm test"

        Write-Host ""
        Write-Ok "Todos los tests pasaron."
    }

    "test-rust" {
        Write-Step "Tests de Rust"
        Invoke-OrFail "cargo test --workspace"
    }

    "test-fe" {
        Write-Step "Tests del frontend"
        Invoke-OrFail "pnpm test"
    }

    "lint" {
        Write-Step "Clippy (warnings como errores)"
        Invoke-OrFail "cargo clippy --workspace --all-targets -- -D warnings"

        Write-Step "ESLint"
        Invoke-OrFail "pnpm lint"

        Write-Host ""
        Write-Ok "Lint limpio."
    }

    "lint-fix" {
        Write-Step "Clippy fix"
        Invoke-OrFail "cargo clippy --fix --workspace --all-targets --allow-dirty"

        Write-Step "ESLint fix"
        Invoke-OrFail "pnpm lint:fix"

        Write-Step "Formato"
        Invoke-OrFail "cargo fmt --all"
        Invoke-OrFail "pnpm format"

        Write-Host ""
        Write-Ok "Auto-fix completado."
    }

    "format" {
        Write-Step "Formateando Rust"
        Invoke-OrFail "cargo fmt --all"

        Write-Step "Formateando frontend"
        Invoke-OrFail "pnpm format"

        Write-Ok "Codigo formateado."
    }

    "check" {
        Write-Step "Sincronizacion de version"
        Invoke-OrFail "node scripts/sync-version.mjs --check"

        Write-Step "i18n"
        Invoke-OrFail "pnpm i18n:check"

        Write-Step "Formato Rust"
        Invoke-OrFail "cargo fmt --all -- --check"

        Write-Step "Clippy"
        Invoke-OrFail "cargo clippy --workspace --all-targets -- -D warnings"

        Write-Step "ESLint"
        Invoke-OrFail "pnpm lint"

        Write-Step "TypeScript"
        Invoke-OrFail "pnpm typecheck"

        Write-Step "Tests de Rust"
        Invoke-OrFail "cargo test --workspace"

        Write-Step "Tests del frontend"
        Invoke-OrFail "pnpm test"

        Write-Host ""
        Write-Ok "Todas las verificaciones pasaron. Listo para commit."
    }

    "clean" {
        Write-Step "Limpiando build artifacts"
        $dirs = @("target\release", "target\debug\deps", "dist")
        foreach ($dir in $dirs) {
            if (Test-Path $dir) {
                Write-Host "    Eliminando $dir..." -ForegroundColor Gray
                Remove-Item -Path $dir -Recurse -Force
            }
        }
        Write-Ok "Limpieza completada."
    }

    "clean-all" {
        Write-Step "Limpieza completa"
        $dirs = @("target", "dist", "node_modules")
        foreach ($dir in $dirs) {
            if (Test-Path $dir) {
                Write-Host "    Eliminando $dir..." -ForegroundColor Gray
                Remove-Item -Path $dir -Recurse -Force
            }
        }
        Write-Ok "Limpieza completa. Ejecuta '.\dev.ps1 setup' para reinstalar."
    }

    "i18n" {
        Write-Step "Verificando i18n"
        Invoke-OrFail "pnpm i18n:check"
        Write-Ok "Locales sincronizados."
    }

    "i18n-fix" {
        Write-Step "Ordenando locales"
        Invoke-OrFail "node scripts/sort-locales.mjs"
        Write-Step "Verificando"
        Invoke-OrFail "pnpm i18n:check"
        Write-Ok "Locales ordenados y verificados."
    }

    "version" {
        $v = Get-Version
        Write-Host "ElectroObra v$v" -ForegroundColor White
    }

    "version-sync" {
        Write-Step "Sincronizando version"
        Invoke-OrFail "node scripts/sync-version.mjs --write"
        Write-Ok "Version sincronizada a todos los manifiestos."
    }

    "installer" {
        Write-Step "Construyendo instalador"
        & .\build-installer.ps1 @Args
    }

    "import" {
        if ($Args.Count -eq 0) {
            Write-Host "Uso: .\dev.ps1 import --source <ruta_legacy.db> --target <ruta_nueva.db> [opciones]" -ForegroundColor Yellow
            Write-Host ""
            Write-Host "Opciones:" -ForegroundColor White
            Write-Host "  --assume-scaled      La base ya tiene la migracion RescaleMonetaryValues" -ForegroundColor Gray
            Write-Host "  --assume-unscaled    La base NO tiene esa migracion" -ForegroundColor Gray
            Write-Host "  --dry-run            Ejecuta sin escribir (solo genera el reporte)" -ForegroundColor Gray
            Write-Host "  --allow-orphans      Convierte FK huerfanas en NULL" -ForegroundColor Gray
            exit 1
        }
        $argStr = $Args -join " "
        Invoke-OrFail "cargo run -p eo-import-legacy -- $argStr"
    }

    default {
        Write-Err "Comando desconocido: $Command"
        Write-Host "Ejecuta '.\dev.ps1 help' para ver los comandos disponibles." -ForegroundColor Yellow
        exit 1
    }
}
