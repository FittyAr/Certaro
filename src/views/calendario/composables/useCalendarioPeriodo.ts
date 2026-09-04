import { computed } from 'vue'
import type { useCalendarioStore } from '@/stores/useCalendarioStore'

export const DIAS_SEMANA = ['Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb', 'Dom']
export const HORAS_DIA = Array.from({ length: 14 }, (_, i) => i + 7) // 07:00 to 20:00

export function pad(n: number): string {
  return n.toString().padStart(2, '0')
}

export function formatearFechaIso(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

export function fechaLocalIsoDe(isoUtc: string): string {
  return formatearFechaIso(new Date(isoUtc))
}

export function formatearHoraLocal(isoUtc: string): string {
  const d = new Date(isoUtc)
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function formatearLocalParaInput(isoUtc: string): string {
  const d = new Date(isoUtc)
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function coincideHora(isoUtc: string, hora: number): boolean {
  const d = new Date(isoUtc)
  return d.getHours() === hora
}

export function useCalendarioPeriodo(store: ReturnType<typeof useCalendarioStore>) {
  const rangoActual = computed(() => {
    const base = new Date(store.fechaSeleccionada)
    const y = base.getFullYear()
    const m = base.getMonth()

    if (store.vistaActual === 'mes') {
      const primerDiaMes = new Date(y, m, 1)
      const ultimoDiaMes = new Date(y, m + 1, 0)
      const diaSemana = (primerDiaMes.getDay() + 6) % 7
      const inicio = new Date(primerDiaMes)
      inicio.setDate(inicio.getDate() - diaSemana)

      const fin = new Date(ultimoDiaMes)
      const extraDias = (7 - ((ultimoDiaMes.getDay() + 6) % 7) - 1) % 7
      fin.setDate(fin.getDate() + extraDias)
      fin.setHours(23, 59, 59, 999)

      return {
        desde: inicio.toISOString(),
        hasta: fin.toISOString(),
        inicio,
        fin,
      }
    }

    if (store.vistaActual === 'semana') {
      const diaSemana = (base.getDay() + 6) % 7
      const inicio = new Date(base)
      inicio.setDate(inicio.getDate() - diaSemana)
      inicio.setHours(0, 0, 0, 0)

      const fin = new Date(inicio)
      fin.setDate(fin.getDate() + 6)
      fin.setHours(23, 59, 59, 999)

      return {
        desde: inicio.toISOString(),
        hasta: fin.toISOString(),
        inicio,
        fin,
      }
    }

    // 'dia' or 'recursos' (single day)
    const inicio = new Date(base)
    inicio.setHours(0, 0, 0, 0)
    const fin = new Date(base)
    fin.setHours(23, 59, 59, 999)

    return {
      desde: inicio.toISOString(),
      hasta: fin.toISOString(),
      inicio,
      fin,
    }
  })

  const tituloPeriodo = computed(() => {
    const d = store.fechaSeleccionada
    const meses = [
      'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
      'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre',
    ]
    if (store.vistaActual === 'mes') {
      return `${meses[d.getMonth()]} ${d.getFullYear()}`
    }
    if (store.vistaActual === 'semana') {
      const { inicio, fin } = rangoActual.value
      const mInicio = meses[inicio.getMonth()] ?? ''
      const mFin = meses[fin.getMonth()] ?? ''
      return `${inicio.getDate()} ${mInicio.substring(0, 3)} - ${fin.getDate()} ${mFin.substring(0, 3)} ${fin.getFullYear()}`
    }
    return `${d.getDate()} de ${meses[d.getMonth()] ?? ''} de ${d.getFullYear()}`
  })

  function irAnterior() {
    const d = new Date(store.fechaSeleccionada)
    if (store.vistaActual === 'mes') {
      d.setMonth(d.getMonth() - 1)
    } else if (store.vistaActual === 'semana') {
      d.setDate(d.getDate() - 7)
    } else {
      d.setDate(d.getDate() - 1)
    }
    store.fechaSeleccionada = d
  }

  function irSiguiente() {
    const d = new Date(store.fechaSeleccionada)
    if (store.vistaActual === 'mes') {
      d.setMonth(d.getMonth() + 1)
    } else if (store.vistaActual === 'semana') {
      d.setDate(d.getDate() + 7)
    } else {
      d.setDate(d.getDate() + 1)
    }
    store.fechaSeleccionada = d
  }

  function irHoy() {
    store.fechaSeleccionada = new Date()
  }

  return {
    rangoActual,
    tituloPeriodo,
    irAnterior,
    irSiguiente,
    irHoy,
  }
}
