import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { useDateFormat } from '@/composables/useDateFormat'
import { useConfigStore } from '@/stores/useConfigStore'

function withLocale(overrides: Record<string, unknown> = {}) {
  const config = useConfigStore()
  config.config = {
    locale: {
      language: 'es',
      formatoFecha: 'dd/MM/yyyy',
      formatoFechaHora: 'dd/MM/yyyy HH:mm',
      primerDiaSemana: 1,
      simboloMoneda: '$',
      separadorMiles: '.',
      separadorDecimal: ',',
      decimalesMoneda: 2,
      decimalesPorcentaje: 2,
      monedaPorDefecto: 'ARS',
      zonaHoraria: 'America/Argentina/Buenos_Aires',
      ...overrides,
    },
  } as never
}

describe('useDateFormat', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    withLocale()
  })

  it('una fecha civil se muestra tal cual, sin corrimiento de zona', () => {
    expect(useDateFormat().formatCivil('2026-01-01')).toBe('01/01/2026')
  })

  it('la fecha civil respeta el patron configurado', () => {
    withLocale({ formatoFecha: 'yyyy-MM-dd' })
    expect(useDateFormat().formatCivil('2026-08-29')).toBe('2026-08-29')
  })

  it('la ida y vuelta de una fecha civil por el widget no cambia el dia', () => {
    const { civilToDate, dateToCivil } = useDateFormat()
    expect(dateToCivil(civilToDate('2026-01-01'))).toBe('2026-01-01')
    expect(dateToCivil(civilToDate('2026-12-31'))).toBe('2026-12-31')
  })

  it('un instante se muestra en hora local', () => {
    const iso = '2026-08-29T15:30:00.000Z'
    const local = new Date(iso)
    const expected = `${String(local.getDate()).padStart(2, '0')}/${String(
      local.getMonth() + 1,
    ).padStart(2, '0')}/${local.getFullYear()} ${String(local.getHours()).padStart(
      2,
      '0',
    )}:${String(local.getMinutes()).padStart(2, '0')}`
    expect(useDateFormat().formatInstant(iso)).toBe(expected)
  })

  it('un instante sin hora usa el formato de fecha', () => {
    const value = useDateFormat().formatInstant('2026-08-29T15:30:00.000Z', false)
    expect(value).not.toContain(':')
  })

  it('la ida y vuelta de un instante conserva el milisegundo', () => {
    const { instantToDate, dateToInstant } = useDateFormat()
    const iso = '2026-08-29T15:30:00.123Z'
    expect(dateToInstant(instantToDate(iso))).toBe(iso)
  })

  it('un valor ausente se formatea como cadena vacia', () => {
    const { formatCivil, formatInstant } = useDateFormat()
    expect(formatCivil(null)).toBe('')
    expect(formatInstant(undefined)).toBe('')
  })

  it('un valor ilegible se devuelve sin tocar en lugar de romper la vista', () => {
    expect(useDateFormat().formatInstant('no es una fecha')).toBe('no es una fecha')
  })
})
