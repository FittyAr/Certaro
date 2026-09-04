import type { ApiError } from '../../client'
import {
  generateUuid,
  mockAudit,
  mockDb,
  saveMockDb,
} from '../database'
import type { MockCertificado, MockOrden, MockProyecto, MockTrabajo } from '../types'

export function handleProyectosCommand<T>(command: string, args?: Record<string, unknown>): T | undefined {
  switch (command) {
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

    default:
      return undefined
  }
}
