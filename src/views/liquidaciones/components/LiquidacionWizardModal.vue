<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Dialog from 'primevue/dialog'
import MultiSelect from 'primevue/multiselect'
import Select from 'primevue/select'
import { computed, ref, watch } from 'vue'
import LiquidacionItemSugerido, { type AjusteLiquidacion } from './LiquidacionItemSugerido.vue'

import DateInput from '@/components/domain/DateInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
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

const mediosPagoOpciones = [
  { label: 'Transferencia Bancaria', value: 'Transferencia' },
  { label: 'Efectivo', value: 'Efectivo' },
  { label: 'Cheque', value: 'Cheque' },
]

async function onProyectoChange(): Promise<void> {
  pagoTrabajoId.value = null
  if (!pagoProyectoId.value) {
    opcionesTrabajo.value = []
    return
  }
  try {
    opcionesTrabajo.value = await trabajos.lookup(pagoProyectoId.value)
    if (opcionesTrabajo.value.length > 0 && opcionesTrabajo.value[0]) {
      pagoTrabajoId.value = opcionesTrabajo.value[0].id
    }
  } catch {
    opcionesTrabajo.value = []
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

async function confirmar(): Promise<void> {
  if (guardando.value) return
  guardando.value = true
  try {
    await store.createBatch(store.sugerencias.map(dtoDe))

    if (registrarEnCaja.value && Number(totalNetoDelLote.value) > 0) {
      try {
        const count = store.sugerencias.length
        const nombres = store.sugerencias.map((s) => s.empleadoNombre).slice(0, 3).join(', ')
        const sufijo = count > 3 ? ` y ${count - 3} más` : ''
        const concepto = `Pago de sueldos: ${nombres}${sufijo} (${periodo.value.desde} al ${periodo.value.hasta}) · ${medioPago.value}`

        let imputacionClienteId: string | null = null
        if (pagoProyectoId.value) {
          try {
            const p = await proyectos.fetchOne(pagoProyectoId.value)
            imputacionClienteId = p?.clienteId ?? null
          } catch {
            // ignore
          }
        }

        await movimientosStore.create({
          fecha: new Date().toISOString(),
          concepto,
          monto: totalNetoDelLote.value,
          cantidad: '1.0000',
          tipoMovimientoId: '00000000-0000-0000-0000-000000000002', // Gasto
          moneda: 'Ars',
          cotizacionAplicada: null,
          tipoConceptoPagoId: '00000000-0000-0000-0000-000000000103', // Liquidación
          categoriaId: categoriaGastoId.value,
          clienteId: imputacionClienteId,
          trabajoId: pagoTrabajoId.value,
          empleadoId: count === 1 ? (store.sugerencias[0]?.empleadoId ?? null) : null,
          facturaId: null,
        })
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
    <div v-if="paso === 1" class="flex flex-col gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Liquidaciones.Empleados') }}</span>
        <MultiSelect
          v-model="seleccion"
          :options="empleados.opciones"
          option-label="label"
          option-value="id"
          :placeholder="$t('Liquidaciones.ElegirEmpleados')"
          filter
          display="chip"
        />
      </label>
      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('General.From') }}</span>
          <DateInput v-model="periodo.desde" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('General.To') }}</span>
          <DateInput v-model="periodo.hasta" />
        </label>
      </div>
      <p class="text-xs text-muted-foreground">{{ $t('Liquidaciones.PasoUnoAyuda') }}</p>
    </div>

    <!-- Step 2: the suggestion, editable -->
    <div v-else-if="paso === 2" class="flex flex-col gap-4">
      <LiquidacionItemSugerido
        v-for="s in store.sugerencias"
        :key="s.empleadoId"
        :sugerencia="s"
        :ajuste="ajusteDe(s.empleadoId)"
        :total-bruto="dtoDe(s).totalBruto"
        :total-neto="(Number(dtoDe(s).totalBruto) - Number(totalAdelantosDe(s))).toFixed(4)"
        @alternar-adelanto="(movId) => alternarAdelanto(s.empleadoId, movId)"
      />
    </div>

    <!-- Step 3: confirmation -->
    <div v-else class="space-y-3">
      <p class="text-sm">
        {{ $t('Liquidaciones.ConfirmarTexto', { cantidad: store.sugerencias?.length ?? 0 }) }}
      </p>
      <ul class="divide-y divide-border text-sm">
        <li
          v-for="s in store.sugerencias"
          :key="s.empleadoId"
          class="flex items-center justify-between py-2"
        >
          <span>{{ s.empleadoNombre }}</span>
          <MoneyText
            :value="(Number(dtoDe(s).totalBruto) - Number(totalAdelantosDe(s))).toFixed(4)"
          />
        </li>
      </ul>
      <div class="flex justify-end gap-2 border-t border-border pt-2 font-medium">
        <span>{{ $t('Liquidaciones.TotalDelLote') }}</span>
        <MoneyText :value="totalNetoDelLote" />
      </div>

      <!-- Cash ledger outflow options -->
      <div class="mt-4 rounded-lg border border-border bg-card/60 p-3 space-y-3">
        <label class="flex items-center gap-2 cursor-pointer font-medium text-sm">
          <Checkbox v-model="registrarEnCaja" binary />
          <span>{{ $t('Liquidaciones.RegistrarEnCaja') }}</span>
        </label>

        <div v-if="registrarEnCaja" class="grid grid-cols-1 gap-3 sm:grid-cols-2 pt-1">
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.MedioPago') }}</span>
            <Select
              v-model="medioPago"
              :options="mediosPagoOpciones"
              option-label="label"
              option-value="value"
            />
          </label>
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.Categoria') }}</span>
            <Select
              v-model="categoriaGastoId"
              :options="categoriasOpciones"
              option-label="label"
              option-value="id"
              filter
              show-clear
              placeholder="Seleccionar categoría"
            />
          </label>
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.ImputarProyecto') }}</span>
            <Select
              v-model="pagoProyectoId"
              :options="opcionesProyecto"
              option-label="label"
              option-value="id"
              filter
              show-clear
              :placeholder="$t('General.None')"
              @change="onProyectoChange()"
            />
          </label>
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.ImputarTrabajo') }}</span>
            <Select
              v-model="pagoTrabajoId"
              :options="opcionesTrabajo"
              option-label="label"
              option-value="id"
              filter
              show-clear
              :placeholder="$t('General.None')"
              :disabled="!pagoProyectoId && opcionesTrabajo.length === 0"
            />
          </label>
        </div>
      </div>
    </div>

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
