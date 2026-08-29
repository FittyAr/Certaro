# ElectroObra — Design System

## Direction and feel

**Who:** Owner of an electrical contracting company in Argentina. Opens the app in the office or on a tablet after visiting a job site.

**Task:** Load and review numbers (money, days, meters, debt). Not "explore a dashboard."

**Feel:** Job-site ledger at 7am — dense, serious, technical paper on concrete. Not SaaS startup. Not dark IDE.

## Signature

- **Obra plate:** work number + name as identity marker in lists and certificates.
- **Document views:** Certificados and Liquidaciones render like the paper they replace.
- **Tabular amounts:** All money uses `Mono` class with tabular numerals.

## Color world (tokens)

| Token | Role | Light |
|-------|------|-------|
| Concrete | Canvas / sidebar | `#F3F1EC` |
| Paper | Raised surfaces | `#FFFCF7` |
| Revoque | Inset / sunken | `#E8E4DC` |
| LedgerInk | Primary text | `#1A1916` |
| Copper | Brand accent, primary actions | `#B87333` |
| Blueprint | Focus, links, info | `#2B5F8A` |
| TapeAmber | Warning, paused works | `#C47A00` |

Semantic aliases: `SurfaceBase`, `Accent`, `TextPrimary`, etc. map to world tokens in `Palette.axaml`.

## Depth strategy

- **Light:** low-opacity borders + one elevation step (`Elevation1`/`Elevation2`). No heavy shadows (WASM-friendly).
- **Dark:** border rings only; minimal shadow.
- **Sidebar:** same background as canvas (`Concrete` / `MainBackgroundBrush`).

## Typography

- **UI:** Inter (Avalonia bundled) / IBM Plex Sans on web splash.
- **Scale:** ratio ~1.25 — Display 44, H1 24, Body 14, Caption 11.
- **Hierarchy:** weight + color over size alone; Display for hero balance on Dashboard.
- **Mono:** amounts, codes, certificate numbers — `FontFeatureSettings: tnum`.

## Spacing and density

- Base unit: 4px; workbench padding 12–16px on controls.
- Page content padding: 24px via `AppShell`.
- Air between groups only; controls stay dense.

## Components

| Component | Notes |
|-----------|-------|
| `Button.eo-primary` | Copper fill, 36px min height, Radius6 |
| `Button.eo-secondary` | Ghost + border |
| `TextBox.eo-input` | Inset (`SurfaceSunken`) |
| `DataGrid.eo-grid` | Flat header, zebra rows, copper hover |
| Nav `menuItem.active` | 2px copper left bar + `NavActiveBackground`, no solid fill |
| `EmptyState` | Master-detail placeholder when no selection |

## Screen archetypes

1. **Master-detail:** Clientes, Obras, Empleados, Facturas — list left, inspector right (no modal overlay).
2. **Document:** Certificados, Liquidaciones — paper layout, export matches screen.
3. **Planilla:** Asistencia — inline grid, semantic jornada colors.
4. **Dashboard:** One hero balance + actionable alert list.

## Theme default

- **Light** (`RequestedThemeVariant="Light"`) for office use.
- Dark available for night shift; derived from same hue family.
