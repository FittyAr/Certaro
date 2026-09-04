import {
  generateUuid,
  mockDb,
  saveMockDb,
} from '../database'
import type { MockEmpleado, MockLiquidacion } from '../types'

export function handlePersonalCommand<T>(command: string, args?: Record<string, unknown>): T | undefined {
  switch (command) {
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

    default:
      return undefined
  }
}
