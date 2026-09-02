# Changelog

All notable changes to Certaro will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-09-02

Major Enterprise Expansion.

### Added

- **Multi-Database Engine Support**:
  - Native connection and pooling for SQLite (local desktop), PostgreSQL (enterprise server/Docker), and MySQL / MariaDB.
  - Multi-dialect portable SeaORM migrations and cross-platform DDL.
- **Enterprise Authentication & RBAC System**:
  - 7 security tables: `usuarios`, `roles`, `permisos`, `usuario_roles`, `rol_permisos`, `sesiones`, `auth_externo`.
  - Argon2id password hashing, JWT session management, and TOTP 2-factor authentication.
  - 39 granular permissions covering every operational and commercial module.
  - Initial Super Administrator account auto-seeded (`admin@certaro.local`).
  - Transparent bypass mode for SQLite desktop deployments (no login required, full permissions).
- **Kanban Board Module**:
  - Custom boards with configurable columns, WIP limits, card priorities, markdown descriptions, and checklists.
  - Preset boards for `Trabajos` and `Órdenes de Trabajo` with automatic bidirectional state synchronization.
  - 21 Tauri IPC commands and reactive Pinia store.
- **Calendar & Scheduler Module**:
  - Month, Week, Day, and Resource Day views (columns per employee/vehicle/equipment).
  - Virtual event projections: automatically renders national holidays, job start/deadlines, and invoice due dates.
  - Resource groups ("Personal", "Vehículos", "Equipos") with automatic employee syncing.
- **Unified Backup & Restore**:
  - Universal JSON dump with strict topological dependency ordering across all 38 business tables.
  - Atomic transactions and versioned schema validation.

## [0.1.0] - 2026-08-30

First public release. Complete rewrite of __ElectroObraApp_PLACEHOLDER__ from C#/Avalonia to Rust/Tauri/Vue.

### Added

- **Movimientos**: cash ledger with server-side filtering, paging, and sorting. Supports ARS and USD with exchange rate lookup.
- **Categorías y Tipos de Movimiento**: CRUD with hierarchical categories and color coding.
- **Clientes**: CRUD with multiple contacts per client, CUIT normalization, and WhatsApp/email deep links.
- **Obras y Trabajos**: site and job management with state machine (Pausado/EnProgreso/Finalizado/Cancelado).
- **Facturas**: invoicing with partial payments, automatic state reclassification, and overdue tracking.
- **Órdenes de Trabajo**: work orders with line items and progress percentages.
- **Certificados**: frozen progress certificates with9-column layout, UOCRA adjustment, and PDF export.
- **Empleados**: employee management with personal details, employment dates, and daily rates.
- **Asistencia**: daily attendance tracking with day-type classification (normal/Saturday/Sunday/holiday).
- **Liquidaciones**: payroll settlements with premium multipliers, advance deduction, and PDF export.
- **Feriados**: holiday management with API sync and manual entry. Manual entries are never overwritten.
- **Dashboard**: KPIs, monthly charts, profitability rankings, current account, debt aging, and exchange rates.
- **Reportes**: PDF, XLSX, DOCX, CSV, and JSON export for movements. PDF for settlements and certificates. All reports are localized.
- **Adjuntos**: file attachments with size/extension validation, trash retention, and OS integration (open/reveal).
- **Backup**: automatic and manual backups with `VACUUM INTO`, integrity verification, and retention policy.
- **Export/Import JSON**: full database dump and restore with schema version validation and atomic import.
- **Configuración**:5-section settings screen (General, Business, Communication, Integrations, System).
- **Migración legacy**: `eo-import-legacy` binary that reads the C# SQLite database and imports into the new schema with scale detection, date conversion, and18 integration tests.
- **i18n**: Spanish and English with `vue-i18n` on the frontend and embedded JSON files for backend reports.
- **Dark mode**: system, light, and dark theme support.
- **Keyboard shortcuts**: Ctrl+N for new, Escape to close drawers, command palette.

### Technical

- **Backend**: Rust with SeaORM, SQLite, Clean Architecture (eo-domain, eo-application, eo-infrastructure).
- **Frontend**: Vue 3 + Pinia + PrimeVue + Shadcn-Vue + Tailwind CSS.
- **Desktop**: Tauri 2.x with IPC commands.
- **Testing**: 509 Rust tests + 81 frontend tests. xUnit.v3 for Rust, Vitest for frontend.
- **CI**: GitHub Actions with build-test on every push, release workflow for Windows/Linux/macOS.
- **Logging**: `tracing` with JSON format, daily rotation, and trace ID correlation.
- **Decimal handling**: `Money(i64)` and `Decimal4(i64)` newtypes with4 decimal places and half-away-from-zero rounding.
- **Soft delete**: `is_deleted` + `deleted_at` on all business entities.
- **Optimistic concurrency**: `row_version` (8-byte BLOB) on all mutable entities.

### Known limitations

- Code signing is not implemented. Windows shows SmartScreen warning; macOS requires right-click to open.
- Auto-update is not implemented. The app checks GitHub for new versions on startup.
- The `eo-import-legacy` tool does not yet handle the `feriados` derivation from the legacy config file (they are recovered by API sync on first launch).
