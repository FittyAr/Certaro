import type { Money } from '@/api/types'

export function parseMoneyInput(raw: string, min?: number, max?: number): Money | null {
  const trimmed = raw.trim().replace(/\s/g, '')
  if (!trimmed) return '0.0000'
  const negative = trimmed.startsWith('-')
  const unsigned = negative ? trimmed.slice(1) : trimmed
  const lastComma = unsigned.lastIndexOf(',')
  const lastDot = unsigned.lastIndexOf('.')
  const decimalIndex = Math.max(lastComma, lastDot)
  const decimalDigits = decimalIndex >= 0 ? unsigned.length - decimalIndex - 1 : 0
  if (decimalIndex >= 0 && decimalDigits > 4) return null
  const hasDecimal = decimalIndex >= 0 && decimalDigits > 0 && decimalDigits <= 4
  const integerPart = hasDecimal ? unsigned.slice(0, decimalIndex) : unsigned
  const fractionPart = hasDecimal ? unsigned.slice(decimalIndex + 1) : ''
  const integer = integerPart.replace(/[.,]/g, '')
  if (!/^\d+$/.test(integer) || !/^\d*$/.test(fractionPart)) return null
  const value = `${negative ? '-' : ''}${integer || '0'}.${fractionPart.padEnd(4, '0')}`
  const numeric = Number(value)
  if (
    !Number.isFinite(numeric) ||
    (min !== undefined && numeric < min) ||
    (max !== undefined && numeric > max)
  )
    return null
  return value
}
