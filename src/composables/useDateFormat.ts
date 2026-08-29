import { computed } from 'vue'

import type { CivilDate, Instant } from '@/api/types'
import { useConfigStore } from '@/stores/useConfigStore'

/**
 * Date formatting and parsing. See `docs/16-frontend.md` §4.3 and `docs/04-dinero-fechas-y-tipos.md` §3.
 *
 * A date format never lives in a locale file: the format is configuration, not translation, so a
 * user can run the interface in English with Argentine dates.
 *
 * The two kinds of value are handled separately on purpose:
 *
 * - a **civil date** (`YYYY-MM-DD`) is a calendar day and is never converted to a timezone;
 *   converting it would shift it back three hours and land on the previous day;
 * - an **instant** is ISO-8601 UTC and is converted to local time for display.
 */

/** Formats using the `dd/MM/yyyy`-style patterns of `Locale.FormatoFecha`. */
function applyPattern(pattern: string, parts: Record<string, string>): string {
  // Longest tokens first so `yyyy` is not consumed as two `yy`.
  return pattern.replace(/yyyy|yy|MM|dd|HH|mm|ss/g, (token) => parts[token] ?? token)
}

function pad(value: number, length = 2): string {
  return String(value).padStart(length, '0')
}

export function useDateFormat() {
  const config = useConfigStore()
  const locale = computed(() => config.config?.locale)

  function partsOf(date: Date): Record<string, string> {
    return {
      yyyy: String(date.getFullYear()),
      yy: pad(date.getFullYear() % 100),
      MM: pad(date.getMonth() + 1),
      dd: pad(date.getDate()),
      HH: pad(date.getHours()),
      mm: pad(date.getMinutes()),
      ss: pad(date.getSeconds()),
    }
  }

  /** Formats `YYYY-MM-DD` with no timezone conversion at all. */
  function formatCivil(value: CivilDate | null | undefined): string {
    if (!value) return ''
    const [year, month, day] = value.split('-')
    if (!year || !month || !day) return value
    return applyPattern(locale.value?.formatoFecha ?? 'dd/MM/yyyy', {
      yyyy: year,
      yy: year.slice(-2),
      MM: month,
      dd: day,
      HH: '00',
      mm: '00',
      ss: '00',
    })
  }

  /** Formats an ISO-8601 UTC instant in the machine's local time. */
  function formatInstant(value: Instant | null | undefined, showTime = true): string {
    if (!value) return ''
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value
    const pattern = showTime
      ? (locale.value?.formatoFechaHora ?? 'dd/MM/yyyy HH:mm')
      : (locale.value?.formatoFecha ?? 'dd/MM/yyyy')
    return applyPattern(pattern, partsOf(date))
  }

  /** A `Date` for `DatePicker`, built from a civil date without crossing a timezone. */
  function civilToDate(value: CivilDate | null | undefined): Date | null {
    if (!value) return null
    const [year, month, day] = value.split('-').map(Number)
    if (!year || !month || !day) return null
    return new Date(year, month - 1, day)
  }

  /** The `YYYY-MM-DD` the backend expects, read from the widget's local `Date`. */
  function dateToCivil(value: Date | null | undefined): CivilDate | null {
    if (!value) return null
    return `${value.getFullYear()}-${pad(value.getMonth() + 1)}-${pad(value.getDate())}`
  }

  function instantToDate(value: Instant | null | undefined): Date | null {
    if (!value) return null
    const date = new Date(value)
    return Number.isNaN(date.getTime()) ? null : date
  }

  /** ISO-8601 UTC with milliseconds, which is exactly the storage format the backend parses. */
  function dateToInstant(value: Date | null | undefined): Instant | null {
    if (!value) return null
    return value.toISOString()
  }

  return {
    formatCivil,
    formatInstant,
    civilToDate,
    dateToCivil,
    instantToDate,
    dateToInstant,
  }
}
