import { describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import en from '@/locales/en.json'
import es from '@/locales/es.json'
import { activeMenuRoute, MENU, menuItems, numericShortcutRoutes } from '@/router/menu'
import { activeRoutes, routes } from '@/router/routes'

/** The mandatory tests of `docs/10-navegacion-y-atajos.md` §9. */

/** Route names that are reachable only from a parent screen and so are not menu entries. */
const DETAIL_ROUTES = new Set([
  'cliente-detalle',
  'cliente-cuenta',
  'obra-detalle',
  'obra-trabajos',
  'obra-caja',
  'trabajo-detalle',
  'trabajo-ordenes',
  'orden-detalle',
  'certificado-detalle',
  'liquidacion-detalle',
  'welcome',
])

function topLevelRoutes(): string[] {
  return routes
    .map((route) => String(route.name ?? ''))
    .filter((name) => name && !DETAIL_ROUTES.has(name))
}

function lookup(dictionary: object, key: string): unknown {
  return key
    .split('.')
    .reduce<unknown>(
      (node, part) => (node as Record<string, unknown> | undefined)?.[part],
      dictionary,
    )
}

describe('menú y rutas', () => {
  it('el menu cubre todas las rutas de nivel superior, y viceversa', () => {
    // The test that would have caught Trabajos having no menu entry for a whole release.
    const inMenu = menuItems(true)
      .map((item) => item.route)
      .sort()
    expect(topLevelRoutes().sort()).toEqual(inMenu)
  })

  it('los atajos numericos derivan de los primeros nueve items del menu', () => {
    expect(numericShortcutRoutes()).toEqual([
      'dashboard',
      'movimientos',
      'clientes',
      'obras',
      'trabajos',
      'certificados',
      'facturas',
      'empleados',
      'asistencia',
    ])
  })

  it('toda ruta tiene titulo traducido en los dos idiomas', () => {
    for (const route of routes) {
      const key = route.meta?.titleKey
      if (!key) continue
      expect(lookup(es, key), `${key} en es.json`).toBeTypeOf('string')
      expect(lookup(en, key), `${key} en en.json`).toBeTypeOf('string')
    }
  })

  it('toda etiqueta del menu esta traducida en los dos idiomas', () => {
    for (const group of MENU) {
      for (const key of [group.labelKey, ...group.items.map((i) => i.labelKey)]) {
        expect(lookup(es, key), `${key} en es.json`).toBeTypeOf('string')
        expect(lookup(en, key), `${key} en en.json`).toBeTypeOf('string')
      }
    }
  })

  it('la ruta seed no se registra sin las pantallas de desarrollo', () => {
    const names = activeRoutes(false).map((r) => r.name)
    expect(names).not.toContain('seed')
    expect(activeRoutes(true).map((r) => r.name)).toContain('seed')
  })

  it('se resalta el ancestro mas cercano presente en el menu', () => {
    // Standing in `/obras/:id/trabajos` the highlighted entry is Obras, not Trabajos: that is
    // how the screen was reached.
    expect(activeMenuRoute(['obras', 'obra-detalle', 'obra-trabajos'])).toBe('obras')
    expect(activeMenuRoute(['clientes', 'cliente-detalle'])).toBe('clientes')
    expect(activeMenuRoute(['movimientos'])).toBe('movimientos')
  })

  it('toda ruta con :id declara el parametro que hay que validar', () => {
    for (const route of routes) {
      if (!route.path.includes(':') || route.path.includes('pathMatch')) continue
      expect(route.meta?.idParam, `${route.path} sin idParam`).toBeTypeOf('string')
      expect(route.path).toContain(`:${route.meta?.idParam}`)
    }
  })
})

describe('guardas', () => {
  async function routerWithGuard() {
    // Memory history: a test has no address bar, and the guards under test do not depend on one.
    const router = createRouter({ history: createMemoryHistory(), routes: activeRoutes(false) })
    const { registerGuards } = await import('@/router/guards')
    registerGuards(router, () => 'ElectroObra')
    return router
  }

  it('un :id que no es uuid redirige al listado padre', async () => {
    const router = await routerWithGuard()
    await router.push('/obras/no-es-un-uuid')
    expect(router.currentRoute.value.name).toBe('obras')
  })

  it('un :id valido entra al detalle', async () => {
    const router = await routerWithGuard()
    await router.push('/obras/018f4c1e-6f7a-7c3d-9b21-0a1b2c3d4e5f')
    expect(router.currentRoute.value.name).toBe('obra-detalle')
  })

  it('una ruta desconocida cae en el panel, no en un 404', async () => {
    const router = await routerWithGuard()
    await router.push('/no-existe')
    expect(router.currentRoute.value.name).toBe('dashboard')
  })
})
