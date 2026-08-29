import { createRouter, createWebHistory } from 'vue-router'

import { registerGuards } from './guards'
import { activeRoutes } from './routes'

/**
 * The seed screen is decided at construction time rather than by a guard: a route that must not
 * exist in a release build is better left unregistered than merely blocked.
 */
export function createAppRouter(options: { seedEnabled: boolean; appName: () => string }) {
  const router = createRouter({
    history: createWebHistory(),
    routes: activeRoutes(options.seedEnabled),
    // A desktop application restores the top of the page on every navigation; there is no browser
    // back-forward cache to honour.
    scrollBehavior: () => ({ top: 0 }),
  })
  registerGuards(router, options.appName)
  return router
}

export type AppRouter = ReturnType<typeof createAppRouter>
