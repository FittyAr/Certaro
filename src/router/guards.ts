import type { Router } from 'vue-router'

import { i18n } from '@/i18n'

/** See `docs/10-navegacion-y-atajos.md` §2.4. */

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

/** The list a malformed detail route falls back to. */
const PARENT_OF: Record<string, string> = {
  clienteId: 'clientes',
  proyectoId: 'proyectos',
  trabajoId: 'trabajos',
  ordenId: 'trabajos',
  certificadoId: 'certificados',
  liquidacionId: 'liquidaciones',
}

export function registerGuards(router: Router, appName: () => string): void {
  router.beforeEach((to) => {
    const idParam = to.meta.idParam
    if (!idParam) return true

    const value = to.params[idParam]
    const raw = Array.isArray(value) ? value[0] : value
    if (typeof raw === 'string' && UUID.test(raw)) return true

    // A hand-edited or stale link lands on the list rather than on a detail screen that would
    // immediately fail its fetch.
    return { name: PARENT_OF[idParam] ?? 'dashboard' }
  })

  router.afterEach((to) => {
    const title = i18n.global.t(to.meta.titleKey ?? 'Menu.Dashboard')
    document.title = `${title} — ${appName()}`
  })
}
