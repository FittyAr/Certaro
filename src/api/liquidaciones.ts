import { callCommand } from './client'
import type {
  Audit,
  CivilDate,
  Decimal4,
  Instant,
  ListQuery,
  Money,
  PagedResult,
  RowVersion,
  Uuid,
} from './types'

/** See `docs/11-contratos-tauri.md` §5.8. */

export interface LiquidacionFiltro {
  empleadoId?: Uuid
  /** Matched against the period, not against the creation date. */
  fechaDesde?: CivilDate
  fechaHasta?: CivilDate
  soloSinPdf?: boolean
}

/** Where the suggested days came from, so the wizard can say why it proposes a number. */
export type OrigenLiquidacion = 'Manual' | 'Asistencia' | 'Calendario'

export interface LiquidacionSugerenciaQuery {
  empleadoIds: Uuid[]
  desde: CivilDate
  hasta: CivilDate
  /** Days typed by hand, per employee. A value here forces the manual branch. */
  diasManuales?: Record<Uuid, Decimal4>
}

export interface LiquidacionDesglose {
  jornadasCompletas: Decimal4
  jornadasMedias: Decimal4
  faltas: number
  faltasJustificadas: number
  diasSabado: Decimal4
  diasDomingo: Decimal4
  diasFeriado: Decimal4
  multiplicadorSabado: Decimal4
  multiplicadorDomingo: Decimal4
  multiplicadorFeriado: Decimal4
  /** How much of the gross comes from the multipliers, shown as its own line. */
  recargos: Money
}

export interface LiquidacionAdelantoSugerido {
  movimientoId: Uuid
  fecha: CivilDate
  concepto: string
  monto: Money
  /** Already consumed by another settlement: shown struck out and not added. */
  yaDescontado: boolean
  liquidacionQueLoDesconto: Uuid | null
  incluir: boolean
}

export interface LiquidacionSugerencia {
  empleadoId: Uuid
  empleadoNombre: string
  desde: CivilDate
  hasta: CivilDate
  diasTrabajados: Decimal4
  tarifaAplicada: Money
  totalBruto: Money
  totalAdelantos: Money
  totalNeto: Money
  origen: OrigenLiquidacion
  incluirSabados: boolean
  incluirDomingos: boolean
  incluirFeriados: boolean
  desglose: LiquidacionDesglose
  adelantos: LiquidacionAdelantoSugerido[]
  /** True when the period has no holiday at all, which usually means a failed sync. */
  feriadosNoDisponibles: boolean
}

export interface LiquidacionAdelantoInput {
  movimientoId: Uuid
  fecha: CivilDate
  concepto: string
  monto: Money
}

export interface LiquidacionInput {
  empleadoId: Uuid
  fechaInicio: CivilDate
  fechaFin: CivilDate
  diasTrabajados: Decimal4
  tarifaAplicada: Money
  incluirSabados: boolean
  incluirDomingos: boolean
  incluirFeriados: boolean
  multiplicadorSabado: Decimal4
  multiplicadorDomingo: Decimal4
  multiplicadorFeriado: Decimal4
  totalBruto: Money
  totalAdelantos: Money
  observaciones: string | null
  adelantos: LiquidacionAdelantoInput[]
}

export interface LiquidacionBatchResult {
  creadas: Uuid[]
}

/** Only the notes are meant to change; the amounts are frozen once the PDF is handed over. */
export interface LiquidacionUpdateInput {
  diasTrabajados: Decimal4
  tarifaAplicada: Money
  totalBruto: Money
  totalAdelantos: Money
  observaciones: string | null
}

export interface LiquidacionAdelantoDto {
  id: Uuid
  movimientoId: Uuid
  fecha: CivilDate
  concepto: string
  monto: Money
}

export interface LiquidacionListItem {
  id: Uuid
  empleadoId: Uuid
  empleadoNombre: string
  fechaInicio: CivilDate
  fechaFin: CivilDate
  diasTrabajados: Decimal4
  totalBruto: Money
  totalAdelantos: Money
  totalNeto: Money
  pdfGeneradoAt: Instant | null
  rowVersion: RowVersion
}

export interface LiquidacionDetalle {
  id: Uuid
  empleadoId: Uuid
  empleadoNombre: string
  empleadoCargo: string | null
  empleadoDni: string | null
  fechaInicio: CivilDate
  fechaFin: CivilDate
  diasTrabajados: Decimal4
  tarifaAplicada: Money
  incluirSabados: boolean
  incluirDomingos: boolean
  incluirFeriados: boolean
  multiplicadorSabado: Decimal4
  multiplicadorDomingo: Decimal4
  multiplicadorFeriado: Decimal4
  totalBruto: Money
  totalAdelantos: Money
  totalNeto: Money
  observaciones: string | null
  pdfGeneradoAt: Instant | null
  admiteCambioDeImportes: boolean
  adelantos: LiquidacionAdelantoDto[]
  audit: Audit
}

export const LIQUIDACIONES_SORTABLE = [
  'empleadoNombre',
  'fechaInicio',
  'diasTrabajados',
  'totalBruto',
  'totalNeto',
] as const

export function listLiquidaciones(
  query: ListQuery<LiquidacionFiltro>,
): Promise<PagedResult<LiquidacionListItem>> {
  return callCommand('liquidaciones_list', { query })
}

export function getLiquidacion(id: Uuid): Promise<LiquidacionDetalle> {
  return callCommand('liquidaciones_get', { id })
}

/** Pure: computes and persists nothing. Feeds step two of the wizard. */
export function suggestLiquidaciones(
  query: LiquidacionSugerenciaQuery,
): Promise<LiquidacionSugerencia[]> {
  return callCommand('liquidaciones_suggest', { query })
}

export function createLiquidacion(dto: LiquidacionInput): Promise<LiquidacionDetalle> {
  return callCommand('liquidaciones_create', { dto })
}

/** Atomic: if one settlement of the batch fails, none is saved. */
export function createLiquidacionesBatch(
  dtos: LiquidacionInput[],
): Promise<LiquidacionBatchResult> {
  return callCommand('liquidaciones_create_batch', { dto: { dtos } })
}

export function updateLiquidacion(
  id: Uuid,
  dto: LiquidacionUpdateInput,
  rowVersion: RowVersion,
): Promise<LiquidacionDetalle> {
  return callCommand('liquidaciones_update', { id, dto, rowVersion })
}

export function deleteLiquidacion(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('liquidaciones_delete', { id, rowVersion })
}
