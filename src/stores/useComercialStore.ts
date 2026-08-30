import { defineStore } from 'pinia'

import {
  antiguedadDeuda,
  cuentaCorriente,
  rentabilidadObras,
  rentabilidadTrabajos,
  type AntiguedadDeuda,
  type AntiguedadDeudaQuery,
  type CuentaCorriente,
  type CuentaCorrienteQuery,
} from '@/api/comercial'
import type { RentabilidadItem } from '@/api/dashboard'
import type { Uuid } from '@/api/types'

export type {
  AntiguedadDeuda,
  AntiguedadDeudaCliente,
  AntiguedadDeudaQuery,
  CuentaCorriente,
  CuentaCorrienteFactura,
  CuentaCorrienteQuery,
} from '@/api/comercial'
export type { RentabilidadItem } from '@/api/dashboard'

/** Reads only, and nothing cached: every screen asks for the figures it is showing. */
export const useComercialStore = defineStore('comercial', () => {
  function fetchCuentaCorriente(query: CuentaCorrienteQuery): Promise<CuentaCorriente> {
    return cuentaCorriente(query)
  }

  function fetchAntiguedad(query: AntiguedadDeudaQuery): Promise<AntiguedadDeuda> {
    return antiguedadDeuda(query)
  }

  function fetchRentabilidadObras(limite?: number): Promise<RentabilidadItem[]> {
    return rentabilidadObras(limite)
  }

  function fetchRentabilidadTrabajos(
    obraId?: Uuid,
    limite?: number,
  ): Promise<RentabilidadItem[]> {
    return rentabilidadTrabajos(obraId, limite)
  }

  return {
    fetchCuentaCorriente,
    fetchAntiguedad,
    fetchRentabilidadObras,
    fetchRentabilidadTrabajos,
  }
})
