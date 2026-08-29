import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, type RouteLocationRaw } from 'vue-router'

import { useNavigationStore } from '@/stores/useNavigationStore'

/**
 * Breadcrumbs derived from `route.meta.breadcrumb`. See `docs/10-navegacion-y-atajos.md` §6.
 *
 * The application name is not part of them: it is already in the window title. The legacy
 * indicator was a flat `"{app} / {section}"` string with no hierarchy and nothing to click, so
 * standing in the detail of a job there was no way to tell which site it belonged to.
 */

export interface Crumb {
  label: string
  to?: RouteLocationRaw
}

export function useBreadcrumb() {
  const route = useRoute()
  const { t } = useI18n()
  const navigation = useNavigationStore()

  const crumbs = computed<Crumb[]>(() => {
    const chain = (route.meta.breadcrumb ?? []) as string[]

    const ancestors = chain.map<Crumb>((name) => ({
      // A level that names an entity shows the name the detail response resolved; until it
      // arrives it shows a placeholder rather than an empty gap that shifts the layout.
      label: navigation.resolvedName(name) ?? t(labelKeyOf(name)),
      to: { name, params: route.params },
    }))

    return [...ancestors, { label: t(route.meta.titleKey ?? 'Menu.Dashboard') }]
  })

  /** The chain of route names, closest last. Used to highlight the menu entry. */
  const routeChain = computed(() => [
    ...((route.meta.breadcrumb ?? []) as string[]),
    String(route.name ?? ''),
  ])

  return { crumbs, routeChain }
}

/**
 * Title key of an ancestor route. The map is small and explicit because reading it from the
 * router would require a lookup that the ancestor may not have yet during a transition.
 */
function labelKeyOf(routeName: string): string {
  const keys: Record<string, string> = {
    dashboard: 'Menu.Dashboard',
    movimientos: 'Menu.Movimientos',
    clientes: 'Menu.Clientes',
    'cliente-detalle': 'Menu.Clientes',
    obras: 'Menu.Obras',
    'obra-detalle': 'Menu.Obras',
    trabajos: 'Menu.Trabajos',
    'trabajo-detalle': 'Menu.Trabajos',
    certificados: 'Menu.Certificados',
    facturas: 'Menu.Facturas',
    empleados: 'Menu.Empleados',
    asistencia: 'Menu.Asistencia',
    liquidaciones: 'Menu.Liquidaciones',
  }
  return keys[routeName] ?? 'Menu.Dashboard'
}
