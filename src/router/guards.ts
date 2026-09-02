import type { Router } from 'vue-router'
import { getActivePinia } from 'pinia'

import { i18n } from '@/i18n'
import { useAuthStore } from '@/stores/useAuthStore'

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
  router.beforeEach(async (to) => {
    const idParam = to.meta.idParam
    if (idParam) {
      const value = to.params[idParam]
      const raw = Array.isArray(value) ? value[0] : value
      if (typeof raw !== 'string' || !UUID.test(raw)) {
        // A hand-edited or stale link lands on the list rather than on a detail screen that would
        // immediately fail its fetch.
        return { name: PARENT_OF[idParam] ?? 'dashboard' }
      }
    }

    if (getActivePinia()) {
      const authStore = useAuthStore()
      if (!authStore.initialized) {
        await authStore.init()
      }

      // If login is required and user is not authenticated, redirect to login
      if (authStore.requiresLogin && !authStore.isAuthenticated) {
        if (to.name !== 'login') {
          return { name: 'login', query: { redirect: to.fullPath } }
        }
        return true
      }

      // If authenticated (or SQLite mode) and visiting login, go to dashboard
      if (authStore.isAuthenticated && to.meta.guestOnly) {
        return { name: 'dashboard' }
      }

      // Check granular permission
      if (to.meta.permission && !authStore.hasPermission(to.meta.permission)) {
        return { name: 'dashboard' }
      }
    }

    return true
  })

  router.afterEach((to) => {
    const title = i18n.global.t(to.meta.titleKey ?? 'Menu.Dashboard')
    document.title = `${title} — ${appName()}`
  })
}
