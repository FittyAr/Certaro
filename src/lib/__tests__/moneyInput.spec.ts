import { describe, expect, it } from 'vitest'

import { parseMoneyInput } from '@/lib/moneyInput'

describe('parseMoneyInput', () => {
  it.each([
    ['12345.67', '12345.6700'],
    ['12345,67', '12345.6700'],
    ['12.345,67', '12345.6700'],
    ['12,345.67', '12345.6700'],
  ])('normaliza %s como %s', (input, expected) => {
    expect(parseMoneyInput(input)).toBe(expected)
  })

  it('rechaza más de cuatro decimales', () => {
    expect(parseMoneyInput('123.45678')).toBeNull()
  })
})
