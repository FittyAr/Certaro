<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Column from 'primevue/column'
import Dialog from 'primevue/dialog'
import MultiSelect from 'primevue/multiselect'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import { useEmpleadosStore } from '@/stores/useEmpleadosStore'
import {
  useLiquidacionesStore,
  type LiquidacionFiltro,
  type LiquidacionInput,
  type LiquidacionListItem,
  type LiquidacionSugerencia,
} from '@/stores/useLiquidacionesStore'

/**
 * Settlements and the wizard that creates them. See `docs/09-modulos-funcionales.md` §3.11.
 *
 * The wizard has three steps: pick the employees and the period, review what the backend suggests,
 * and confirm. Step two is where the numbers can be corrected; every amount shown there was
 * computed by the backend, because the frontend must not do arithmetic on money.
 */

const router = useRouter()
const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useLiquidacionesStore()
const empleados = useEmpleadosStore()

const table = useServerTable<LiquidacionFiltro, LiquidacionListItem>({
  key: 'liquidaciones',
  initialFilter: {},
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fechaInicio', dir: 'Desc' },
})

const filtrosActivos = computed(
  () =>
    Boolean(table.filter.value.empleadoId) ||
    Boolean(table.filter.value.fechaDesde) ||
    Boolean(table.filter.value.fechaHasta) ||
    table.filter.value.soloSinPdf === true,
)

function onDelete(row: LiquidacionListItem): void {
  confirmDelete({
    entityKey: 'Entity.Liquidacion',
    label: row.empleadoNombre,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

function abrirDetalle(row: LiquidacionListItem): void {
  void router.push({ name: 'liquidacion-detalle', params: { liquidacionId: row.id } })
}

// ------------------------------------------------------------------ wizard

/** What the user can change in step two, kept apart from the suggestion the backend returned. */
interface Ajuste {
  diasTrabajados: string
  tarifaAplicada: string
  observaciones: string | null
  adelantosIncluidos: Set<string>
}

const wizardOpen = ref(false)
const paso = ref<1 | 2 | 3>(1)
const cargandoSugerencias = ref(false)
const guardando = ref(false)

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

function abrirWizard(): void {
  paso.value = 1
  seleccion.value = []
  periodo.value = { desde: primerDiaDelMes(), hasta: hoy() }
  ajustes.value = {}
  store.sugerencias = []
  wizardOpen.value = true
  void empleados.fetchLookup(true)
}

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
          // Only what the backend already marked as includable: a spent advance stays out.
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

/**
 * Recomputed with the sums the backend sent per advance. The gross is not recomputed here: if the
 * days or the rate changed, the numbers are recalculated by asking the backend again.
 */
function totalAdelantosDe(s: LiquidacionSugerencia): string {
  const incluidos = ajusteDe(s.empleadoId).adelantosIncluidos
  return s.adelantos
    .filter((a) => incluidos.has(a.movimientoId))
    .reduce((acc, a) => acc + Number(a.monto), 0)
    .toFixed(4)
}

const huboCambioDeBase = computed(() =>
  store.sugerencias.some((s) => {
    const ajuste = ajusteDe(s.empleadoId)
    return ajuste.diasTrabajados !== s.diasTrabajados || ajuste.tarifaAplicada !== s.tarifaAplicada
  }),
)

function dtoDe(s: LiquidacionSugerencia): LiquidacionInput {
  const ajuste = ajusteDe(s.empleadoId)
  const totalBruto = (Number(ajuste.diasTrabajados) * Number(ajuste.tarifaAplicada)).toFixed(4)
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
    // The untouched gross keeps the surcharges the backend computed; a corrected base cannot.
    totalBruto: huboCambioDeBase.value ? totalBruto : s.totalBruto,
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
    wizardOpen.value = false
    await table.reload()
  } catch (e) {
    // The batch is atomic: nothing was saved, so the wizard stays open with the same numbers.
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

useShortcuts({ 'ctrl+n': abrirWizard })

onMounted(() => {
  table.start()
  void empleados.fetchLookup(true)
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Liquidaciones')" :subtitle="$t('Liquidaciones.Subtitle')">
      <template #actions>
        <Button @click="abrirWizard()">
          <AppIcon name="plus" :size="16" />
          {{ $t('Liquidaciones.Nueva') }}
        </Button>
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Empleados.Nombre') }}</span>
        <Select
          v-model="table.filter.value.empleadoId"
          :options="empleados.opciones"
          option-label="label"
          option-value="id"
          :placeholder="$t('General.All')"
          show-clear
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.From') }}</span>
        <DateInput v-model="table.filter.value.fechaDesde" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.To') }}</span>
        <DateInput v-model="table.filter.value.fechaHasta" />
      </label>
      <label class="flex items-center gap-2 self-end pb-2">
        <Checkbox v-model="table.filter.value.soloSinPdf" binary />
        <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.SoloSinPdf') }}</span>
      </label>
    </FilterBar>

    <DataGrid
      :table="table"
      empty-key="Liquidaciones.Empty"
      class="flex-1"
      @row-edit="(row: any) => abrirDetalle(row)"
    >
      <Column field="empleadoNombre" :header="$t('Empleados.Nombre')" sortable />
      <Column field="fechaInicio" :header="$t('Liquidaciones.Periodo')" sortable>
        <template #body="{ data }">
          <DateText :value="data.fechaInicio" /> – <DateText :value="data.fechaFin" />
        </template>
      </Column>
      <Column field="diasTrabajados" :header="$t('Liquidaciones.Dias')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.diasTrabajados }}</span>
        </template>
      </Column>
      <Column field="totalBruto" :header="$t('Liquidaciones.TotalBruto')" sortable>
        <template #body="{ data }"><MoneyText :value="data.totalBruto" /></template>
      </Column>
      <Column field="totalAdelantos" :header="$t('Liquidaciones.TotalAdelantos')">
        <template #body="{ data }"><MoneyText :value="data.totalAdelantos" /></template>
      </Column>
      <Column field="totalNeto" :header="$t('Liquidaciones.TotalNeto')" sortable>
        <template #body="{ data }"><MoneyText :value="data.totalNeto" /></template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button variant="ghost" size="sm" @click="abrirDetalle(data)">
            <AppIcon name="eye" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" @click="onDelete(data)">
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <Dialog
      v-model:visible="wizardOpen"
      modal
      maximizable
      :header="$t('Liquidaciones.AsistenteTitulo', { paso })"
      class="w-full max-w-5xl"
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
        <div
          v-for="s in store.sugerencias"
          :key="s.empleadoId"
          class="space-y-3 rounded-md border border-border p-3"
        >
          <div class="flex items-baseline justify-between">
            <h4 class="font-semibold">{{ s.empleadoNombre }}</h4>
            <span class="text-xs text-muted-foreground">
              {{ $t(`Liquidaciones.Origen.${s.origen}`) }}
            </span>
          </div>

          <p v-if="s.feriadosNoDisponibles" class="rounded bg-warning/10 p-2 text-xs text-warning">
            {{ $t('Liquidaciones.Warning.FeriadosNoDisponibles') }}
          </p>

          <div class="grid grid-cols-2 gap-3 md:grid-cols-4">
            <label class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Dias') }}</span>
              <DecimalInput v-model="ajusteDe(s.empleadoId).diasTrabajados" :min="0" />
            </label>
            <label class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Tarifa') }}</span>
              <MoneyInput v-model="ajusteDe(s.empleadoId).tarifaAplicada" :min="0" />
            </label>
            <div class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Recargos') }}</span>
              <span class="py-2 text-sm"><MoneyText :value="s.desglose.recargos" /></span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">
                {{ $t('Liquidaciones.TotalBruto') }}
              </span>
              <span class="py-2 text-sm"><MoneyText :value="dtoDe(s).totalBruto" /></span>
            </div>
          </div>

          <div v-if="(s.adelantos?.length ?? 0) > 0" class="space-y-1">
            <span class="text-xs font-medium">{{ $t('Liquidaciones.Adelantos') }}</span>
            <label
              v-for="adelanto in s.adelantos"
              :key="adelanto.movimientoId"
              class="flex items-center gap-2 text-xs"
              :class="{ 'text-muted-foreground line-through': adelanto.yaDescontado }"
            >
              <Checkbox
                :model-value="ajusteDe(s.empleadoId).adelantosIncluidos.has(adelanto.movimientoId)"
                binary
                :disabled="adelanto.yaDescontado"
                @update:model-value="alternarAdelanto(s.empleadoId, adelanto.movimientoId)"
              />
              <DateText :value="adelanto.fecha" />
              <span class="flex-1">{{ adelanto.concepto }}</span>
              <MoneyText :value="adelanto.monto" />
              <span v-if="adelanto.yaDescontado">
                {{ $t('Liquidaciones.YaDescontado') }}
              </span>
            </label>
          </div>

          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">
              {{ $t('Liquidaciones.Observaciones') }}
            </span>
            <Textarea v-model="ajusteDe(s.empleadoId).observaciones" rows="2" auto-resize />
          </label>

          <div class="flex justify-end gap-2 border-t border-border pt-2 text-sm">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.TotalNeto') }}</span>
            <MoneyText
              :value="(Number(dtoDe(s).totalBruto) - Number(totalAdelantosDe(s))).toFixed(4)"
            />
          </div>
        </div>
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
      </div>

      <template #footer>
        <Button v-if="paso === 1" variant="outline" @click="wizardOpen = false">
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
  </section>
</template>
