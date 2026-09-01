# ElectroObra

Gestión operativa y control de caja real para pymes que trabajan **por proyecto / obra**:
movimientos (ingresos, gastos y adelantos), clientes, obras/proyectos, órdenes de trabajo y certificados
de avance, facturas y pagos, empleados, asistencia, liquidaciones, reportes y exportaciones.

Nació para instalaciones eléctricas, pero el dominio es genérico: cualquier pyme que presupuesta, ejecuta por hitos, certifica avance y liquida jornales.

## ¿Para quién es? (casos de uso)

Sirve directo, sin tocar código, para:

**Instalaciones:** eléctrica, sanitaria/gas, HVAC, redes, seguridad electrónica, ascensores.
**Construcción liviana:** albañilería, pintura, durlock, herrería, carpintería, vidriería, impermeabilizaciones.
**Mantenimiento:** edilicio, facility, paisajismo, limpieza de obra.
**Servicios técnicos a campo:** talleres, mantenimiento industrial, obras civiles menores.

> `Obra` es un término genérico para *proyecto/servicio*. En `src/locales/es.json` puedes renombrarlo a `Proyecto` sin tocar lógica. Con `tipos_movimiento`/`categorias` configurables, el mismo binario cubre todos los rubros de arriba.

**No encaja sin rediseño:** retail, gastronomía, salud o educación (no trabajan por obra ni certifican avance).

> **Esta rama es una reescritura total.** La implementación anterior era C# / .NET / Avalonia.
> Esta rama huérfana (`rewrite/rust-tauri`) arranca de cero sobre **Rust + Tauri 2 + Vue 3**.
> No se arrastra código C#: sólo las reglas de negocio, documentadas de forma exhaustiva en
> [`docs/`](./docs).

Versión en inglés de este documento: [README.md](./README.md).

## Stack

| Capa | Tecnología |
| --- | --- |
| Contenedor de escritorio | Tauri 2 |
| Backend / dominio | Rust (Clean Architecture, un crate por capa) |
| Persistencia | SQLite con SeaORM + `sea-orm-migration` |
| Framework de UI | Vue 3 (`<script setup>`) + TypeScript |
| Componentes | PrimeVue 4 (widgets de datos) + Shadcn-Vue / Reka UI (primitivas) |
| Estilos | Tailwind CSS + `tailwindcss-primeui` |
| Estado | Pinia |
| i18n | vue-i18n (frontend) + catálogos JSON compartidos con el backend |
| Reportes | `printpdf`, `rust_xlsxwriter`, `docx-rs`, `csv` |
| Logging | `tracing` + `tracing-subscriber` + `tracing-appender` |
| Tests | `cargo test` (+ `rstest`, `mockall`, `wiremock`) y Vitest |

## Estructura del repositorio

```
electroobra/
├── Cargo.toml                  # workspace de Rust
├── package.json                # workspace del frontend
├── VERSION                     # única fuente de verdad de la versión
├── docs/                       # LA especificación — leer antes de escribir código
├── crates/
│   ├── eo-domain/              # entidades puras, enums, Money, errores de dominio
│   ├── eo-application/         # casos de uso, DTOs, validación, puertos (traits)
│   ├── eo-infrastructure/      # repos SeaORM, PDF/XLSX/DOCX, HTTP, backup, settings
│   ├── eo-migration/           # migraciones sea-orm-migration
│   └── eo-import-legacy/       # importador one-shot desde la base C# anterior
├── src-tauri/                  # app Tauri: comandos, estado, mapeo de errores
├── src/                        # aplicación Vue 3
└── scripts/                    # sincronización de versión, chequeos de i18n
```

## Puesta en marcha

Requisitos previos:

- Rust stable (ver `rust-toolchain.toml`)
- Node.js >= 20.11 y pnpm 9
- Requisitos de plataforma de Tauri 2 (WebView2 en Windows, `webkit2gtk` en Linux, Xcode CLT en macOS)

```bash
pnpm install
pnpm tauri:dev      # ejecuta la app de escritorio en desarrollo
pnpm tauri:build    # genera los instaladores
```

Sólo backend:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Sólo frontend:

```bash
pnpm dev            # servidor de desarrollo de Vite (sin shell nativo)
pnpm test
pnpm typecheck
pnpm lint
```

## Documentación

La especificación de [`docs/`](./docs) es normativa: define el esquema de base de datos, cada
fórmula de negocio, cada validación con su clave i18n, los contratos de comandos Tauri y los
layouts de reportes. Empezar por [`docs/00-INDICE.md`](./docs/00-INDICE.md).

El orden de implementación y el criterio de terminado de cada fase están en
[`docs/19-roadmap.md`](./docs/19-roadmap.md).

## Migración desde la aplicación anterior

Los datos existentes se trasladan con el binario de un solo uso `eo-import-legacy`, que lee la
base SQLite vieja en modo sólo lectura y escribe una nueva. Nunca modifica el origen. El
procedimiento completo, incluida la corrida en seco obligatoria y la verificación post-import,
está en [`docs/15-migracion-de-datos.md`](./docs/15-migracion-de-datos.md).

## Reglas no negociables

1. **Cero hardcoding.** Ningún texto de usuario literal, ningún número mágico, ningún color
   fuera de los tokens de diseño. Todo sale de los catálogos i18n o de la configuración.
2. **El dinero es `Money(i64)`** con escala fija de 4 decimales. Nunca `f64`.
3. **Todas las fechas y horas se almacenan en UTC**; la hora local sólo al renderizar.
4. **Cada caso de uso y cada fórmula tiene test unitario.**
5. **Los mensajes de commit van en inglés.**

Ver [AGENTS.md](./.agents/AGENTS.md) para el reglamento completo aplicado a contribuciones asistidas
por IA y [CONTRIBUTING.md](./CONTRIBUTING.md) para el flujo de trabajo.

## Licencia

Business Source License 1.1 — ver [LICENSE](./LICENSE).
Change Date: **6 de julio de 2030**. Change License: **GPL-2.0-or-later**.
Se permite el uso no productivo; el uso comercial en producción requiere licencia del Licenciante.
