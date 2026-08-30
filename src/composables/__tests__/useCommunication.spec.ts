import { describe, expect, it } from 'vitest'

import { normalizarTelefono } from '@/composables/useCommunication'

/** Behaviour required by `docs/13-servicios-externos-y-archivos.md` §7.2. */

describe('normalizarTelefono', () => {
  it('un telefono con formato produce solo digitos con el codigo de pais', () => {
    expect(normalizarTelefono('(011) 4567-8901', '54')).toBe('5401145678901')
  })

  it('un telefono que ya tiene el codigo de pais no lo duplica', () => {
    expect(normalizarTelefono('+54 11 4567-8901', '54')).toBe('541145678901')
  })

  it('un telefono vacio devuelve vacio', () => {
    expect(normalizarTelefono('', '54')).toBe('')
  })

  it('un telefono con solo espacios devuelve vacio', () => {
    expect(normalizarTelefono('   ', '54')).toBe('')
  })

  it('un telefono sin formato se prefija con el codigo', () => {
    expect(normalizarTelefono('1145678901', '54')).toBe('541145678901')
  })

  it('el codigo de pais puede ser otro', () => {
    expect(normalizarTelefono('(234) 567-8901', '1')).toBe('12345678901')
  })
})
