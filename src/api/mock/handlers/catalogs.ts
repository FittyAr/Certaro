import {
  generateUuid,
  mockDb,
  saveMockDb,
} from '../database'
import type { MockCategory, MockTipoMovimiento } from '../types'

export function handleCatalogsCommand<T>(command: string, args?: Record<string, unknown>): T | undefined {
  switch (command) {
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
      return undefined
  }
}
