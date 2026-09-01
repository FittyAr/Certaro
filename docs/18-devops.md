# 18 · DevOps: versionado, CI y distribución

Qué corre en cada push, cómo se versiona, y cómo se produce un instalador. El sistema anterior tenía
un workflow de build y test y otro manual de release; se conserva el enfoque y se extiende a las tres
plataformas.

---

## 1. Versionado

### 1.1 Fuente única

El archivo `VERSION` en la raíz contiene la versión semántica y **nada más**:

```
0.1.0
```

Todo lo demás la lee de ahí. Ninguna versión se escribe dos veces.

| Consumidor | Cómo la obtiene |
| --- | --- |
| Crates de Rust | `version.workspace = true` en cada `Cargo.toml`, y el `Cargo.toml` del workspace la tiene |
| `src-tauri/tauri.conf.json` | el campo `version` |
| `package.json` | el campo `version` |
| La interfaz | `useConfigStore().appVersion`, que viene de un comando Tauri |
| Los reportes | el pie de página (doc 12) |
| Los logs | campo `app_version` de cada entrada |

Un script `scripts/sync-version.mjs` propaga el contenido de `VERSION` a los tres archivos y falla si
alguno quedó desincronizado. Corre en CI con `--check` y en local con `--write`.

Sostener la versión en cuatro archivos a mano no funciona: siempre queda uno atrás y el número que
muestra la aplicación deja de coincidir con el del instalador.

### 1.2 Reglas de incremento

| Cambio | Incremento |
| --- | --- |
| Corrección sin cambio de esquema ni de API | patch |
| Módulo o campo nuevo, compatible | minor |
| Migración que no se puede revertir, o cambio de contrato de comandos | major |

Antes de `1.0.0` un minor puede romper compatibilidad, con la nota correspondiente en el changelog.

### 1.3 Migraciones y versión

Cada migración de `eo-migration` registra en `app_metadata` la versión de la aplicación que la aplicó.
Sirve para diagnosticar: si un usuario reporta un problema, la base dice con qué versión se creó y
cuál fue la última que la migró.

Regla dura: **una migración publicada no se edita.** Corregir una migración que ya corrió en la
máquina de alguien produce dos bases con el mismo número de esquema y contenido distinto. La
corrección va en una migración nueva.

---

## 2. Integración continua

### 2.1 `ci.yml`

Se dispara en push a cualquier rama y en todo pull request.

```yaml
name: CI

on:
  push:
    branches: ['**']
  pull_request:

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  version-sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: node scripts/sync-version.mjs --check

  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
      - run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace --locked
      - run: cargo install cargo-llvm-cov --locked
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: actions/upload-artifact@v4
        with: { name: coverage-rust, path: lcov.info }

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20', cache: 'pnpm' }
      - run: pnpm install --frozen-lockfile
      - run: pnpm lint
      - run: pnpm typecheck
      - run: pnpm i18n:check
      - run: pnpm test:coverage
      - uses: actions/upload-artifact@v4
        with: { name: coverage-frontend, path: coverage/ }

  build:
    needs: [rust, frontend]
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: windows-latest, target: x86_64-pc-windows-msvc }
          - { os: ubuntu-22.04,   target: x86_64-unknown-linux-gnu }
          - { os: macos-latest,   target: aarch64-apple-darwin }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - uses: Swatinem/rust-cache@v2
        with: { key: ${{ matrix.target }} }
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20', cache: 'pnpm' }
      - if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf
      - run: pnpm install --frozen-lockfile
      - run: pnpm tauri build --target ${{ matrix.target }} --ci
```

Puntos de diseño:

- `version-sync` corre primero y es barato. Un desajuste de versión invalida todo lo demás, así que
  no tiene sentido gastar diez minutos de build antes de detectarlo.
- `rust` y `frontend` corren en paralelo en Linux, que es el runner más rápido y barato.
- `build` sólo corre si los dos pasaron, y ahí sí en las tres plataformas. Compilar Tauri en tres
  sistemas es lo caro del pipeline; no se paga antes de saber que los tests pasan.
- `fail-fast: false` en la matriz: si Windows falla, se quiere saber igual si Linux y macOS andan.
- `--locked` en `cargo test`: el `Cargo.lock` está versionado y CI no lo puede modificar. Un build
  que resuelve dependencias distintas de las del desarrollador no verifica nada.
- `clippy` con `-D warnings`: los warnings son errores. El sistema anterior acumuló decenas de
  warnings de compilación que dejaron de leerse, y ahí se esconden los problemas reales.

### 2.2 Umbrales de cobertura

Los umbrales de doc 17 §8.2 se aplican en el job de cobertura. Si un crate baja de su piso, el job
falla. El mensaje del fallo incluye el porcentaje anterior y el nuevo.

### 2.3 `pnpm i18n:check`

Corre `scripts/check-i18n.mjs`, que implementa las verificaciones de doc 14 §5:

- `es.json` y `en.json` tienen exactamente el mismo conjunto de claves.
- Toda clave usada en `$t()` en el frontend existe en los dos.
- Toda clave de error emitida por Rust existe en los dos.
- Ningún valor de traducción contiene un formato de fecha ni un parámetro posicional.
- Los dos archivos están ordenados alfabéticamente.

Es un job aparte del lint porque el mensaje de error tiene que ser específico: qué clave falta y en
qué archivo. Un fallo genérico de lint no dice nada útil.

### 2.4 Auditoría de dependencias

```yaml
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with: { token: ${{ secrets.GITHUB_TOKEN }} }
      - uses: pnpm/action-setup@v4
      - run: pnpm audit --audit-level high
```

Corre en el push a la rama principal y en un cron semanal, no en cada push a una rama de trabajo: un
aviso de seguridad nuevo no debería bloquear un commit en progreso.

### 2.5 Dependabot

`.github/dependabot.yml` con tres ecosistemas: `cargo`, `npm` y `github-actions`. Semanal, con las
actualizaciones agrupadas por tipo para no recibir veinte pull requests.

Los grupos: dependencias de patch juntas, minor juntas, y major por separado, porque un major hay que
revisarlo a mano.

---

## 3. Release

### 3.1 `release.yml`

Se dispara con un tag `v*` o a mano.

```yaml
name: Release

on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      dry_run:
        description: 'Compila sin publicar'
        type: boolean
        default: false

jobs:
  verify:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.v.outputs.version }}
    steps:
      - uses: actions/checkout@v4
      - id: v
        run: echo "version=$(cat VERSION)" >> "$GITHUB_OUTPUT"
      - name: El tag coincide con VERSION
        if: startsWith(github.ref, 'refs/tags/')
        run: |
          test "v$(cat VERSION)" = "${GITHUB_REF#refs/tags/}"
      - run: node scripts/sync-version.mjs --check

  release:
    needs: verify
    permissions:
      contents: write
    strategy:
      fail-fast: true
      matrix:
        include:
          - { os: windows-latest, target: x86_64-pc-windows-msvc,   args: '' }
          - { os: ubuntu-22.04,   target: x86_64-unknown-linux-gnu, args: '' }
          - { os: macos-latest,   target: aarch64-apple-darwin,     args: '' }
          - { os: macos-latest,   target: x86_64-apple-darwin,      args: '' }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20', cache: 'pnpm' }
      - if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf
      - run: pnpm install --frozen-lockfile
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Certaro v__VERSION__'
          releaseBody: ${{ needs.verify.outputs.notes }}
          releaseDraft: true
          args: --target ${{ matrix.target }} ${{ matrix.args }}
```

`fail-fast: true` acá, al contrario que en CI: no se publica una release parcial. Si falta el binario
de una plataforma, no hay release.

`releaseDraft: true` porque la release se revisa antes de publicarla. Se verifica que estén los
artefactos de las cuatro plataformas y que el changelog esté bien.

### 3.2 Artefactos

| Plataforma | Formatos |
| --- | --- |
| Windows | `.msi` (WiX) y `.exe` (NSIS) |
| Linux | `.deb`, `.AppImage` |
| macOS | `.dmg` para Apple Silicon e Intel |

Nombre de archivo: `Certaro_{version}_{arch}.{ext}`, que es lo que produce Tauri por defecto.

### 3.3 Firma

| Plataforma | Estado |
| --- | --- |
| Windows | sin firmar en la fase inicial. El instalador muestra el aviso de SmartScreen; se documenta en el README. |
| macOS | sin firmar ni notarizar. El usuario abre con click derecho la primera vez; se documenta. |
| Linux | no aplica |

Firmar cuesta dinero y no es un requisito de un sistema de uso interno. La decisión se revisa si el
sistema se distribuye fuera. Los secretos de firma quedan previstos en el workflow como variables
opcionales, así que activarla más adelante no requiere reescribirlo.

### 3.4 Actualizaciones automáticas

**No** en la fase inicial. El plugin `tauri-plugin-updater` requiere firmar los artefactos y alojar un
manifiesto; se agrega cuando la firma esté resuelta.

Mientras tanto la aplicación consulta la API de releases de GitHub en el arranque, compara la versión
y muestra un aviso no bloqueante con el enlace de descarga si hay una más nueva. La consulta tiene
timeout corto y degrada en silencio, como las demás llamadas externas (doc 13 §2.4).

### 3.5 Changelog

`CHANGELOG.md` en formato Keep a Changelog, escrito a mano. No se genera de los commits: un changelog
generado repite el mensaje técnico del commit y no le sirve al usuario.

Cada entrada dice **qué cambió para quien usa el sistema**, no qué archivo se tocó.

Las entradas que involucran una migración de esquema llevan una nota explícita, porque el usuario
tiene que hacer un backup antes de actualizar.

---

## 4. Desarrollo local

### 4.1 Requisitos

| Herramienta | Versión |
| --- | --- |
| Rust | estable, según `rust-toolchain.toml` |
| Node.js | ≥ 20.11 |
| pnpm | 9.x |
| Tauri: dependencias del sistema | según la plataforma, en el README |

`rust-toolchain.toml` fija el canal y los componentes, así que `rustup` instala lo correcto solo al
entrar al directorio.

### 4.2 Comandos

| Comando | Qué hace |
| --- | --- |
| `pnpm install` | dependencias del frontend |
| `pnpm tauri:dev` | aplicación completa con recarga en caliente |
| `pnpm dev` | sólo el frontend en el navegador; los comandos Tauri fallan |
| `cargo test --workspace` | tests del backend |
| `pnpm test` | tests del frontend |
| `cargo clippy --workspace --all-targets` | lint del backend |
| `pnpm lint` | lint del frontend |
| `pnpm tauri:build` | instalador local |
| `node scripts/sync-version.mjs --write` | propaga `VERSION` |

`pnpm dev` sirve para trabajar el layout y los estilos sin esperar la compilación de Rust, con la
salvedad de que toda llamada al backend falla. Un `dev-mock` que devuelva datos de ejemplo queda
fuera de alcance: mantener dos implementaciones del backend cuesta más de lo que ahorra.

### 4.3 Hooks de git

`.githooks/pre-commit`, activado con `git config core.hooksPath .githooks`:

```bash
#!/bin/sh
set -e
cargo fmt --all -- --check
pnpm lint
node scripts/sync-version.mjs --check
```

Sólo formato y lint: son rápidos. Los tests **no** van en el hook. Un pre-commit que tarda un minuto
termina esquivado con `--no-verify`, y entonces no sirve para nada.

---

## 5. Logging y diagnóstico

Definidos en [`02-arquitectura.md`](./02-arquitectura.md) §5 y
[`14-configuracion-e-i18n.md`](./14-configuracion-e-i18n.md) §2.10. Lo relevante para operación:

| Aspecto | Valor |
| --- | --- |
| Ubicación | `{data_dir}/logs/Certaro-{fecha}.log` |
| Formato | JSON por línea |
| Rotación | diaria |
| Retención | 30 días, configurable |
| Nivel por defecto | `info` en release, `debug` en desarrollo |
| `trace_id` | uno por operación, presente en toda entrada y en el `ApiError` |

El `trace_id` es lo que hace utilizable el log: el usuario reporta el identificador que vio en el
mensaje de error y esa línea del log se encuentra de una.

Un comando `abrir_carpeta_de_logs` en el menú de ayuda evita tener que explicarle a alguien cómo
llegar a `%LOCALAPPDATA%`.

---

## 6. Estructura de `.github`

```
.github/
├── workflows/
│   ├── ci.yml
│   ├── release.yml
│   └── audit.yml
├── dependabot.yml
├── ISSUE_TEMPLATE/
│   ├── bug.yml            # incluye campo obligatorio de versión y trace_id
│   └── feature.yml
└── pull_request_template.md
```

El template de bug pide la versión y el `trace_id` como campos obligatorios. Sin esos dos datos un
reporte no es accionable.

El template de pull request es el checklist de doc 19: tests, i18n, sin colores literales, sin
hardcoding, documentación actualizada.
