import { createRouter, createWebHistory } from 'vue-router'

import { routes } from './routes'

export const router = createRouter({
  history: createWebHistory(),
  routes,
  // A desktop application restores the top of the page on every navigation; there is no browser
  // back-forward cache to honour.
  scrollBehavior: () => ({ top: 0 }),
})
