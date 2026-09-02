import { describe, expect, it } from 'vitest'

import {
  validateMockCategoria,
  validateMockCliente,
  validateMockEmpleado,
  validateMockFactura,
  validateMockMovimiento,
  validateMockProyecto,
  validateMockTipoMovimiento,
  validateMockTrabajo,
} from '@/api/client'

describe('validacion del mock', () => {
  it('rechaza crear un movimiento sin los campos obligatorios', () => {
    expect(
      validateMockMovimiento({
        concepto: '',
        monto: '0.0000',
        cantidad: '1.0000',
        tipoMovimientoId: '',
        moneda: 'Ars',
        cotizacionAplicada: null,
        categoriaId: null,
      }),
    ).toMatchObject({
      code: 'VALIDATION',
      fields: expect.arrayContaining([
        expect.objectContaining({ field: 'concepto' }),
        expect.objectContaining({ field: 'monto' }),
        expect.objectContaining({ field: 'tipoMovimientoId' }),
        expect.objectContaining({ field: 'categoriaId' }),
      ]),
    })
  })

  it('rechaza crear un cliente sin nombre', () => {
    expect(validateMockCliente({ nombre: '', email: 'correo-invalido' })).toMatchObject({
      code: 'VALIDATION',
      fields: expect.arrayContaining([expect.objectContaining({ field: 'nombre' })]),
    })
  })

  it('rechaza crear una proyecto sin cliente', () => {
    expect(validateMockProyecto({ nombre: '', clienteId: '' })).toMatchObject({ code: 'VALIDATION' })
  })
  it('rechaza crear un trabajo sin descripcion', () => {
    expect(validateMockTrabajo({ descripcion: '', proyectoId: '' })).toMatchObject({ code: 'VALIDATION' })
  })
  it('rechaza crear una factura sin cliente', () => {
    expect(validateMockFactura({ numero: '', clienteId: '' })).toMatchObject({ code: 'VALIDATION' })
  })
  it('rechaza crear un empleado sin nombre', () => {
    expect(validateMockEmpleado({ nombre: '' })).toMatchObject({ code: 'VALIDATION' })
  })
  it('rechaza crear una categoria sin nombre', () => {
    expect(validateMockCategoria({ nombre: '' })).toMatchObject({ code: 'VALIDATION' })
  })
  it('rechaza crear un tipo de movimiento sin nombre', () => {
    expect(validateMockTipoMovimiento({ nombre: '' })).toMatchObject({ code: 'VALIDATION' })
  })
  it('acepta un movimiento completo con monto positivo', () => {
    expect(
      validateMockMovimiento({
        concepto: 'Compra de materiales',
        monto: '100.0000',
        cantidad: '1.0000',
        tipoMovimientoId: 'tipo-1',
        moneda: 'Ars',
        cotizacionAplicada: null,
        categoriaId: 'categoria-1',
      }),
    ).toBeNull()
  })
})
