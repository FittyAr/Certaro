# Contributing

Thanks for working on Certaro. This document describes the workflow; the coding rules live
in [AGENTS.md](./.agents/AGENTS.md) and the functional specification in [`docs/`](./docs).

## Prerequisites

- Rust stable (`rust-toolchain.toml` pins the channel and components)
- Node.js >= 20.11, pnpm 9 (`corepack enable pnpm`)
- Tauri 2 platform prerequisites:
  - **Windows**: Microsoft Edge WebView2 runtime, MSVC build tools
  - **Linux**: `webkit2gtk-4.1`, `libayatana-appindicator3`, `librsvg2`, `patchelf`
  - **macOS**: Xcode Command Line Tools

## Setup

```bash
git clone <repo> && cd Certaro
pnpm install
cargo build --workspace
pnpm tauri:dev
```

## Branching

| Branch | Purpose |
| --- | --- |
| `rewrite/rust-tauri` | integration branch for the rewrite |
| `feat/<scope>-<short-desc>` | new functionality |
| `fix/<scope>-<short-desc>` | bug fixes |
| `docs/<scope>` | specification changes only |

`<scope>` matches a module name from the docs: `movimientos`, `facturas`, `liquidaciones`,
`certificados`, `asistencia`, `reportes`, `migration`, `frontend`, `ci`…

## Commit messages

English, imperative, conventional prefix:

```
feat(liquidaciones): add attendance-based settlement suggestion
fix(facturas): recompute outstanding balance after payment deletion
docs(datos): document ON DELETE behaviour for cliente_contactos
test(certificados): cover accumulated percentage boundary at 100
```

## Definition of done

A change is complete when **all** of the following hold:

- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `pnpm typecheck`, `pnpm lint` and `pnpm test` pass
- [ ] New/changed formulas have unit tests asserting exact expected values
- [ ] New/changed validations have tests asserting the returned i18n key
- [ ] No new user-facing literal strings (everything through i18n)
- [ ] `es` and `en` locale files are both updated and stay key-synchronized
- [ ] Migrations are additive and reversible (`up` **and** `down` implemented)
- [ ] `node scripts/sync-version.mjs --check` passes
- [ ] The frontend architecture tests in [`docs/16-frontend.md`](./docs/16-frontend.md) §8 pass
- [ ] Docs updated when behaviour or structure changed

The per-phase version of this checklist, with the extra items that apply when closing a roadmap
phase, is in [`docs/19-roadmap.md`](./docs/19-roadmap.md) §13.

## Adding a database migration

1. Add a new file under `crates/eo-migration/src/` named `mYYYYMMDD_HHMMSS_<description>.rs`.
2. Register it in the migrator list.
3. Implement both `up` and `down`.
4. Update [`docs/03-modelo-de-datos.md`](./docs/03-modelo-de-datos.md) in the same commit.
5. Never edit a migration that has already been released; add a new one.

## Adding an i18n key

1. Add the key to `src/locales/es.json` (canonical) **and** `src/locales/en.json`.
2. Keep the nesting and alphabetical order within each object.
3. Run `pnpm i18n:check` — it fails if the two catalogs diverge.

## Adding a Tauri command

1. Define the use case in `eo-application` first, with tests.
2. Add the command in `src-tauri/src/commands/`, keeping it a thin adapter.
3. Register it in the `invoke_handler` and in the capability file if needed.
4. Mirror the request/response types in `src/api/` and document them in
   [`docs/11-contratos-tauri.md`](./docs/11-contratos-tauri.md).

## Reporting a spec gap

If the specification is ambiguous, missing, or contradicts the legacy behaviour, open an issue
titled `spec: <document> — <what is unclear>` instead of guessing. Financial formulas are never
to be inferred.

## License

By contributing you agree that your contributions are licensed under the
Business Source License 1.1 in [LICENSE](./LICENSE).
