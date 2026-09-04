import {
  DEFAULT_CONFIG,
  createSeedMockDb,
  mockConfig,
  mockDb,
  saveMockConfig,
  setMockConfig,
  setMockDb,
} from '../database'

export function handleSystemCommand<T>(command: string, args?: Record<string, unknown>): T | undefined {
  switch (command) {
    case 'app_is_ready':
      return true as T
    case 'ping':
      return `pong: ${String(args?.message ?? '')}` as T
    case 'app_info':
      return {
        name: mockConfig.application.name,
        version: '0.1.0',
        environment: 'development',
        dataDir: 'Browser Preview Mode',
      } as T
    case 'app_config':
    case 'config_get_all':
      return structuredClone(mockConfig) as T
    case 'config_set': {
      const cambios = (args?.cambios ?? {}) as Record<string, unknown>
      for (const [key, val] of Object.entries(cambios)) {
        const parts = key.split('.')
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        let target: any = mockConfig
        for (let i = 0; i < parts.length - 1; i++) {
          const part = parts[i]
          if (part) target = target[part]
        }
        if (target && parts.length > 0) {
          const last = parts[parts.length - 1]
          if (last) {
            try {
              target[last] =
                typeof val === 'string' && (val.startsWith('{') || val.startsWith('['))
                  ? JSON.parse(val)
                  : val
            } catch {
              target[last] = val
            }
          }
        }
      }
      saveMockConfig(mockConfig)
      return structuredClone(mockConfig) as T
    }
    case 'config_reset': {
      const resetConfig = structuredClone(DEFAULT_CONFIG)
      setMockConfig(resetConfig)
      return structuredClone(resetConfig) as T
    }
    case 'sistema_detect_legacy_db':
      return null as T
    case 'sistema_info':
      return {
        version: '0.1.0',
        baseSaludable: true,
        estado: 'Dashboard.Estado.Saludable',
        migraciones: 2,
        tamanoBytes: 524288,
      } as unknown as T
    case 'dev_seed_database': {
      const newSeed = createSeedMockDb()
      setMockDb(newSeed)
      return {
        categorias: newSeed.categorias.length,
        tiposMovimiento: newSeed.tiposMovimiento.length,
        empleados: newSeed.empleados.length,
        clientes: newSeed.clientes.length,
        proyectos: newSeed.proyectos.length,
        trabajos: newSeed.trabajos.length,
        ordenesTrabajo: newSeed.ordenes.length,
        movimientos: newSeed.movimientos.length,
        facturas: newSeed.facturas.length,
        liquidaciones: newSeed.liquidaciones.length,
      } as unknown as T
    }
    case 'backup_list':
      return [] as T
    case 'dashboard_stats':
    case 'dashboard_kpis':
      return structuredClone({
        periodo: (args?.periodo as string) ?? 'Mensual',
        desde: new Date(new Date().getFullYear(), new Date().getMonth(), 1).toISOString(),
        hasta: new Date().toISOString(),
        totalIngresos: '2037000.0000',
        totalGastos: '662000.0000',
        balance: '1375000.0000',
        cantidadMovimientos: mockDb.movimientos.length,
        rentabilidad: '67.5000',
        anteriorIngresos: '1800000.0000',
        anteriorGastos: '550000.0000',
        variacionIngresos: '13.1600',
        variacionGastos: '20.3600',
        variacionBalance: '10.0000',
        clientesActivos: mockDb.clientes.length,
        trabajosPendientes: 3,
        proyectosPausadas: 0,
        facturasVencidas: 0,
        liquidacionesPendientes: 1,
        serieMensual: Array.from({ length: 12 }, (_, i) => ({
          mes: i + 1,
          ingresos: i === new Date().getMonth() ? '2037000.0000' : '1500000.0000',
          gastos: i === new Date().getMonth() ? '662000.0000' : '450000.0000',
        })),
        topClientes: [
          { id: mockDb.clientes[2]?.id ?? '', nombre: mockDb.clientes[2]?.nombre ?? '', total: '1452000.0000' },
          { id: mockDb.clientes[0]?.id ?? '', nombre: mockDb.clientes[0]?.nombre ?? '', total: '500000.0000' },
          { id: mockDb.clientes[1]?.id ?? '', nombre: mockDb.clientes[1]?.nombre ?? '', total: '85000.0000' },
        ],
        gastosPorCategoria: [
          { id: mockDb.categorias[1]?.id ?? '', nombre: mockDb.categorias[1]?.nombre ?? '', colorHex: mockDb.categorias[1]?.colorHex ?? '#3B82F6', total: '340000.0000', porcentaje: '51.3600' },
          { id: mockDb.categorias[2]?.id ?? '', nombre: mockDb.categorias[2]?.nombre ?? '', colorHex: mockDb.categorias[2]?.colorHex ?? '#F59E0B', total: '125000.0000', porcentaje: '18.8800' },
          { id: mockDb.categorias[4]?.id ?? '', nombre: mockDb.categorias[4]?.nombre ?? '', colorHex: mockDb.categorias[4]?.colorHex ?? '#EF4444', total: '62000.0000', porcentaje: '9.3600' },
        ],
        mejoresProyectos: [
          { id: mockDb.proyectos[0]?.id ?? '', numero: 1, nombre: mockDb.proyectos[0]?.nombre ?? '', rentabilidad: '1112000.0000', margen: '76.5800' },
        ],
        peoresProyectos: [],
        ultimosMovimientos: mockDb.movimientos.slice(0, 5),
        estadoSistema: {
          version: '0.1.0',
          baseSaludable: true,
          estado: 'Dashboard.Estado.Saludable',
          migraciones: 2,
          tamanoBytes: 524288,
        },
      }) as unknown as T
    case 'dashboard_serie_mensual':
      return structuredClone(Array.from({ length: 12 }, (_, i) => ({
        mes: i + 1,
        ingresos: i === new Date().getMonth() ? '2037000.0000' : '1500000.0000',
        gastos: i === new Date().getMonth() ? '662000.0000' : '450000.0000',
      }))) as T
    case 'dashboard_top_clientes':
      return structuredClone([
        { id: mockDb.clientes[2]?.id ?? '', nombre: mockDb.clientes[2]?.nombre ?? '', total: '1452000.0000' },
        { id: mockDb.clientes[0]?.id ?? '', nombre: mockDb.clientes[0]?.nombre ?? '', total: '500000.0000' },
        { id: mockDb.clientes[1]?.id ?? '', nombre: mockDb.clientes[1]?.nombre ?? '', total: '85000.0000' },
      ]) as T
    case 'dashboard_gastos_categorias':
      return structuredClone([
        { id: mockDb.categorias[1]?.id ?? '', nombre: mockDb.categorias[1]?.nombre ?? '', colorHex: mockDb.categorias[1]?.colorHex ?? '#3B82F6', total: '340000.0000', porcentaje: '51.3600' },
        { id: mockDb.categorias[2]?.id ?? '', nombre: mockDb.categorias[2]?.nombre ?? '', colorHex: mockDb.categorias[2]?.colorHex ?? '#F59E0B', total: '125000.0000', porcentaje: '18.8800' },
        { id: mockDb.categorias[4]?.id ?? '', nombre: mockDb.categorias[4]?.nombre ?? '', colorHex: mockDb.categorias[4]?.colorHex ?? '#EF4444', total: '62000.0000', porcentaje: '9.3600' },
      ]) as T
    case 'dashboard_rentabilidad_proyectos':
      return structuredClone([
        { id: mockDb.proyectos[0]?.id ?? '', numero: 1, nombre: mockDb.proyectos[0]?.nombre ?? '', rentabilidad: '1112000.0000', margen: '76.5800' },
      ]) as T
    case 'dashboard_ultimos_movimientos':
      return structuredClone(mockDb.movimientos.slice(0, 10)) as T
    case 'dashboard_alertas':
      return [] as T
    case 'cotizaciones_list':
    case 'cotizaciones_get':
      return [
        {
          casa: 'blue',
          nombre: 'Dólar Blue',
          compra: '1280.0000',
          venta: '1300.0000',
          fechaActualizacion: new Date().toISOString(),
          esVieja: false,
        },
        {
          casa: 'oficial',
          nombre: 'Dólar Oficial',
          compra: '950.0000',
          venta: '990.0000',
          fechaActualizacion: new Date().toISOString(),
          esVieja: false,
        },
      ] as T
    case 'feriados_list':
      return structuredClone(mockDb.feriados) as T
    case 'feriados_sync':
      return { agregados: 0, total: mockDb.feriados.length, aniosConError: 0 } as T
    default:
      return undefined
  }
}
