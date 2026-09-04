import {
  generateUuid,
  mockDb,
  saveMockDb,
} from '../database'
import type { MockMovimiento } from '../types'

export function handleMovimientosCommand<T>(command: string, args?: Record<string, unknown>): T | undefined {
  switch (command) {
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
    default:
      return undefined
  }
}
