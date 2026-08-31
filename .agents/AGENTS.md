# AGENTS.md — Rules for the implementing agent

You are implementing **ElectroObra** from scratch on Rust + Tauri 2 + Vue 3. The full
specification lives in [`docs/`](../docs) and is **normative**: if the code and the docs
disagree, the docs win. If the docs are silent or contradictory, stop and ask — do not invent
business rules.

## 0. Before you write a single line

1. Read [`docs/00-INDICE.md`](../docs/00-INDICE.md) to find which document covers your task.
2. Read that document **completely**. Each document is self-contained on purpose.
3. Check [`docs/19-roadmap.md`](../docs/19-roadmap.md): phases are ordered by dependency.
   Do not start phase *N+1* until phase *N* meets its "done" criteria.
4. Consult and apply the specialized skills available in `.agents/skills/`:
   - **`rust-skills`** (`.agents/skills/rust-skills/`): Comprehensive Rust coding guidelines across 26 categories (ownership, error handling, async, numeric safety, conversions, testing, API design).
   - **`tauri-v2`** (`.agents/skills/tauri-v2/`): Tauri v2 configuration, IPC (commands/events), capabilities, permissions, and desktop integration.
   - **`vue-best-practices`** (`.agents/skills/vue-best-practices/`): Vue 3 + TypeScript standards, Composition API with `<script setup>`, Pinia stores, Vue Router, and component structure.

## 1. Zero hardcoding — absolute

| Never hardcode | Where it belongs |
| --- | --- |
| User-facing text | i18n catalogs (`src/locales/*.json`), key referenced via `t('...')` |
| Colours, spacing, radii, shadows | Tailwind theme tokens / CSS variables |
| Connection strings, paths, directories | configuration (`../docs/14-configuracion-e-i18n.md`) |
| Company name, contractor name, logo | configuration — the legacy app hardcoded these |
| Tax rates, thresholds, page sizes, timeouts | configuration or a `constants` module |
| Fixed GUIDs of system rows | `eo-domain` constants module, single definition |

A literal string in a `.vue` template or a colour like `#252525` anywhere is a defect.

## 2. Money and dates

- Money is the newtype `Money(i64)` from `eo-domain`, scale **4** (value × 10 000).
  Never `f64`, never `f32`. All arithmetic goes through `Money` methods with explicit rounding.
- Percentages, multipliers and worked-day counts use `Decimal4(i64)` with the same scale.
- Every instant is `DateTime<Utc>`. Storage is UTC ISO-8601. Local time appears **only** in
  the presentation layer.
- See [`docs/04-dinero-fechas-y-tipos.md`](../docs/04-dinero-fechas-y-tipos.md).

## 3. Architecture boundaries

```
eo-domain  ←  eo-application  ←  eo-infrastructure
                    ↑                    ↑
                    └──── src-tauri ─────┘   →   src/ (Vue)
```

- `eo-domain` depends on nothing but `chrono`, `uuid`, `serde`, `thiserror`. **No SeaORM, no I/O.**
- `eo-application` declares **ports** as traits and contains use cases. It must not reference
  SeaORM, `reqwest`, the filesystem or Tauri.
- `eo-infrastructure` implements the ports. It is the only crate that knows about SQL, HTTP,
  PDF generation and the filesystem.
- `src-tauri` commands are a **thin** layer: deserialize input → call a use case → map
  `Result<T, AppError>` to a serializable payload. No business logic in commands.
- The Vue layer never computes business values that the backend can compute. It formats.

## 4. Errors

- No `panic!`, no `unwrap()`, no `expect()` in production paths. `unwrap` is allowed in tests only.
- Domain and application functions return `Result<T, AppError>`.
- `AppError` variants carry a stable machine-readable `code` and an **i18n key**, never a
  pre-translated message. Translation happens in the frontend.

## 5. Tests are part of the definition of done

- Every formula in [`docs/06-casos-de-uso-y-formulas.md`](../docs/06-casos-de-uso-y-formulas.md)
  gets at least one unit test with the exact expected value, including boundary cases.
- Every validation rule in [`docs/07-validaciones.md`](../docs/07-validaciones.md) gets a passing
  and a failing case, asserting the **i18n key** returned.
- Repositories are tested against an in-memory SQLite database with migrations applied.
- HTTP adapters are tested against `wiremock`, including the timeout and degradation paths.
- Vue components and stores are tested with Vitest.
- The architecture tests of [`docs/16-frontend.md`](../docs/16-frontend.md) §8 must keep passing.
  They are what prevent literal colours, untranslated text and money arithmetic from creeping
  back into the frontend.
- The mandatory test list of [`docs/17-testing.md`](../docs/17-testing.md) §3.4 outranks the
  coverage percentage. Hitting the threshold without those tests is not done.
- A pull request that adds a use case without tests is incomplete.

## 6. Style and tooling

- Follow the guidelines provided by the installed skills in `.agents/skills/`:
  - `rust-skills` for all Rust crates (`eo-domain`, `eo-application`, `eo-infrastructure`, `src-tauri`).
  - `tauri-v2` for Tauri configuration, capability files, and backend commands.
  - `vue-best-practices` for the frontend Vue 3 application (`src/`).
- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` must pass.
- `pnpm lint`, `pnpm typecheck` and `pnpm test` must pass.
- Vue components use `<script setup lang="ts">`. Props and emits are typed.
- Naming: Rust `snake_case` / `PascalCase` as idiomatic; TypeScript `camelCase`;
  Vue components `PascalCase.vue`; i18n keys `dot.case` in Spanish domain wording.
- Database identifiers are `snake_case` exactly as written in
  [`docs/03-modelo-de-datos.md`](../docs/03-modelo-de-datos.md). Do not rename columns.

## 7. Comments and documentation

- Comments explain **why**, never **what**. No narration of the obvious.
- Public items in `eo-domain` and `eo-application` carry `///` doc comments.
- If you change the structure or the setup steps, update `README.md` **and** `README.es.md`.
- If you discover a business rule the docs missed, add it to the relevant `docs/` file in the
  same change.

## 8. Git

- **Commit messages in English**, imperative mood, conventional-commit prefix:
  `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `ci:`.
- One logical change per commit. Do not mix a schema migration with UI work.
- Never commit `*.db`, `*.log`, `node_modules/`, `target/`, real customer data, or anything
  under `legacy/`.
- Never rewrite published history.

## 9. Security

- All SQL goes through SeaORM query builders or parameterized statements. String interpolation
  of user input into SQL is forbidden. Dynamic identifiers must be validated against an
  allowlist derived from the model (see `../docs/13-servicios-externos-y-archivos.md`).
- Attachment uploads validate extension, MIME type and size against the configured allowlist.
  Filenames are sanitized; path traversal is rejected.
- The Tauri capability set is minimal: only the commands and plugins actually used.

## 10. When in doubt

Ask. An incorrect financial formula silently corrupts the client's cash-flow records, which is
the one thing this system exists to protect.
