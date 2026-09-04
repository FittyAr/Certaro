import type { ApiError, ApiFieldError } from '../client'

export function validateMockMovimiento(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  const add = (field: string, messageKey: string): void => {
    fields.push({ field, messageKey, params: {} })
  }
  if (typeof dto.concepto !== 'string' || !dto.concepto.trim())
    add('concepto', 'Validation.Movimiento.ConceptoRequired')
  const monto = Number(dto.monto)
  if (!Number.isFinite(monto) || monto <= 0)
    add('monto', 'Validation.Movimiento.MontoRequired')
  const cantidad = Number(dto.cantidad)
  if (!Number.isFinite(cantidad) || cantidad <= 0)
    add('cantidad', 'Validation.Movimiento.CantidadRequired')
  if (typeof dto.tipoMovimientoId !== 'string' || !dto.tipoMovimientoId.trim())
    add('tipoMovimientoId', 'Validation.Movimiento.TipoRequired')
  if (typeof dto.categoriaId !== 'string' || !dto.categoriaId.trim())
    add('categoriaId', 'Validation.Movimiento.CategoriaRequired')
  if (dto.moneda === 'Usd' && (!Number.isFinite(Number(dto.cotizacionAplicada)) || Number(dto.cotizacionAplicada) <= 0))
    add('cotizacionAplicada', 'Validation.Movimiento.CotizacionRequired')
  if (dto.moneda !== 'Usd' && dto.cotizacionAplicada !== null && dto.cotizacionAplicada !== undefined)
    add('cotizacionAplicada', 'Validation.Movimiento.CotizacionForbidden')
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockCliente(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.nombre !== 'string' || !dto.nombre.trim())
    fields.push({ field: 'nombre', messageKey: 'Validation.Cliente.NombreRequired', params: {} })
  if (typeof dto.email === 'string' && dto.email.trim() && !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(dto.email))
    fields.push({ field: 'email', messageKey: 'Validation.Cliente.EmailInvalid', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockProyecto(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.nombre !== 'string' || !dto.nombre.trim())
    fields.push({ field: 'nombre', messageKey: 'Validation.Proyecto.NombreRequired', params: {} })
  if (typeof dto.clienteId !== 'string' || !dto.clienteId.trim())
    fields.push({ field: 'clienteId', messageKey: 'Validation.Proyecto.ClienteRequired', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockTrabajo(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.descripcion !== 'string' || !dto.descripcion.trim())
    fields.push({ field: 'descripcion', messageKey: 'Validation.Trabajo.DescripcionRequired', params: {} })
  if (typeof dto.proyectoId !== 'string' || !dto.proyectoId.trim())
    fields.push({ field: 'proyectoId', messageKey: 'Validation.Trabajo.ProyectoRequired', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockFactura(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.numero !== 'string' || !dto.numero.trim())
    fields.push({ field: 'numero', messageKey: 'Validation.Factura.NumeroRequired', params: {} })
  if (typeof dto.clienteId !== 'string' || !dto.clienteId.trim())
    fields.push({ field: 'clienteId', messageKey: 'Validation.Factura.ClienteRequired', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockEmpleado(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.nombre !== 'string' || !dto.nombre.trim())
    fields.push({ field: 'nombre', messageKey: 'Validation.Empleado.NombreRequired', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockCategoria(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.nombre !== 'string' || !dto.nombre.trim())
    fields.push({ field: 'nombre', messageKey: 'Validation.Categoria.NombreRequired', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockTipoMovimiento(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.nombre !== 'string' || !dto.nombre.trim())
    fields.push({ field: 'nombre', messageKey: 'Validation.TipoMovimiento.NombreRequired', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}

export function validateMockFeriado(dto: Record<string, unknown>): ApiError | null {
  const fields: ApiFieldError[] = []
  if (typeof dto.fecha !== 'string' || !dto.fecha)
    fields.push({ field: 'fecha', messageKey: 'Validation.Feriado.FechaRequired', params: {} })
  if (typeof dto.nombre !== 'string' || !dto.nombre.trim())
    fields.push({ field: 'nombre', messageKey: 'Validation.Feriado.NombreRequired', params: {} })
  return fields.length
    ? { code: 'VALIDATION', messageKey: 'Validation.Invalid', params: {}, fields, traceId: 'preview-validation' }
    : null
}
