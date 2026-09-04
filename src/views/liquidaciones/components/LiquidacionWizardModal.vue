<script setup lang="ts">
import Dialog from 'primevue/dialog'
import { computed, ref, watch } from 'vue'
import LiquidacionItemSugerido, { type AjusteLiquidacion } from './LiquidacionItemSugerido.vue'
import WizardStepSeleccion from './WizardStepSeleccion.vue'
import WizardStepConfirmacion from './WizardStepConfirmacion.vue'

import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
import { useEmpleadosStore } from '@/stores/useEmpleadosStore'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import {
  useLiquidacionesStore,
  type LiquidacionInput,
  type LiquidacionSugerencia,
} from '@/stores/useLiquidacionesStore'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'saved'): void
}>()

const { notify } = useApiError()
const store = useLiquidacionesStore()
const empleados = useEmpleadosStore()
const catalog = useCatalogStore()
const movimientosStore = useMovimientosStore()
const proyectos = useProyectosStore()
const trabajos = useTrabajosStore()

/** What the user can change in step two, kept apart from the suggestion the backend returned. */
type Ajuste = AjusteLiquidacion

const paso = ref<1 | 2 | 3>(1)
const cargandoSugerencias = ref(false)
const guardando = ref(false)

// Cash ledger integration for settlement payout
const registrarEnCaja = ref(true)
const categoriaGastoId = ref<string | null>(null)
const medioPago = ref<'Efectivo' | 'Transferencia' | 'Cheque'>('Transferencia')
const categoriasOpciones = ref<LookupItem[]>([])
const pagoProyectoId = ref<string | null>(null)
const pagoTrabajoId = ref<string | null>(null)
const opcionesProyecto = ref<LookupItem[]>([])
const opcionesTrabajo = ref<LookupItem[]>([])
const trabajosPorProyecto = ref<Record<string, LookupItem[]>>({})

const mediosPagoOpciones = [
  { label: 'Transferencia Bancaria', value: 'Transferencia' },
  { label: 'Efectivo', value: 'Efectivo' },
  { label: 'Cheque', value: 'Cheque' },
]

async function cargarTrabajosProyecto(proyectoId: string): Promise<LookupItem[]> {
  if (trabajosPorProyecto.value[proyectoId]) {
    return trabajosPorProyecto.value[proyectoId]
  }
  try {
    const list = await trabajos.lookup(proyectoId)
    trabajosPorProyecto.value[proyectoId] = list
    return list
  } catch {
    trabajosPorProyecto.value[proyectoId] = []
    return []
  }
}

async function onProyectoChange(): Promise<void> {
  pagoTrabajoId.value = null
  if (!pagoProyectoId.value) {
    opcionesTrabajo.value = []
    return
  }
  opcionesTrabajo.value = await cargarTrabajosProyecto(pagoProyectoId.value)
  if (opcionesTrabajo.value.length > 0 && opcionesTrabajo.value[0]) {
    pagoTrabajoId.value = opcionesTrabajo.value[0].id
  }
}

async function onEmpleadoProyectoChange(empleadoId: string, proyectoId: string | null): Promise<void> {
  const aj = ajustes.value[empleadoId]
  if (!aj) return
  aj.trabajoId = null
  if (proyectoId) {
    const trs = await cargarTrabajosProyecto(proyectoId)
    if (trs.length > 0 && trs[0]) {
      aj.trabajoId = trs[0].id
    }
  }
}

function primerDiaDelMes(): string {
  const now = new Date()
  return new Date(now.getFullYear(), now.getMonth(), 1).toISOString().slice(0, 10)
}

function hoy(): string {
  return new Date().toISOString().slice(0, 10)
}

const seleccion = ref<string[]>([])
const periodo = ref({ desde: primerDiaDelMes(), hasta: hoy() })
const ajustes = ref<Record<string, Ajuste>>({})

async function inicializarWizard(): Promise<void> {
  paso.value = 1
  seleccion.value = []
  periodo.value = { desde: primerDiaDelMes(), hasta: hoy() }
  ajustes.value = {}
  trabajosPorProyecto.value = {}
  store.sugerencias = []
  registrarEnCaja.value = true
  medioPago.value = 'Transferencia'
  pagoProyectoId.value = null
  pagoTrabajoId.value = null
  opcionesTrabajo.value = []
  void empleados.fetchLookup(true)
  try {
    const [cats, proys] = await Promise.all([
      catalog.loadCategorias(),
      proyectos.lookup(undefined, undefined, 200),
    ])
    categoriasOpciones.value = cats
    opcionesProyecto.value = proys
    const catSueldos = categoriasOpciones.value.find((c) =>
      c.label.toLowerCase().includes('sueldo'),
    )
    if (catSueldos) {
      categoriaGastoId.value = catSueldos.id
    }
  } catch {
    // Ignore error loading categories
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      void inicializarWizard()
    }
  },
)

async function calcular(): Promise<void> {
  if (seleccion.value.length === 0 || cargandoSugerencias.value) return
  cargandoSugerencias.value = true
  try {
    const sugerencias = await store.suggest({
      empleadoIds: seleccion.value,
      desde: periodo.value.desde,
      hasta: periodo.value.hasta,
    })
    ajustes.value = Object.fromEntries(
      sugerencias.map((s) => [
        s.empleadoId,
        {
          diasTrabajados: s.diasTrabajados,
          tarifaAplicada: s.tarifaAplicada,
          observaciones: null,
          adelantosIncluidos: new Set(
            s.adelantos.filter((a) => a.incluir).map((a) => a.movimientoId),
          ),
          proyectoId: null,
          trabajoId: null,
        },
      ]),
    )
    paso.value = 2
  } catch (e) {
    notify(e)
  } finally {
    cargandoSugerencias.value = false
  }
}

function ajusteDe(empleadoId: string): Ajuste {
  return (
    ajustes.value[empleadoId] ?? {
      diasTrabajados: '0.0000',
      tarifaAplicada: '0.0000',
      observaciones: null,
      adelantosIncluidos: new Set<string>(),
      proyectoId: null,
      trabajoId: null,
    }
  )
}

function alternarAdelanto(empleadoId: string, movimientoId: string): void {
  const incluidos = ajusteDe(empleadoId).adelantosIncluidos
  if (incluidos.has(movimientoId)) incluidos.delete(movimientoId)
  else incluidos.add(movimientoId)
}

function totalAdelantosDe(s: LiquidacionSugerencia): string {
  const incluidos = ajusteDe(s.empleadoId).adelantosIncluidos
  return s.adelantos
    .filter((a) => incluidos.has(a.movimientoId))
    .reduce((acc, a) => acc + Number(a.monto), 0)
    .toFixed(4)
}

function empleadoCambioDeBase(s: LiquidacionSugerencia): boolean {
  const ajuste = ajusteDe(s.empleadoId)
  return ajuste.diasTrabajados !== s.diasTrabajados || ajuste.tarifaAplicada !== s.tarifaAplicada
}

function recargosDe(s: LiquidacionSugerencia, tarifa: number): number {
  if (!s.desglose) return 0
  const multSab = Math.max(0, Number(s.desglose.multiplicadorSabado) - 1)
  const multDom = Math.max(0, Number(s.desglose.multiplicadorDomingo) - 1)
  const multFer = Math.max(0, Number(s.desglose.multiplicadorFeriado) - 1)
  const sab = Number(s.desglose.diasSabado) * multSab * tarifa
  const dom = Number(s.desglose.diasDomingo) * multDom * tarifa
  const fer = Number(s.desglose.diasFeriado) * multFer * tarifa
  const sum = sab + dom + fer
  return sum > 0 ? sum : Number(s.desglose.recargos ?? 0)
}

function dtoDe(s: LiquidacionSugerencia): LiquidacionInput {
  const ajuste = ajusteDe(s.empleadoId)
  const fueModificado = empleadoCambioDeBase(s)
  const baseModificada = Number(ajuste.diasTrabajados) * Number(ajuste.tarifaAplicada)
  const recargos = recargosDe(s, Number(ajuste.tarifaAplicada))
  const totalBrutoModificado = (baseModificada + recargos).toFixed(4)
  return {
    empleadoId: s.empleadoId,
    fechaInicio: s.desde,
    fechaFin: s.hasta,
    diasTrabajados: ajuste.diasTrabajados,
    tarifaAplicada: ajuste.tarifaAplicada,
    incluirSabados: s.incluirSabados,
    incluirDomingos: s.incluirDomingos,
    incluirFeriados: s.incluirFeriados,
    multiplicadorSabado: s.desglose.multiplicadorSabado,
    multiplicadorDomingo: s.desglose.multiplicadorDomingo,
    multiplicadorFeriado: s.desglose.multiplicadorFeriado,
    totalBruto: fueModificado ? totalBrutoModificado : s.totalBruto,
    totalAdelantos: totalAdelantosDe(s),
    observaciones: ajuste.observaciones,
    adelantos: s.adelantos
      .filter((a) => ajuste.adelantosIncluidos.has(a.movimientoId))
      .map((a) => ({
        movimientoId: a.movimientoId,
        fecha: a.fecha,
        concepto: a.concepto,
        monto: a.monto,
      })),
  }
}

const hayImputacionIndividual = computed(() =>
  store.sugerencias.some((s) => Boolean(ajusteDe(s.empleadoId).proyectoId)),
)

async function confirmar(): Promise<void> {
  if (guardando.value) return
  guardando.value = true
  try {
    await store.createBatch(store.sugerencias.map(dtoDe))

    if (registrarEnCaja.value && Number(totalNetoDelLote.value) > 0) {
      try {
        const fechaIso = new Date().toISOString()
        const clientesCache = new Map<string, string | null>()

        async function resolverClienteId(proyId: string | null): Promise<string | null> {
          if (!proyId) return null
          if (clientesCache.has(proyId)) return clientesCache.get(proyId)!
          try {
            const p = await proyectos.fetchOne(proyId)
            const cId = p?.clienteId ?? null
            clientesCache.set(proyId, cId)
            return cId
          } catch {
            clientesCache.set(proyId, null)
            return null
          }
        }

        if (hayImputacionIndividual.value) {
          // Record individual cash movements per employee to properly impute labor costs to each site
          for (const s of store.sugerencias) {
            const aj = ajusteDe(s.empleadoId)
            const netoEmpleado = (Number(dtoDe(s).totalBruto) - Number(totalAdelantosDe(s))).toFixed(4)
            if (Number(netoEmpleado) <= 0) continue

            const proyId = aj.proyectoId || pagoProyectoId.value
            const trabId = aj.proyectoId ? (aj.trabajoId || null) : (pagoTrabajoId.value || null)
            const clienteId = await resolverClienteId(proyId)
            const concepto = `Pago de sueldo: ${s.empleadoNombre} (${periodo.value.desde} al ${periodo.value.hasta}) · ${medioPago.value}`

            await movimientosStore.create({
              fecha: fechaIso,
              concepto,
              monto: netoEmpleado,
              cantidad: '1.0000',
              tipoMovimientoId: '00000000-0000-0000-0000-000000000002', // Gasto
              moneda: 'Ars',
              cotizacionAplicada: null,
              tipoConceptoPagoId: '00000000-0000-0000-0000-000000000103', // Liquidación
              categoriaId: categoriaGastoId.value,
              clienteId,
              trabajoId: trabId,
              empleadoId: s.empleadoId,
              facturaId: null,
            })
          }
        } else {
          // Unified batch cash movement
          const count = store.sugerencias.length
          const nombres = store.sugerencias.map((s) => s.empleadoNombre).slice(0, 3).join(', ')
          const sufijo = count > 3 ? ` y ${count - 3} más` : ''
          const concepto = `Pago de sueldos: ${nombres}${sufijo} (${periodo.value.desde} al ${periodo.value.hasta}) · ${medioPago.value}`
          const clienteId = await resolverClienteId(pagoProyectoId.value)

          await movimientosStore.create({
            fecha: fechaIso,
            concepto,
            monto: totalNetoDelLote.value,
            cantidad: '1.0000',
            tipoMovimientoId: '00000000-0000-0000-0000-000000000002', // Gasto
            moneda: 'Ars',
            cotizacionAplicada: null,
            tipoConceptoPagoId: '00000000-0000-0000-0000-000000000103', // Liquidación
            categoriaId: categoriaGastoId.value,
            clienteId,
            trabajoId: pagoTrabajoId.value,
            empleadoId: count === 1 ? (store.sugerencias[0]?.empleadoId ?? null) : null,
            facturaId: null,
          })
        }
      } catch (movErr) {
        notify(movErr)
      }
    }

    emit('update:visible', false)
    emit('saved')
  } catch (e) {
    notify(e)
  } finally {
    guardando.value = false
  }
}

const totalNetoDelLote = computed(() =>
  store.sugerencias
    .reduce((acc, s) => acc + Number(dtoDe(s).totalBruto) - Number(totalAdelantosDe(s)), 0)
    .toFixed(4),
)
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    maximizable
    :header="$t('Liquidaciones.AsistenteTitulo', { paso })"
    class="w-full max-w-5xl"
    @update:visible="(val) => emit('update:visible', val)"
  >
    <!-- Step 1: employees and period -->
    <WizardStepSeleccion
      v-if="paso === 1"
      v-model:seleccion="seleccion"
      v-model:periodo="periodo"
      :empleados-opciones="empleados.opciones"
    />

    <!-- Step 2: the suggestion, editable -->
    <div v-else-if="paso === 2" class="flex flex-col gap-4">
      <LiquidacionItemSugerido
        v-for="s in store.sugerencias"
        :key="s.empleadoId"
        :sugerencia="s"
        :ajuste="ajusteDe(s.empleadoId)"
        :total-bruto="dtoDe(s).totalBruto"
        :total-neto="(Number(dtoDe(s).totalBruto) - Number(totalAdelantosDe(s))).toFixed(4)"
        :opciones-proyecto="opcionesProyecto"
        :opciones-trabajo="ajusteDe(s.empleadoId).proyectoId ? (trabajosPorProyecto[ajusteDe(s.empleadoId).proyectoId!] ?? []) : []"
        @alternar-adelanto="(movId) => alternarAdelanto(s.empleadoId, movId)"
        @proyecto-change="(proyId) => onEmpleadoProyectoChange(s.empleadoId, proyId)"
      />
    </div>

    <!-- Step 3: confirmation -->
    <WizardStepConfirmacion
      v-else
      :sugerencias="store.sugerencias"
      :dto-de="dtoDe"
      :total-adelantos-de="totalAdelantosDe"
      :total-neto-del-lote="totalNetoDelLote"
      :hay-imputacion-individual="hayImputacionIndividual"
      v-model:registrar-en-caja="registrarEnCaja"
      v-model:medio-pago="medioPago"
      v-model:categoria-gasto-id="categoriaGastoId"
      v-model:pago-proyecto-id="pagoProyectoId"
      v-model:pago-trabajo-id="pagoTrabajoId"
      :categorias-opciones="categoriasOpciones"
      :opciones-proyecto="opcionesProyecto"
      :opciones-trabajo="opcionesTrabajo"
      :medios-pago-opciones="mediosPagoOpciones"
      @proyecto-change="onProyectoChange()"
    />

    <template #footer>
      <Button v-if="paso === 1" variant="outline" @click="emit('update:visible', false)">
        {{ $t('General.Cancel') }}
      </Button>
      <Button v-else variant="outline" @click="paso = paso === 3 ? 2 : 1">
        {{ $t('General.Back') }}
      </Button>

      <Button
        v-if="paso === 1"
        :disabled="(seleccion?.length ?? 0) === 0 || cargandoSugerencias"
        @click="calcular()"
      >
        {{ $t('Liquidaciones.Calcular') }}
      </Button>
      <Button v-else-if="paso === 2" @click="paso = 3">{{ $t('General.Next') }}</Button>
      <Button v-else :disabled="guardando" @click="confirmar()">
        {{ $t('Liquidaciones.Confirmar') }}
      </Button>
    </template>
  </Dialog>
</template>
