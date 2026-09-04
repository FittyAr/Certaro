import type { ApiError } from '../client'
import {
  DEFAULT_CONFIG,
  createSeedMockDb,
  generateUuid,
  mockAudit,
  mockConfig,
  mockDb,
  saveMockConfig,
  saveMockDb,
  setMockConfig,
  setMockDb,
} from './database'
import type {
  MockCategory,
  MockCertificado,
  MockCliente,
  MockEmpleado,
  MockFactura,
  MockLiquidacion,
  MockMovimiento,
  MockOrden,
  MockProyecto,
  MockTipoMovimiento,
  MockTrabajo,
} from './types'

export function mockBrowserCommand<T>(command: string, args?: Record<string, unknown>): T {
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

    // ==========================================
    // MOVIMIENTOS
    // ==========================================
    case 'movimientos_list': {
      const query = (args?.query ?? {}) as Record<string, unknown>
      const filtro = (args?.filtro ?? query?.filtro ?? {}) as Record<string, unknown>
      let filtered = [...mockDb.movimientos]
      if (filtro.concepto && typeof filtro.concepto === 'string' && filtro.concepto.trim() !== '') {
        const needle = filtro.concepto.trim().toLowerCase()
        filtered = filtered.filter(m => m.concepto.toLowerCase().includes(needle))
      }
      if (filtro.tipoMovimientoId) {
        filtered = filtered.filter(m => m.tipoMovimientoId === filtro.tipoMovimientoId)
      }
      if (filtro.categoriaId) {
        filtered = filtered.filter(m => m.categoriaId === filtro.categoriaId)
      }
      if (filtro.fechaDesde) {
        filtered = filtered.filter(m => m.fecha >= String(filtro.fechaDesde))
      }
      if (filtro.fechaHasta) {
        filtered = filtered.filter(m => m.fecha <= String(filtro.fechaHasta))
      }
      let totalIngresosNum = 0
      let totalGastosNum = 0
      for (const m of filtered) {
        const val = parseFloat(m.total) || 0
        if (m.esIngreso) totalIngresosNum += val
        else totalGastosNum += val
      }
      const resumen = {
        totalIngresos: totalIngresosNum.toFixed(4),
        totalGastos: totalGastosNum.toFixed(4),
        balance: (totalIngresosNum - totalGastosNum).toFixed(4),
        cantidad: filtered.length,
      }
      return structuredClone({
        items: filtered,
        totalCount: filtered.length,
        page: 1,
        size: 30,
        resumen,
      }) as T
    }
    case 'movimientos_resumen':
    case 'movimiento_resumen': {
      let totalIngresosNum = 0
      let totalGastosNum = 0
      for (const m of mockDb.movimientos) {
        const val = parseFloat(m.total) || 0
        if (m.esIngreso) totalIngresosNum += val
        else totalGastosNum += val
      }
      return structuredClone({
        totalIngresos: totalIngresosNum.toFixed(4),
        totalGastos: totalGastosNum.toFixed(4),
        balance: (totalIngresosNum - totalGastosNum).toFixed(4),
        cantidad: mockDb.movimientos.length,
      }) as T
    }
    case 'movimientos_get':
    case 'movimiento_get': {
      const id = String(args?.id ?? '')
      const found = mockDb.movimientos.find(m => m.id === id) || mockDb.movimientos[0]
      return structuredClone({
        ...found,
        createdAt: found?.createdAt ?? new Date().toISOString(),
        updatedAt: found?.updatedAt ?? null,
      }) as T
    }
    case 'movimientos_create':
    case 'movimiento_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const tipo = mockDb.tiposMovimiento.find(t => t.id === dto.tipoMovimientoId)
      const cat = mockDb.categorias.find(c => c.id === dto.categoriaId)
      const montoNum = parseFloat(String(dto.monto || '0'))
      const cantNum = parseFloat(String(dto.cantidad || '1'))
      const total = (montoNum * cantNum).toFixed(4)
      const newMov: MockMovimiento = {
        id: generateUuid(),
        fecha: String(dto.fecha || new Date().toISOString()),
        concepto: String(dto.concepto || ''),
        monto: String(dto.monto !== undefined ? dto.monto : '0.0000'),
        cantidad: String(dto.cantidad !== undefined ? dto.cantidad : '1.0000'),
        total,
        moneda: String(dto.moneda || 'Ars'),
        cotizacionAplicada: (dto.cotizacionAplicada as string | null) ?? null,
        tipoMovimientoId: String(dto.tipoMovimientoId || ''),
        tipoMovimientoNombre: tipo?.nombre ?? 'General',
        esIngreso: tipo?.esIngreso ?? true,
        categoriaId: (dto.categoriaId as string | null) ?? null,
        categoriaNombre: cat?.nombre ?? null,
        categoriaColor: cat?.colorHex ?? null,
        clienteId: (dto.clienteId as string | null) ?? null,
        trabajoId: (dto.trabajoId as string | null) ?? null,
        empleadoId: (dto.empleadoId as string | null) ?? null,
        facturaId: (dto.facturaId as string | null) ?? null,
        tipoConceptoPagoId: (dto.tipoConceptoPagoId as string | null) ?? null,
        bloqueadoPorLiquidacion: false,
        rowVersion: generateUuid(),
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }
      mockDb.movimientos.unshift(newMov)
      saveMockDb(mockDb)
      return structuredClone(newMov) as T
    }
    case 'movimientos_update':
    case 'movimiento_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.movimientos.findIndex(m => m.id === id)
      if (idx >= 0) {
        const tipo = mockDb.tiposMovimiento.find(t => t.id === dto.tipoMovimientoId)
        const cat = mockDb.categorias.find(c => c.id === dto.categoriaId)
        const montoNum = parseFloat(String(dto.monto !== undefined ? dto.monto : mockDb.movimientos[idx]!.monto))
        const cantNum = parseFloat(String(dto.cantidad !== undefined ? dto.cantidad : mockDb.movimientos[idx]!.cantidad))
        const total = (montoNum * cantNum).toFixed(4)
        mockDb.movimientos[idx] = {
          ...mockDb.movimientos[idx]!,
          fecha: String(dto.fecha || mockDb.movimientos[idx]!.fecha),
          concepto: String(dto.concepto || mockDb.movimientos[idx]!.concepto),
          monto: String(dto.monto !== undefined ? dto.monto : mockDb.movimientos[idx]!.monto),
          cantidad: String(dto.cantidad !== undefined ? dto.cantidad : mockDb.movimientos[idx]!.cantidad),
          total,
          moneda: String(dto.moneda || mockDb.movimientos[idx]!.moneda),
          cotizacionAplicada: (dto.cotizacionAplicada as string | null) ?? null,
          tipoMovimientoId: String(dto.tipoMovimientoId || mockDb.movimientos[idx]!.tipoMovimientoId),
          tipoMovimientoNombre: tipo?.nombre ?? mockDb.movimientos[idx]!.tipoMovimientoNombre,
          esIngreso: tipo?.esIngreso ?? mockDb.movimientos[idx]!.esIngreso,
          categoriaId: (dto.categoriaId as string | null) ?? null,
          categoriaNombre: cat?.nombre ?? null,
          categoriaColor: cat?.colorHex ?? null,
          clienteId: (dto.clienteId as string | null) ?? null,
          trabajoId: (dto.trabajoId as string | null) ?? null,
          empleadoId: (dto.empleadoId as string | null) ?? null,
          facturaId: (dto.facturaId as string | null) ?? null,
          tipoConceptoPagoId: (dto.tipoConceptoPagoId as string | null) ?? null,
          rowVersion: generateUuid(),
          updatedAt: new Date().toISOString(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.movimientos[idx]) as T
      }
      return structuredClone(mockDb.movimientos[0]) as T
    }
    case 'movimientos_delete':
    case 'movimiento_delete': {
      const id = String(args?.id ?? '')
      mockDb.movimientos = mockDb.movimientos.filter(m => m.id !== id)
      saveMockDb(mockDb)
      return null as T
    }

    // ==========================================
    // CLIENTES
    // ==========================================
    case 'clientes_list': {
      const query = (args?.query ?? {}) as Record<string, unknown>
      const filtro = (args?.filtro ?? query?.filtro ?? {}) as Record<string, unknown>
      let filtered = [...mockDb.clientes]
      if (filtro.nombre && typeof filtro.nombre === 'string' && filtro.nombre.trim() !== '') {
        const needle = filtro.nombre.trim().toLowerCase()
        filtered = filtered.filter(c => c.nombre.toLowerCase().includes(needle) || (c.cuit && c.cuit.includes(needle)))
      }
      return structuredClone({ items: filtered, totalCount: filtered.length, page: 1, size: 30 }) as T
    }
    case 'clientes_lookup':
      return structuredClone(mockDb.clientes.map(c => ({ id: c.id, label: c.nombre }))) as T
    case 'clientes_get':
    case 'cliente_get': {
      const id = String(args?.id ?? '')
      const cli = mockDb.clientes.find(c => c.id === id) || mockDb.clientes[0]!
      return structuredClone({
        ...cli,
        contactos: [
          { id: generateUuid(), etiqueta: 'Administración', email: cli.email ?? '', nombre: cli.nombre, telefono: cli.telefono, esPrincipal: true },
        ],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }) as T
    }
    case 'clientes_create':
    case 'cliente_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newCli: MockCliente = {
        id: generateUuid(),
        nombre: String(dto.nombre || ''),
        cuit: (dto.cuit as string | null) ?? null,
        direccion: (dto.direccion as string | null) ?? null,
        telefono: (dto.telefono as string | null) ?? null,
        email: (dto.email as string | null) ?? null,
        condicionIva: (dto.condicionIva as string | null) ?? 'Responsable Inscripto',
        proyectosCount: 0,
        facturasCount: 0,
        deuda: '0.0000',
        puedeEliminarse: true,
        rowVersion: generateUuid(),
      }
      mockDb.clientes.unshift(newCli)
      saveMockDb(mockDb)
      return structuredClone(newCli) as T
    }
    case 'clientes_update':
    case 'cliente_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.clientes.findIndex(c => c.id === id)
      if (idx >= 0) {
        mockDb.clientes[idx] = {
          ...mockDb.clientes[idx]!,
          nombre: String(dto.nombre || mockDb.clientes[idx]!.nombre),
          cuit: (dto.cuit as string | null) ?? mockDb.clientes[idx]!.cuit,
          direccion: (dto.direccion as string | null) ?? mockDb.clientes[idx]!.direccion,
          telefono: (dto.telefono as string | null) ?? mockDb.clientes[idx]!.telefono,
          email: (dto.email as string | null) ?? mockDb.clientes[idx]!.email,
          condicionIva: (dto.condicionIva as string | null) ?? mockDb.clientes[idx]!.condicionIva,
          rowVersion: generateUuid(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.clientes[idx]) as T
      }
      return structuredClone(mockDb.clientes[0]) as T
    }
    case 'clientes_delete':
    case 'cliente_delete': {
      const id = String(args?.id ?? '')
      mockDb.clientes = mockDb.clientes.filter(c => c.id !== id)
      saveMockDb(mockDb)
      return null as T
    }
    case 'clientes_cuenta_corriente':
      return {
        clienteId: args?.id,
        saldo: '0.0000',
        movimientos: [],
        facturas: [],
      } as T

    // ==========================================
    // PROYECTOS
    // ==========================================
    case 'proyectos_list': {
      const query = (args?.query ?? {}) as Record<string, unknown>
      const filtro = (args?.filtro ?? query?.filtro ?? {}) as Record<string, unknown>
      let filtered = [...mockDb.proyectos]
      if (filtro.nombre && typeof filtro.nombre === 'string' && filtro.nombre.trim() !== '') {
        const needle = filtro.nombre.trim().toLowerCase()
        filtered = filtered.filter(o => o.nombre.toLowerCase().includes(needle) || o.clienteNombre.toLowerCase().includes(needle))
      }
      if (filtro.estado) {
        filtered = filtered.filter(o => o.estado === filtro.estado)
      }
      return structuredClone({ items: filtered, totalCount: filtered.length, page: 1, size: 30 }) as T
    }
    case 'proyectos_lookup':
      return structuredClone(mockDb.proyectos.map(o => ({ id: o.id, label: `${o.numero}. ${o.nombre}` }))) as T
    case 'proyectos_get':
    case 'obra_get': {
      const id = String(args?.id ?? '')
      const ob = mockDb.proyectos.find(o => o.id === id) || mockDb.proyectos[0]!
      return structuredClone({
        ...ob,
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }) as T
    }
    case 'proyectos_siguiente_numero':
      return (mockDb.proyectos.length + 1) as T
    case 'proyectos_create':
    case 'obra_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const cli = mockDb.clientes.find(c => c.id === dto.clienteId)
      const newOb: MockProyecto = {
        id: generateUuid(),
        numero: mockDb.proyectos.length + 1,
        nombre: String(dto.nombre || ''),
        direccion: (dto.direccion as string | null) ?? null,
        localidad: (dto.localidad as string | null) ?? null,
        clienteId: String(dto.clienteId || ''),
        clienteNombre: cli?.nombre ?? '',
        estado: 'Activa',
        trabajosCount: 0,
        rentabilidad: '0.0000',
        puedeEliminarse: true,
        rowVersion: generateUuid(),
      }
      mockDb.proyectos.unshift(newOb)
      saveMockDb(mockDb)
      return structuredClone(newOb) as T
    }
    case 'proyectos_update':
    case 'obra_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.proyectos.findIndex(o => o.id === id)
      if (idx >= 0) {
        const cli = mockDb.clientes.find(c => c.id === dto.clienteId)
        mockDb.proyectos[idx] = {
          ...mockDb.proyectos[idx]!,
          nombre: String(dto.nombre || mockDb.proyectos[idx]!.nombre),
          direccion: (dto.direccion as string | null) ?? mockDb.proyectos[idx]!.direccion,
          localidad: (dto.localidad as string | null) ?? mockDb.proyectos[idx]!.localidad,
          clienteId: String(dto.clienteId || mockDb.proyectos[idx]!.clienteId),
          clienteNombre: cli?.nombre ?? mockDb.proyectos[idx]!.clienteNombre,
          rowVersion: generateUuid(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.proyectos[idx]) as T
      }
      return structuredClone(mockDb.proyectos[0]) as T
    }
    case 'proyectos_transition': {
      const id = String(args?.id ?? '')
      const nuevoEstado = String(args?.nuevoEstado ?? 'Activa')
      const idx = mockDb.proyectos.findIndex(o => o.id === id)
      if (idx >= 0) {
        mockDb.proyectos[idx]!.estado = nuevoEstado
        saveMockDb(mockDb)
        return structuredClone(mockDb.proyectos[idx]) as T
      }
      return structuredClone(mockDb.proyectos[0]) as T
    }
    case 'proyectos_delete':
    case 'obra_delete': {
      const id = String(args?.id ?? '')
      mockDb.proyectos = mockDb.proyectos.filter(o => o.id !== id)
      saveMockDb(mockDb)
      return null as T
    }
    case 'obras_next_numero':
      return (mockDb.proyectos.length + 1) as T

    // ==========================================
    // TRABAJOS
    // ==========================================
    case 'trabajos_list': {
      const query = (args?.query ?? {}) as Record<string, unknown>
      const filtro = (args?.filtro ?? query?.filtro ?? {}) as Record<string, unknown>
      let filtered = [...mockDb.trabajos]
      if (filtro.descripcion && typeof filtro.descripcion === 'string' && filtro.descripcion.trim() !== '') {
        const needle = filtro.descripcion.trim().toLowerCase()
        filtered = filtered.filter(t => t.descripcion.toLowerCase().includes(needle) || t.proyectoNombre.toLowerCase().includes(needle))
      }
      if (filtro.estado) {
        filtered = filtered.filter(t => t.estado === filtro.estado)
      }
      return structuredClone({ items: filtered, totalCount: filtered.length, page: 1, size: 30 }) as T
    }
    case 'trabajos_lookup':
      return structuredClone(mockDb.trabajos.map(t => ({ id: t.id, label: t.descripcion }))) as T
    case 'trabajos_get':
    case 'trabajo_get': {
      const id = String(args?.id ?? '')
      const trab = mockDb.trabajos.find(t => t.id === id) || mockDb.trabajos[0]!
      return structuredClone({
        ...trab,
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }) as T
    }
    case 'trabajos_create':
    case 'trabajo_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const ob = mockDb.proyectos.find(o => o.id === dto.proyectoId)
      const newTrab: MockTrabajo = {
        id: generateUuid(),
        proyectoId: String(dto.proyectoId || ''),
        proyectoNumero: ob?.numero ?? 1,
        proyectoNombre: ob?.nombre ?? '',
        clienteId: ob?.clienteId ?? '',
        clienteNombre: ob?.clienteNombre ?? '',
        descripcion: String(dto.descripcion || ''),
        fechaInicio: String(dto.fechaInicio || new Date().toISOString().split('T')[0]),
        fechaFin: (dto.fechaFin as string | null) ?? null,
        presupuesto: String(dto.presupuesto || '0.0000'),
        estado: 'EnProceso',
        rowVersion: generateUuid(),
      }
      mockDb.trabajos.unshift(newTrab)
      saveMockDb(mockDb)
      return structuredClone(newTrab) as T
    }
    case 'trabajos_update':
    case 'trabajo_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.trabajos.findIndex(t => t.id === id)
      if (idx >= 0) {
        mockDb.trabajos[idx] = {
          ...mockDb.trabajos[idx]!,
          descripcion: String(dto.descripcion || mockDb.trabajos[idx]!.descripcion),
          fechaInicio: String(dto.fechaInicio || mockDb.trabajos[idx]!.fechaInicio),
          fechaFin: (dto.fechaFin as string | null) ?? mockDb.trabajos[idx]!.fechaFin,
          presupuesto: String(dto.presupuesto || mockDb.trabajos[idx]!.presupuesto),
          rowVersion: generateUuid(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.trabajos[idx]) as T
      }
      return structuredClone(mockDb.trabajos[0]) as T
    }
    case 'trabajos_transition': {
      const id = String(args?.id ?? '')
      const nuevoEstado = String(args?.nuevoEstado ?? 'EnProceso')
      const idx = mockDb.trabajos.findIndex(t => t.id === id)
      if (idx >= 0) {
        mockDb.trabajos[idx]!.estado = nuevoEstado
        saveMockDb(mockDb)
        return structuredClone(mockDb.trabajos[idx]) as T
      }
      return structuredClone(mockDb.trabajos[0]) as T
    }
    case 'trabajos_delete':
    case 'trabajo_delete': {
      const id = String(args?.id ?? '')
      mockDb.trabajos = mockDb.trabajos.filter(t => t.id !== id)
      saveMockDb(mockDb)
      return null as T
    }

    // ==========================================
    // ORDENES DE TRABAJO
    // ==========================================
    case 'ordenes_trabajo_list': {
      const trabajoId = String(args?.trabajoId ?? '')
      let filtered = [...mockDb.ordenes]
      if (trabajoId) filtered = filtered.filter(o => o.trabajoId === trabajoId)
      return structuredClone(filtered) as T
    }
    case 'ordenes_trabajo_get': {
      const id = String(args?.id ?? '')
      const ord = mockDb.ordenes.find(o => o.id === id) || mockDb.ordenes[0]!
      const dummyItem = { id: generateUuid(), descripcion: 'Item QA', unidad: 'u', cantidad: '10.0000', precioUnitario: '10000.0000', porcentajeAnterior: '0.0000', porcentajeActual: '0.0000', porcentajeAcumulado: '0.0000', porcentajePendiente: '100.0000', base: '100000.0000', subtotalActual: '0.0000', subtotalAcumulado: '0.0000', ejecutado: false, nota: null, orden: 1, certificado: false }
      return structuredClone({ ...ord, trabajoDescripcion: ord.titulo, proyectoId: '', proyectoNumero: 1, proyectoNombre: '', clienteId: '', clienteNombre: '', observaciones: null, ajusteUocraPorcentaje: '0.0000', otrosDescuentos: '0.0000', items: [dummyItem], totalPresupuestado: '100000.0000', totalCertificado: '0.0000', ajusteUocra: '0.0000', totalNeto: ord.totalNeto, certificadosCount: 0, puedeEliminarse: true, audit: mockAudit(ord.rowVersion) }) as T
    }
    case 'ordenes_trabajo_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      if (typeof dto.titulo !== 'string' || !String(dto.titulo).trim()) throw { code: 'VALIDATION', messageKey: 'Validation.OrdenTrabajo.TituloRequired', params: {}, fields: [{ field: 'titulo', messageKey: 'Validation.OrdenTrabajo.TituloRequired', params: {} }], traceId: 'preview-validation' } as ApiError
      const newOrden: MockOrden = {
        id: generateUuid(),
        trabajoId: String(dto.trabajoId ?? ''),
        titulo: String(dto.titulo ?? ''),
        numeroCertificado: null,
        fecha: String(dto.fecha ?? new Date().toISOString().split('T')[0]),
        totalCertificados: 0,
        totalNeto: '0.0000',
        rowVersion: generateUuid(),
      }
      mockDb.ordenes.unshift(newOrden)
      saveMockDb(mockDb)
      return structuredClone({ ...newOrden, audit: mockAudit(newOrden.rowVersion), items: [] }) as T
    }
    case 'ordenes_trabajo_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.ordenes.findIndex(o => o.id === id)
      if (idx >= 0) {
        mockDb.ordenes[idx] = { ...mockDb.ordenes[idx]!, titulo: String(dto.titulo ?? mockDb.ordenes[idx]!.titulo), fecha: String(dto.fecha ?? mockDb.ordenes[idx]!.fecha), rowVersion: generateUuid() }
        saveMockDb(mockDb)
        return structuredClone({ ...mockDb.ordenes[idx]!, audit: mockAudit(mockDb.ordenes[idx]!.rowVersion) }) as T
      }
      return structuredClone(mockDb.ordenes[0]) as T
    }
    case 'ordenes_trabajo_delete': {
      const id = String(args?.id ?? '')
      mockDb.ordenes = mockDb.ordenes.filter(o => o.id !== id)
      saveMockDb(mockDb)
      return null as T
    }
    case 'ordenes_trabajo_lookup':
      return structuredClone(mockDb.ordenes.map(o => ({ id: o.id, label: o.titulo }))) as T

    // ==========================================
    // EMPLEADOS
    // ==========================================
    case 'empleados_list': {
      const query = (args?.query ?? {}) as Record<string, unknown>
      const filtro = (args?.filtro ?? query?.filtro ?? {}) as Record<string, unknown>
      let filtered = [...mockDb.empleados]
      if (filtro.nombre && typeof filtro.nombre === 'string' && filtro.nombre.trim() !== '') {
        const needle = filtro.nombre.trim().toLowerCase()
        filtered = filtered.filter(e => e.nombre.toLowerCase().includes(needle) || (e.dni && e.dni.includes(needle)))
      }
      return structuredClone({ items: filtered, totalCount: filtered.length, page: 1, size: 30 }) as T
    }
    case 'empleados_lookup':
      return structuredClone(mockDb.empleados.map(e => ({ id: e.id, label: `${e.nombre} (${e.cargo ?? ''})` }))) as T
    case 'empleados_cargos':
      return structuredClone(Array.from(new Set(mockDb.empleados.map(e => e.cargo).filter(Boolean)))) as T
    case 'empleados_get':
    case 'empleado_get': {
      const id = String(args?.id ?? '')
      const emp = mockDb.empleados.find(e => e.id === id) || mockDb.empleados[0]!
      return structuredClone({
        ...emp,
        multiplicadorSabado: '1.5000',
        multiplicadorDomingo: '2.0000',
        multiplicadorFeriado: '2.0000',
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }) as T
    }
    case 'empleados_create':
    case 'empleado_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newEmp: MockEmpleado = {
        id: generateUuid(),
        nombre: String(dto.nombre || ''),
        dni: (dto.dni as string | null) ?? null,
        cargo: (dto.cargo as string | null) ?? null,
        tarifaDiaria: String(dto.tarifaDiaria || '0.0000'),
        sueldoBase: String(dto.sueldoBase || '0.0000'),
        pagoFrecuencia: String(dto.pagoFrecuencia || 'Quincenal'),
        email: (dto.email as string | null) ?? null,
        telefono: (dto.telefono as string | null) ?? null,
        fechaIngreso: String(dto.fechaIngreso || new Date().toISOString().split('T')[0]),
        fechaEgreso: (dto.fechaEgreso as string | null) ?? null,
        activo: Boolean(dto.activo ?? true),
        rowVersion: generateUuid(),
      }
      mockDb.empleados.unshift(newEmp)
      saveMockDb(mockDb)
      return structuredClone(newEmp) as T
    }
    case 'empleados_update':
    case 'empleado_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.empleados.findIndex(e => e.id === id)
      if (idx >= 0) {
        mockDb.empleados[idx] = {
          ...mockDb.empleados[idx]!,
          nombre: String(dto.nombre || mockDb.empleados[idx]!.nombre),
          dni: (dto.dni as string | null) ?? mockDb.empleados[idx]!.dni,
          cargo: (dto.cargo as string | null) ?? mockDb.empleados[idx]!.cargo,
          tarifaDiaria: String(dto.tarifaDiaria || mockDb.empleados[idx]!.tarifaDiaria),
          sueldoBase: String(dto.sueldoBase || mockDb.empleados[idx]!.sueldoBase),
          pagoFrecuencia: String(dto.pagoFrecuencia || mockDb.empleados[idx]!.pagoFrecuencia),
          email: (dto.email as string | null) ?? mockDb.empleados[idx]!.email,
          telefono: (dto.telefono as string | null) ?? mockDb.empleados[idx]!.telefono,
          fechaIngreso: String(dto.fechaIngreso || mockDb.empleados[idx]!.fechaIngreso),
          fechaEgreso: (dto.fechaEgreso as string | null) ?? mockDb.empleados[idx]!.fechaEgreso,
          activo: Boolean(dto.activo ?? mockDb.empleados[idx]!.activo),
          rowVersion: generateUuid(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.empleados[idx]) as T
      }
      return structuredClone(mockDb.empleados[0]) as T
    }
    case 'empleados_delete':
    case 'empleado_delete': {
      const id = String(args?.id ?? '')
      mockDb.empleados = mockDb.empleados.filter(e => e.id !== id)
      saveMockDb(mockDb)
      return null as T
    }
    case 'asistencias_mes':
      return { asistencias: [], totalDiasTrabajados: '11.0000' } as T
    case 'asistencias_upsert':
    case 'asistencias_bulk':
      return { creadas: 10 } as T

    // ==========================================
    // FACTURAS
    // ==========================================
    case 'facturas_list': {
      const query = (args?.query ?? {}) as Record<string, unknown>
      const filtro = (args?.filtro ?? query?.filtro ?? {}) as Record<string, unknown>
      let filtered = [...mockDb.facturas]
      if (filtro.numero && typeof filtro.numero === 'string' && filtro.numero.trim() !== '') {
        const needle = filtro.numero.trim().toLowerCase()
        filtered = filtered.filter(f => f.numero.toLowerCase().includes(needle) || f.clienteNombre.toLowerCase().includes(needle))
      }
      if (filtro.estado) {
        filtered = filtered.filter(f => f.estado === filtro.estado)
      }
      return structuredClone({ items: filtered, totalCount: filtered.length, page: 1, size: 30 }) as T
    }
    case 'facturas_get':
    case 'factura_get': {
      const id = String(args?.id ?? '')
      const fact = mockDb.facturas.find(f => f.id === id) || mockDb.facturas[0]!
      return structuredClone({
        ...fact,
        observaciones: 'Facturación de proyecto',
        items: [],
        pagos: [],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }) as T
    }
    case 'facturas_create':
    case 'factura_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const cli = mockDb.clientes.find(c => c.id === dto.clienteId)
      const subNum = parseFloat(String(dto.subtotal || '0'))
      const ivaNum = parseFloat(String(dto.iva || '0'))
      const totNum = subNum + ivaNum
      const newFact: MockFactura = {
        id: generateUuid(),
        numero: String(dto.numero || `0001-0000010${mockDb.facturas.length + 1}`),
        fecha: String(dto.fecha || new Date().toISOString().split('T')[0]),
        fechaVencimiento: (dto.fechaVencimiento as string | null) ?? null,
        clienteId: String(dto.clienteId || ''),
        clienteNombre: cli?.nombre ?? '',
        estado: 'Borrador',
        subtotal: subNum.toFixed(4),
        iva: ivaNum.toFixed(4),
        total: totNum.toFixed(4),
        saldoPendiente: totNum.toFixed(4),
        rowVersion: generateUuid(),
      }
      mockDb.facturas.unshift(newFact)
      saveMockDb(mockDb)
      return structuredClone(newFact) as T
    }
    case 'facturas_update':
    case 'factura_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.facturas.findIndex(f => f.id === id)
      if (idx >= 0) {
        const subNum = parseFloat(String(dto.subtotal !== undefined ? dto.subtotal : mockDb.facturas[idx]!.subtotal))
        const ivaNum = parseFloat(String(dto.iva !== undefined ? dto.iva : mockDb.facturas[idx]!.iva))
        const totNum = subNum + ivaNum
        mockDb.facturas[idx] = {
          ...mockDb.facturas[idx]!,
          numero: String(dto.numero || mockDb.facturas[idx]!.numero),
          fecha: String(dto.fecha || mockDb.facturas[idx]!.fecha),
          fechaVencimiento: (dto.fechaVencimiento as string | null) ?? mockDb.facturas[idx]!.fechaVencimiento,
          subtotal: subNum.toFixed(4),
          iva: ivaNum.toFixed(4),
          total: totNum.toFixed(4),
          rowVersion: generateUuid(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.facturas[idx]) as T
      }
      return structuredClone(mockDb.facturas[0]) as T
    }
    case 'facturas_transition': {
      const id = String(args?.id ?? '')
      const nuevoEstado = String(args?.nuevoEstado ?? 'Emitida')
      const idx = mockDb.facturas.findIndex(f => f.id === id)
      if (idx >= 0) {
        mockDb.facturas[idx]!.estado = nuevoEstado
        saveMockDb(mockDb)
        return structuredClone(mockDb.facturas[idx]) as T
      }
      return structuredClone(mockDb.facturas[0]) as T
    }
    case 'facturas_delete':
    case 'factura_delete': {
      const id = String(args?.id ?? '')
      mockDb.facturas = mockDb.facturas.filter(f => f.id !== id)
      saveMockDb(mockDb)
      return null as T
    }
    case 'pagos_factura_registrar': {
      const id = String(args?.facturaId ?? '')
      const idx = mockDb.facturas.findIndex(f => f.id === id)
      if (idx >= 0) {
        mockDb.facturas[idx]!.estado = 'Pagada'
        mockDb.facturas[idx]!.saldoPendiente = '0.0000'
        saveMockDb(mockDb)
      }
      return { id: generateUuid(), facturaId: id, monto: '1000.0000', fecha: new Date().toISOString().split('T')[0], medioPago: 'Transferencia', rowVersion: 'v1' } as T
    }
    case 'pagos_factura_eliminar':
      return null as T

    // ==========================================
    // CERTIFICADOS
    // ==========================================
    case 'certificados_list': {
      return structuredClone({ items: mockDb.certificados, totalCount: mockDb.certificados.length, page: 1, size: 30 }) as T
    }
    case 'certificados_get':
    case 'certificado_get': {
      const id = String(args?.id ?? '')
      const cert = mockDb.certificados.find(c => c.id === id) || mockDb.certificados[0]!
      return structuredClone({
        ...cert,
        ajusteUocra: '148000.0000',
        otrosDescuentos: '0.0000',
        observaciones: 'Certificado aprobado',
        items: [],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }) as T
    }
    case 'certificados_borrador':
      return structuredClone({
        ordenTrabajoId: mockDb.ordenes[0]?.id ?? '',
        ordenTitulo: mockDb.ordenes[0]?.titulo ?? '',
        numeroSugerido: 1,
        trabajoDescripcion: mockDb.trabajos[0]?.descripcion ?? '',
        proyectoNombre: mockDb.proyectos[0]?.nombre ?? '',
        clienteNombre: mockDb.clientes[2]?.nombre ?? '',
        ajusteUocraPorcentaje: '8.0000',
        otrosDescuentos: '0.0000',
        items: [],
      }) as T
    case 'certificados_emitir': {
      const newCert: MockCertificado = {
        id: generateUuid(),
        ordenTrabajoId: mockDb.ordenes[0]?.id ?? '',
        ordenTitulo: mockDb.ordenes[0]?.titulo ?? '',
        numero: mockDb.certificados.length + 1,
        fecha: new Date().toISOString().split('T')[0] ?? '',
        totalCertificado: '1500000.0000',
        totalNeto: '1620000.0000',
        rowVersion: generateUuid(),
      }
      mockDb.certificados.unshift(newCert)
      saveMockDb(mockDb)
      return structuredClone(newCert) as T
    }
    case 'certificados_anular': {
      const id = String(args?.id ?? '')
      mockDb.certificados = mockDb.certificados.filter(c => c.id !== id)
      saveMockDb(mockDb)
      return null as T
    }

    // ==========================================
    // LIQUIDACIONES
    // ==========================================
    case 'liquidaciones_list': {
      return structuredClone({ items: mockDb.liquidaciones, totalCount: mockDb.liquidaciones.length, page: 1, size: 30 }) as T
    }
    case 'liquidaciones_get':
    case 'liquidacion_get': {
      const id = String(args?.id ?? '')
      const liq = mockDb.liquidaciones.find(l => l.id === id) || mockDb.liquidaciones[0]!
      return structuredClone({
        ...liq,
        incluirSabados: true,
        incluirDomingos: false,
        incluirFeriados: false,
        multiplicadorSabado: '1.5000',
        multiplicadorDomingo: '2.0000',
        multiplicadorFeriado: '2.0000',
        observaciones: 'Liquidación quincenal',
        desglose: {
          jornadasCompletas: '11.0000',
          jornadasMedias: '0.0000',
          faltas: 0,
          faltasJustificadas: 0,
          diasSabado: '1.0000',
          diasDomingo: '0.0000',
          diasFeriado: '0.0000',
          multiplicadorSabado: '1.5000',
          multiplicadorDomingo: '2.0000',
          multiplicadorFeriado: '2.0000',
          recargos: '22500.0000',
        },
        adelantos: [],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }) as T
    }
    case 'liquidaciones_sugerir':
      return structuredClone([
        {
          empleadoId: mockDb.empleados[0]?.id ?? '',
          empleadoNombre: mockDb.empleados[0]?.nombre ?? '',
          cargo: mockDb.empleados[0]?.cargo ?? '',
          tarifaDiaria: mockDb.empleados[0]?.tarifaDiaria ?? '45000.0000',
          diasSugeridos: '11.0000',
          origen: 'Asistencia',
          desglose: {
            jornadasCompletas: '11.0000',
            jornadasMedias: '0.0000',
            faltas: 0,
            faltasJustificadas: 0,
            diasSabado: '1.0000',
            diasDomingo: '0.0000',
            diasFeriado: '0.0000',
            multiplicadorSabado: '1.5000',
            multiplicadorDomingo: '2.0000',
            multiplicadorFeriado: '2.0000',
            recargos: '22500.0000',
          },
          adelantos: [],
          totalBruto: '495000.0000',
          totalAdelantos: '0.0000',
          totalNeto: '495000.0000',
        },
      ]) as T
    case 'liquidaciones_emitir': {
      const newLiq: MockLiquidacion = {
        id: generateUuid(),
        empleadoId: mockDb.empleados[0]?.id ?? '',
        empleadoNombre: mockDb.empleados[0]?.nombre ?? '',
        empleadoCargo: mockDb.empleados[0]?.cargo ?? '',
        fechaInicio: new Date().toISOString().split('T')[0] ?? '',
        fechaFin: new Date().toISOString().split('T')[0] ?? '',
        diasTrabajados: '11.0000',
        tarifaAplicada: '45000.0000',
        totalBruto: '495000.0000',
        totalAdelantos: '0.0000',
        totalNeto: '495000.0000',
        tienePdf: false,
        rowVersion: generateUuid(),
      }
      mockDb.liquidaciones.unshift(newLiq)
      saveMockDb(mockDb)
      return structuredClone([newLiq]) as T
    }
    case 'liquidaciones_delete':
    case 'liquidacion_delete': {
      const id = String(args?.id ?? '')
      mockDb.liquidaciones = mockDb.liquidaciones.filter(l => l.id !== id)
      saveMockDb(mockDb)
      return null as T
    }

    // ==========================================
    // CATEGORIAS & TIPOS
    // ==========================================
    case 'categorias_list': {
      return structuredClone({ items: mockDb.categorias, totalCount: mockDb.categorias.length, page: 1, size: 30 }) as T
    }
    case 'categorias_lookup':
      return structuredClone(mockDb.categorias.map(c => ({ id: c.id, label: c.nombre }))) as T
    case 'categorias_get': {
      const id = String(args?.id ?? '')
      const cat = mockDb.categorias.find(c => c.id === id) || mockDb.categorias[0]!
      return structuredClone({ ...cat, createdAt: new Date().toISOString(), updatedAt: null }) as T
    }
    case 'categorias_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newCat: MockCategory = {
        id: generateUuid(),
        nombre: String(dto.nombre || ''),
        descripcion: (dto.descripcion as string | null) ?? null,
        colorHex: (dto.colorHex as string | null) ?? '#3B82F6',
        icono: (dto.icono as string | null) ?? 'package',
        categoriaPadreId: (dto.categoriaPadreId as string | null) ?? null,
        categoriaPadreNombre: null,
        nivel: 0,
        movimientosCount: 0,
        subcategoriasCount: 0,
        puedeEliminarse: true,
        rowVersion: generateUuid(),
      }
      mockDb.categorias.unshift(newCat)
      saveMockDb(mockDb)
      return structuredClone(newCat) as T
    }
    case 'categorias_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.categorias.findIndex(c => c.id === id)
      if (idx >= 0) {
        mockDb.categorias[idx] = {
          ...mockDb.categorias[idx]!,
          nombre: String(dto.nombre || mockDb.categorias[idx]!.nombre),
          descripcion: (dto.descripcion as string | null) ?? mockDb.categorias[idx]!.descripcion,
          colorHex: (dto.colorHex as string | null) ?? mockDb.categorias[idx]!.colorHex,
          icono: (dto.icono as string | null) ?? mockDb.categorias[idx]!.icono,
          categoriaPadreId: (dto.categoriaPadreId as string | null) ?? mockDb.categorias[idx]!.categoriaPadreId,
          rowVersion: generateUuid(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.categorias[idx]) as T
      }
      return structuredClone(mockDb.categorias[0]) as T
    }
    case 'categorias_delete': {
      const id = String(args?.id ?? '')
      mockDb.categorias = mockDb.categorias.filter(c => c.id !== id)
      saveMockDb(mockDb)
      return null as T
    }

    case 'tipos_movimiento_list': {
      return structuredClone({ items: mockDb.tiposMovimiento, totalCount: mockDb.tiposMovimiento.length, page: 1, size: 30 }) as T
    }
    case 'tipos_movimiento_lookup':
      return structuredClone(mockDb.tiposMovimiento.map(t => ({ id: t.id, label: t.nombre }))) as T
    case 'tipos_movimiento_get': {
      const id = String(args?.id ?? '')
      const tipo = mockDb.tiposMovimiento.find(t => t.id === id) || mockDb.tiposMovimiento[0]!
      return structuredClone({ ...tipo, createdAt: new Date().toISOString(), updatedAt: null }) as T
    }
    case 'tipos_movimiento_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newTipo: MockTipoMovimiento = {
        id: generateUuid(),
        nombre: String(dto.nombre || ''),
        descripcion: (dto.descripcion as string | null) ?? null,
        esIngreso: Boolean(dto.esIngreso ?? true),
        esSistema: false,
        movimientosCount: 0,
        puedeEliminarse: true,
        rowVersion: generateUuid(),
      }
      mockDb.tiposMovimiento.unshift(newTipo)
      saveMockDb(mockDb)
      return structuredClone(newTipo) as T
    }
    case 'tipos_movimiento_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.tiposMovimiento.findIndex(t => t.id === id)
      if (idx >= 0) {
        mockDb.tiposMovimiento[idx] = {
          ...mockDb.tiposMovimiento[idx]!,
          nombre: String(dto.nombre || mockDb.tiposMovimiento[idx]!.nombre),
          descripcion: (dto.descripcion as string | null) ?? mockDb.tiposMovimiento[idx]!.descripcion,
          rowVersion: generateUuid(),
        }
        saveMockDb(mockDb)
        return structuredClone(mockDb.tiposMovimiento[idx]) as T
      }
      return structuredClone(mockDb.tiposMovimiento[0]) as T
    }
    case 'tipos_movimiento_delete': {
      const id = String(args?.id ?? '')
      mockDb.tiposMovimiento = mockDb.tiposMovimiento.filter(t => t.id !== id)
      saveMockDb(mockDb)
      return null as T
    }

    default:
      return null as unknown as T
  }
}
