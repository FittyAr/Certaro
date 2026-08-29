import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { useMoney } from '@/composables/useMoney'
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
    // The composable only reads `locale`; the rest of the config is irrelevant here.
  } as never
}

describe('useMoney', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    withLocale()
  })

  it('agrupa los miles y usa los separadores configurados', () => {
    expect(useMoney().format('1234567.8900')).toBe('$ 1.234.567,89')
  })

  it('el signo va delante del simbolo, como en la convencion local', () => {
    expect(useMoney().format('-1500.0000')).toBe('-$ 1.500,00')
  })

  it('no muestra signo mas salvo que se pida', () => {
    const { format } = useMoney()
    expect(format('10.0000')).toBe('$ 10,00')
    expect(format('10.0000', { showSign: true })).toBe('+$ 10,00')
  })

  it('redondea half away from zero al recortar decimales', () => {
    const { format } = useMoney()
    expect(format('0.1250')).toBe('$ 0,13')
    expect(format('0.1240')).toBe('$ 0,12')
    expect(format('-0.1250')).toBe('-$ 0,13')
  })

  it('arrastra el acarreo hasta la parte entera', () => {
    expect(useMoney().format('9.9950')).toBe('$ 10,00')
  })

  it('sin decimales visibles no imprime el separador decimal', () => {
    withLocale({ decimalesMoneda: 0 })
    expect(useMoney().format('1234.5600')).toBe('$ 1.235')
  })

  it('un valor vacio o ausente se formatea como cadena vacia', () => {
    const { format } = useMoney()
    expect(format(null)).toBe('')
    expect(format(undefined)).toBe('')
    expect(format('')).toBe('')
  })

  it('el porcentaje lleva su propio numero de decimales', () => {
    withLocale({ decimalesPorcentaje: 1 })
    expect(useMoney().formatPercent('21.0000')).toBe('21,0 %')
  })

  it('la ida y vuelta por el widget conserva el valor', () => {
    const { toInputValue, fromInputValue } = useMoney()
    expect(fromInputValue(toInputValue('1234.5600'))).toBe('1234.5600')
  })

  it('un numero del widget siempre vuelve con cuatro decimales', () => {
    expect(useMoney().fromInputValue(12.5)).toBe('12.5000')
  })

  it('detecta el signo y el cero sin convertir a numero', () => {
    const { isNegative, isZero } = useMoney()
    expect(isNegative('-0.0001')).toBe(true)
    expect(isNegative('0.0001')).toBe(false)
    expect(isZero('0.0000')).toBe(true)
    expect(isZero('-0.0000')).toBe(true)
    expect(isZero('0.0001')).toBe(false)
  })
})
