import type { RouteRecordRaw } from 'vue-router'

/**
 * Routes. The full table of 15 routes from `docs/10-navegacion-y-atajos.md` §2 lands in phase 3;
 * phase 0 only needs the shell to resolve somewhere.
 */
export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'dashboard',
    component: () => import('@/views/dashboard/DashboardView.vue'),
    meta: { titleKey: 'Menu.Dashboard' },
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: () => import('@/views/errors/NotFoundView.vue'),
    meta: { titleKey: 'Errors.NotFoundTitle' },
  },
]
