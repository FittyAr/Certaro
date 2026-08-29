import type { RouteRecordRaw } from 'vue-router'

/**
 * The 15 navigation routes plus the nested detail ones. See `docs/10-navegacion-y-atajos.md` §2.
 *
 * Creating and editing a list record is **not** a route: it is a drawer over the list, so closing
 * it returns to the same page, filters and scroll without serialising all of that state. The
 * exception is an entity with children — Obra, Trabajo, Orden, Certificado — which gets a detail
 * route because it is a working screen rather than a form.
 */

declare module 'vue-router' {
  interface RouteMeta {
    /** i18n key of the section title. */
    titleKey: string
    /** Chain of ancestor route names, closest last. Drives the breadcrumb. */
    breadcrumb?: string[]
    /** Name of the route parameter that must be a UUID. */
    idParam?: string
    /** Registered only when the seed screen is enabled. */
    devOnly?: boolean
  }
}

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'dashboard',
    component: () => import('@/views/dashboard/DashboardView.vue'),
    meta: { titleKey: 'Menu.Dashboard' },
  },
  {
    path: '/movimientos',
    name: 'movimientos',
    component: () => import('@/views/movimientos/MovimientosView.vue'),
    meta: { titleKey: 'Menu.Movimientos' },
  },

  // ------------------------------------------------------------------ comercial
  {
    path: '/clientes',
    name: 'clientes',
    component: () => import('@/views/clientes/ClientesView.vue'),
    meta: { titleKey: 'Menu.Clientes' },
  },
  {
    path: '/clientes/:clienteId',
    name: 'cliente-detalle',
    component: () => import('@/views/clientes/ClienteDetalleView.vue'),
    meta: { titleKey: 'Menu.Clientes', breadcrumb: ['clientes'], idParam: 'clienteId' },
  },
  {
    path: '/clientes/:clienteId/cuenta-corriente',
    name: 'cliente-cuenta',
    component: () => import('@/views/comercial/CuentaCorrienteView.vue'),
    meta: {
      titleKey: 'Comercial.CuentaCorriente.Title',
      breadcrumb: ['clientes', 'cliente-detalle'],
      idParam: 'clienteId',
    },
  },
  {
    path: '/obras',
    name: 'obras',
    component: () => import('@/views/obras/ObrasView.vue'),
    meta: { titleKey: 'Menu.Obras' },
  },
  {
    path: '/obras/:obraId',
    name: 'obra-detalle',
    component: () => import('@/views/obras/ObraDetalleView.vue'),
    meta: { titleKey: 'Menu.Obras', breadcrumb: ['obras'], idParam: 'obraId' },
  },
  {
    path: '/obras/:obraId/trabajos',
    name: 'obra-trabajos',
    component: () => import('@/views/obras/ObraTrabajosView.vue'),
    meta: {
      titleKey: 'Menu.Trabajos',
      breadcrumb: ['obras', 'obra-detalle'],
      idParam: 'obraId',
    },
  },
  {
    path: '/obras/:obraId/caja',
    name: 'obra-caja',
    component: () => import('@/views/obras/ObraCajaView.vue'),
    meta: {
      titleKey: 'Obras.Caja.Title',
      breadcrumb: ['obras', 'obra-detalle'],
      idParam: 'obraId',
    },
  },
  {
    path: '/trabajos',
    name: 'trabajos',
    component: () => import('@/views/trabajos/TrabajosView.vue'),
    meta: { titleKey: 'Menu.Trabajos' },
  },
  {
    path: '/trabajos/:trabajoId',
    name: 'trabajo-detalle',
    component: () => import('@/views/trabajos/TrabajoDetalleView.vue'),
    meta: { titleKey: 'Menu.Trabajos', breadcrumb: ['trabajos'], idParam: 'trabajoId' },
  },
  {
    path: '/trabajos/:trabajoId/ordenes',
    name: 'trabajo-ordenes',
    component: () => import('@/views/ordenes/OrdenesView.vue'),
    meta: {
      titleKey: 'Ordenes.Title',
      breadcrumb: ['trabajos', 'trabajo-detalle'],
      idParam: 'trabajoId',
    },
  },
  {
    path: '/ordenes/:ordenId',
    name: 'orden-detalle',
    component: () => import('@/views/ordenes/OrdenDetalleView.vue'),
    meta: { titleKey: 'Ordenes.Title', breadcrumb: ['trabajos'], idParam: 'ordenId' },
  },
  {
    path: '/certificados',
    name: 'certificados',
    component: () => import('@/views/certificados/CertificadosView.vue'),
    meta: { titleKey: 'Menu.Certificados' },
  },
  {
    path: '/certificados/:certificadoId',
    name: 'certificado-detalle',
    component: () => import('@/views/certificados/CertificadoDetalleView.vue'),
    meta: {
      titleKey: 'Menu.Certificados',
      breadcrumb: ['certificados'],
      idParam: 'certificadoId',
    },
  },
  {
    path: '/facturas',
    name: 'facturas',
    component: () => import('@/views/facturas/FacturasView.vue'),
    meta: { titleKey: 'Menu.Facturas' },
  },

  // ------------------------------------------------------------------ personal
  {
    path: '/empleados',
    name: 'empleados',
    component: () => import('@/views/empleados/EmpleadosView.vue'),
    meta: { titleKey: 'Menu.Empleados' },
  },
  {
    path: '/asistencia',
    name: 'asistencia',
    component: () => import('@/views/asistencia/AsistenciaView.vue'),
    meta: { titleKey: 'Menu.Asistencia' },
  },
  {
    path: '/liquidaciones',
    name: 'liquidaciones',
    component: () => import('@/views/liquidaciones/LiquidacionesView.vue'),
    meta: { titleKey: 'Menu.Liquidaciones' },
  },
  {
    path: '/liquidaciones/:liquidacionId',
    name: 'liquidacion-detalle',
    component: () => import('@/views/liquidaciones/LiquidacionDetalleView.vue'),
    meta: {
      titleKey: 'Menu.Liquidaciones',
      breadcrumb: ['liquidaciones'],
      idParam: 'liquidacionId',
    },
  },

  // ------------------------------------------------------------------ sistema
  {
    path: '/reportes',
    name: 'reportes',
    component: () => import('@/views/reportes/ReportesView.vue'),
    meta: { titleKey: 'Menu.Reports' },
  },
  {
    path: '/admin/categorias',
    name: 'categorias',
    component: () => import('@/views/categorias/CategoriasView.vue'),
    meta: { titleKey: 'Menu.Categories' },
  },
  {
    path: '/admin/tipos-movimiento',
    name: 'tipos-movimiento',
    component: () => import('@/views/tipos-movimiento/TiposMovimientoView.vue'),
    meta: { titleKey: 'Menu.MovementTypes' },
  },
  {
    path: '/configuracion',
    name: 'configuracion',
    component: () => import('@/views/configuracion/ConfiguracionView.vue'),
    meta: { titleKey: 'Menu.Settings' },
  },
  {
    path: '/dev/seed',
    name: 'seed',
    component: () => import('@/views/seed/SeedView.vue'),
    meta: { titleKey: 'Menu.Seed', devOnly: true },
  },

  // An unknown route goes to the dashboard: there is no 404 screen in a desktop application whose
  // links are all generated by itself.
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

/** The routes to register, given whether the development screens are enabled. */
export function activeRoutes(seedEnabled: boolean): RouteRecordRaw[] {
  return routes.filter((route) => seedEnabled || !route.meta?.devOnly)
}
