import { computed } from 'vue'

import type { Decimal4, Money } from '@/api/types'
import { useConfigStore } from '@/stores/useConfigStore'

/**
 * Formatting and parsing of amounts. See `docs/16-frontend.md` §4.2.
 *
 * A `Money` is a string with four decimals for its whole life in the frontend. It is only ever a
 * `number` inside {@link toInputValue} and {@link fromInputValue}, which exist because PrimeVue's
 * `InputNumber` works with numbers. Outside that boundary, doing arithmetic on an amount is a bug:
 * `Number` is IEEE-754 and loses centavos, and a screen total that disagrees with the PDF costs
 * the user's trust in the whole system.
 */

export interface FormatMoneyOptions {
  /** Prefixes a `+` on positive values. Off by default. */
  showSign?: boolean
  /** Drops the currency symbol, for columns that carry it in the header. */
  hideSymbol?: boolean
  /** Overrides the configured number of visible decimals. */
  decimals?: number
}

/** Splits a four-decimal string without going through a float. */
function split(raw: string): { negative: boolean; integer: string; fraction: string } {
  const trimmed = (raw ?? '').trim()
  const negative = trimmed.startsWith('-')
  const unsigned = negative ? trimmed.slice(1) : trimmed
  const [integer = '0', fraction = ''] = unsigned.split('.')
  return { negative, integer: integer === '' ? '0' : integer, fraction: fraction.padEnd(4, '0') }
}

/** Rounds the decimal string to `decimals` places, half-away-from-zero, still as text. */
function roundDecimals(integer: string, fraction: string, decimals: number): [string, string] {
  if (decimals >= fraction.length) return [integer, fraction.padEnd(decimals, '0')]

  const keep = fraction.slice(0, decimals)
  const nextDigit = Number(fraction[decimals] ?? '0')
  if (nextDigit < 5) return [integer, keep]

  // Increment the kept digits as an integer, carrying into the integer part when they overflow.
  const bumped = (BigInt(keep === '' ? '0' : keep) + 1n).toString()
  if (bumped.length > decimals) {
    return [(BigInt(integer) + 1n).toString(), '0'.repeat(decimals)]
  }
  return [integer, bumped.padStart(decimals, '0')]
}

function groupThousands(integer: string, separator: string): string {
  return integer.replace(/\B(?=(\d{3})+(?!\d))/g, separator)
}

export function useMoney() {
  const config = useConfigStore()
  const locale = computed(() => config.config?.locale)

  function format(raw: Money | null | undefined, opts: FormatMoneyOptions = {}): string {
    if (raw === null || raw === undefined || raw === '') return ''

    const l = locale.value
    const decimals = opts.decimals ?? l?.decimalesMoneda ?? 2
    const thousands = l?.separadorMiles ?? '.'
    const decimalMark = l?.separadorDecimal ?? ','
    const symbol = l?.simboloMoneda ?? '$'

    const { negative, integer, fraction } = split(raw)
    const [roundedInteger, roundedFraction] = roundDecimals(integer, fraction, decimals)

    const body =
      decimals > 0
        ? `${groupThousands(roundedInteger, thousands)}${decimalMark}${roundedFraction}`
        : groupThousands(roundedInteger, thousands)

    const sign = negative ? '-' : opts.showSign ? '+' : ''
    return opts.hideSymbol ? `${sign}${body}` : `${sign}${symbol} ${body}`
  }

  function formatPercent(raw: Decimal4 | null | undefined): string {
    if (raw === null || raw === undefined || raw === '') return ''
    const decimals = locale.value?.decimalesPorcentaje ?? 2
    const decimalMark = locale.value?.separadorDecimal ?? ','
    const { negative, integer, fraction } = split(raw)
    const [roundedInteger, roundedFraction] = roundDecimals(integer, fraction, decimals)
    const body = decimals > 0 ? `${roundedInteger}${decimalMark}${roundedFraction}` : roundedInteger
    return `${negative ? '-' : ''}${body} %`
  }

  /** `"12345.6700"` to `12345.67`. Only to hand a value to `InputNumber`. */
  function toInputValue(raw: Money | null | undefined): number {
    if (raw === null || raw === undefined || raw === '') return 0
    const parsed = Number(raw)
    return Number.isFinite(parsed) ? parsed : 0
  }

  /** `12345.67` to `"12345.6700"`. Only to send a value to the backend. */
  function fromInputValue(value: number | null | undefined): Money {
    if (value === null || value === undefined || !Number.isFinite(value)) return '0.0000'
    // `toFixed(4)` is exact here: the input widget cannot produce more than the configured number
    // of visible decimals, which is never more than four.
    return value.toFixed(4)
  }

  /** True when the amount is below zero, without parsing it as a number. */
  function isNegative(raw: Money | null | undefined): boolean {
    return typeof raw === 'string' && raw.trim().startsWith('-')
  }

  /** True when every digit is zero. */
  function isZero(raw: Money | null | undefined): boolean {
    if (raw === null || raw === undefined || raw === '') return true
    return /^-?0*\.?0*$/.test(raw.trim())
  }

  return {
    format,
    formatPercent,
    toInputValue,
    fromInputValue,
    parse: fromInputValue,
    isNegative,
    isZero,
  }
}
