# ElectroObra

Operational management and real cash-flow control for SMEs working **by project / site**:
movements (income, expenses and advances), clients, projects/sites, work orders and certificates
of progress, invoices and payments, employees, attendance, settlements, reports and exports.

Born for electrical installations, but the domain is generic: any SME that quotes, executes by milestones, certifies progress and settles daily wages.

## Who is it for? (use cases)

Works out-of-the-box for:

**Installations:** electrical, plumbing/gas, HVAC, networking, electronic security, elevators.
**Light construction:** masonry, painting, drywall, metalwork, carpentry, glazing, waterproofing.
**Maintenance:** building, facility, landscaping, site cleaning.
**Field services:** workshops, industrial maintenance, small civil works.

> \Site\/\Obra\ is a generic term for *project/service*. You can rename it to \Project\ in \src/locales/en.json\ without touching logic. With configurable \movement types\/\categories\, the same binary covers all the above.

**Not a fit without redesign:** retail, hospitality, healthcare or education (no site-based progress certification).

> **This branch is a full rewrite.** The previous implementation was C# / .NET / Avalonia.
> This orphan branch (`rewrite/rust-tauri`) starts from zero on **Rust + Tauri 2 + Vue 3**.
> No C# code is carried over — only the business rules, which are documented exhaustively
> in [`docs/`](./docs).

Spanish version of this document: [README.es.md](./README.es.md).

## Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri 2 |
| Backend / domain | Rust (Clean Architecture, one crate per layer) |
| Persistence | SQLite via SeaORM + `sea-orm-migration` |
| Frontend framework | Vue 3 (`<script setup>`) + TypeScript |
| UI components | PrimeVue 4 (data-heavy widgets) + Shadcn-Vue / Reka UI (primitives) |
| Styling | Tailwind CSS + `tailwindcss-primeui` |
| State | Pinia |
| i18n | vue-i18n (frontend) + JSON catalogs shared with the backend |
| Reports | `printpdf`, `rust_xlsxwriter`, `docx-rs`, `csv` |
| Logging | `tracing` + `tracing-subscriber` + `tracing-appender` |
| Tests | `cargo test` (+ `rstest`, `mockall`, `wiremock`) and Vitest |

## Repository layout

```
electroobra/
├── Cargo.toml                  # Rust workspace
├── package.json                # frontend workspace
├── VERSION                     # single source of truth for the version number
├── docs/                       # THE specification — read this before writing code
├── crates/
│   ├── eo-domain/              # pure entities, enums, Money, domain errors
│   ├── eo-application/         # use cases, DTOs, validation, ports (traits)
│   ├── eo-infrastructure/      # SeaORM repos, PDF/XLSX/DOCX, HTTP, backup, settings
│   ├── eo-migration/           # sea-orm-migration definitions
│   └── eo-import-legacy/       # one-shot importer from the legacy C# database
├── src-tauri/                  # Tauri app: commands, state, error mapping
├── src/                        # Vue 3 application
└── scripts/                    # version sync, i18n checks
```

## Getting started

Prerequisites:

- Rust stable (see `rust-toolchain.toml`)
- Node.js >= 20.11 and pnpm 9
- Tauri 2 platform prerequisites (WebView2 on Windows, `webkit2gtk` on Linux, Xcode CLT on macOS)

```bash
pnpm install
pnpm tauri:dev      # run the desktop app in development
pnpm tauri:build    # produce installers
```

Backend only:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Frontend only:

```bash
pnpm dev            # Vite dev server (no native shell)
pnpm test
pnpm typecheck
pnpm lint
```

## Documentation

The specification in [`docs/`](./docs) is normative: it defines the database schema, every
business formula, every validation rule with its i18n key, the Tauri command contracts and
the report layouts. Start at [`docs/00-INDICE.md`](./docs/00-INDICE.md).

The documents are written in Spanish, because that is the language of the business domain
(*obra*, *trabajo*, *certificado*, *liquidación*, *adelanto*) and translating those terms
would make the specification harder to match against the client's own vocabulary.

Implementation order and the definition of done for each phase are in
[`docs/19-roadmap.md`](./docs/19-roadmap.md).

## Migrating from the legacy application

Existing data is moved with the one-shot `eo-import-legacy` binary, which reads the old SQLite
database read-only and writes a fresh one. It never modifies the source. The full procedure,
including the mandatory dry run and post-import verification, is in
[`docs/15-migracion-de-datos.md`](./docs/15-migracion-de-datos.md).

## Non-negotiable rules

1. **No hardcoding.** No literal user-facing strings, no magic numbers, no colours outside the
   design tokens. Everything comes from i18n catalogs or configuration.
2. **Money is `Money(i64)`** with a fixed scale of 4 decimal places. Never `f64`.
3. **All timestamps are UTC** in storage and in the domain. Local time only at render time.
4. **Every use case and every formula has a unit test.**
5. **Commit messages are in English.**

See [AGENTS.md](./.agents/AGENTS.md) for the full ruleset applied to AI-assisted contributions and
[CONTRIBUTING.md](./CONTRIBUTING.md) for the workflow.

## License

Business Source License 1.1 — see [LICENSE](./LICENSE).
Change Date: **July 6, 2030**. Change License: **GPL-2.0-or-later**.
Non-production use is granted; commercial production use requires a license from the Licensor.
