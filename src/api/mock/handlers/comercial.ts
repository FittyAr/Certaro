import {
  generateUuid,
  mockDb,
  saveMockDb,
} from '../database'
import type { MockCliente, MockFactura } from '../types'

export function handleComercialCommand<T>(command: string, args?: Record<string, unknown>): T | undefined {
  switch (command) {
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
    default:
      return undefined
  }
}
