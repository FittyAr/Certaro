/**
 * The menu, as data. See `docs/10-navegacion-y-atajos.md` §3.
 *
 * The sidebar, the numeric shortcuts and the navigation section of the command palette are all
 * derived from this one structure. In the legacy system the menu was written item by item in the
 * markup and the shortcut list was a separate array kept in sync by hand, which is exactly why the
 * Trabajos screen ended up with no menu entry at all.
 */

export interface MenuItem {
  /** `name` of the route. */
  route: string
  labelKey: string
  /** Lucide icon name. */
  icon: string
  /** Extra i18n keys the command palette matches against, for synonyms. */
  synonymKeys?: string[]
  /** Only registered when `Application.SeedEnabled` is on. */
  devOnly?: boolean
  /** Required permission to see in menu. */
  permission?: string
}

export interface MenuGroup {
  labelKey: string
  items: MenuItem[]
  collapsible?: boolean
  defaultExpanded?: boolean
}

export const MENU: MenuGroup[] = [
  {
    labelKey: 'Menu.Group.Operacion',
    collapsible: true,
    defaultExpanded: true,
    items: [
      { route: 'dashboard', labelKey: 'Menu.Dashboard', icon: 'layout-dashboard' },
      {
        route: 'movimientos',
        labelKey: 'Menu.Movimientos',
        icon: 'arrow-left-right',
        synonymKeys: ['Menu.Synonym.Caja', 'Menu.Synonym.Gastos'],
      },
    ],
  },
  {
    labelKey: 'Menu.Group.Comercial',
    collapsible: true,
    defaultExpanded: true,
    items: [
      { route: 'clientes', labelKey: 'Menu.Clientes', icon: 'users' },
      { route: 'proyectos', labelKey: 'Menu.Proyectos', icon: 'building-2' },
      { route: 'trabajos', labelKey: 'Menu.Trabajos', icon: 'hammer' },
      { route: 'kanban', labelKey: 'Menu.Kanban', icon: 'kanban', permission: 'kanban:ver' },
      { route: 'calendario', labelKey: 'Menu.Calendario', icon: 'calendar', permission: 'calendario:ver' },
      { route: 'certificados', labelKey: 'Menu.Certificados', icon: 'file-badge' },
      {
        route: 'facturas',
        labelKey: 'Menu.Facturas',
        icon: 'receipt',
        synonymKeys: ['Menu.Synonym.Cobros'],
      },
    ],
  },
  {
    labelKey: 'Menu.Group.Personal',
    collapsible: true,
    defaultExpanded: true,
    items: [
      { route: 'empleados', labelKey: 'Menu.Empleados', icon: 'id-card' },
      { route: 'asistencia', labelKey: 'Menu.Asistencia', icon: 'calendar-check' },
      {
        route: 'liquidaciones',
        labelKey: 'Menu.Liquidaciones',
        icon: 'banknote',
        synonymKeys: ['Menu.Synonym.Sueldos'],
      },
    ],
  },
  {
    labelKey: 'Menu.Group.Sistema',
    collapsible: true,
    defaultExpanded: true,
    items: [
      { route: 'reportes', labelKey: 'Menu.Reports', icon: 'file-chart-column' },
      { route: 'categorias', labelKey: 'Menu.Categories', icon: 'tags' },
      { route: 'tipos-movimiento', labelKey: 'Menu.MovementTypes', icon: 'list-tree' },
      { route: 'feriados', labelKey: 'Menu.Feriados', icon: 'calendar-days' },
      { route: 'usuarios', labelKey: 'Menu.Usuarios', icon: 'users-round', permission: 'usuarios:ver' },
      { route: 'roles', labelKey: 'Menu.Roles', icon: 'shield-check', permission: 'usuarios:gestionar_roles' },
      { route: 'configuracion', labelKey: 'Menu.Settings', icon: 'settings' },
      { route: 'seed', labelKey: 'Menu.Seed', icon: 'database', devOnly: true },
    ],
  },
]

/** Every item, flattened, in menu order. */
export function menuItems(includeDevOnly = false): MenuItem[] {
  return MENU.flatMap((group) => group.items).filter((item) => includeDevOnly || !item.devOnly)
}

/**
 * `Ctrl+1` … `Ctrl+9`, derived rather than declared: keeping a second list in sync by hand is what
 * broke in the legacy system.
 */
export function numericShortcutRoutes(): string[] {
  return menuItems()
    .slice(0, 9)
    .map((item) => item.route)
}

/** The menu entry to highlight, given the chain of route names of the current location. */
export function activeMenuRoute(routeChain: string[]): string | undefined {
  const known = new Set(menuItems(true).map((item) => item.route))
  // Walked from the deepest ancestor outwards: standing in `/proyectos/:id/trabajos` the entry to
  // highlight is Proyectos, because that is how the screen was reached.
  return [...routeChain].reverse().find((name) => known.has(name))
}
