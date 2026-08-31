<script setup lang="ts">
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useLiquidacionesStore, type LiquidacionDetalle } from '@/stores/useLiquidacionesStore'

/**
 * One settlement. See `docs/09-modulos-funcionales.md` §3.11.
 *
 * The rate, the multipliers and each advance are the copies frozen when it was settled, so raising
 * the employee's rate afterwards cannot rewrite what was paid. Once the PDF was handed over, not
 * even the amounts can be corrected: only the notes.
 */

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useLiquidacionesStore()

const liquidacionId = computed(() => String(route.params.liquidacionId ?? ''))

const liquidacion = ref<LiquidacionDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
const observaciones = ref<string | null>(null)
const guardando = ref(false)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    liquidacion.value = await store.fetchOne(liquidacionId.value)
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

watch(liquidacion, (value) => {
  observaciones.value = value?.observaciones ?? null
})

const observacionesCambiaron = computed(
  () => (observaciones.value ?? '') !== (liquidacion.value?.observaciones ?? ''),
)

async function guardarObservaciones(): Promise<void> {
  const actual = liquidacion.value
  if (!actual || guardando.value) return
  guardando.value = true
  try {
    liquidacion.value = await store.update(
      actual.id,
      {
        diasTrabajados: actual.diasTrabajados,
        tarifaAplicada: actual.tarifaAplicada,
        totalBruto: actual.totalBruto,
        totalAdelantos: actual.totalAdelantos,
        observaciones: observaciones.value?.trim() ? observaciones.value : null,
      },
      actual.audit.rowVersion,
    )
  } catch (e) {
    notify(e)
  } finally {
    guardando.value = false
  }
}

function anular(): void {
  const actual = liquidacion.value
  if (!actual) return
  confirmDelete({
    entityKey: 'Entity.Liquidacion',
    label: actual.empleadoNombre,
    action: () => store.remove(actual.id, actual.audit.rowVersion),
    onDone: () => router.push({ name: 'liquidaciones' }),
  })
}

onMounted(cargar)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader
      :title="liquidacion?.empleadoNombre ?? $t('Menu.Liquidaciones')"
      :subtitle="liquidacion?.empleadoCargo ?? undefined"
    >
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />
          {{ $t('General.Back') }}
        </Button>
        <Button variant="destructive" :disabled="!liquidacion" @click="anular()">
          <AppIcon name="trash-2" :size="16" />
          {{ $t('Liquidaciones.Anular') }}
        </Button>
      </template>
    </PageHeader>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!liquidacion"
      :is-filtered="false"
      empty-key="Liquidaciones.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div v-if="liquidacion" class="space-y-4">
        <dl class="grid grid-cols-2 gap-3 text-sm md:grid-cols-4">
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Periodo') }}</dt>
            <dd>
              <DateText :value="liquidacion.fechaInicio" /> –
              <DateText :value="liquidacion.fechaFin" />
            </dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Dias') }}</dt>
            <dd class="tabular-nums">{{ liquidacion.diasTrabajados }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Tarifa') }}</dt>
            <dd><MoneyText :value="liquidacion.tarifaAplicada" /></dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Empleados.Dni') }}</dt>
            <dd class="tabular-nums">{{ liquidacion.empleadoDni ?? '—' }}</dd>
          </div>
        </dl>

        <div class="rounded-md border border-border p-3 text-sm">
          <h4 class="mb-2 font-semibold">{{ $t('Liquidaciones.ReglasAplicadas') }}</h4>
          <dl class="grid grid-cols-3 gap-3">
            <div>
              <dt class="text-xs text-muted-foreground">{{ $t('Empleados.MultSabado') }}</dt>
              <dd class="tabular-nums">
                {{
                  liquidacion.incluirSabados ? liquidacion.multiplicadorSabado : $t('General.No')
                }}
              </dd>
            </div>
            <div>
              <dt class="text-xs text-muted-foreground">{{ $t('Empleados.MultDomingo') }}</dt>
              <dd class="tabular-nums">
                {{
                  liquidacion.incluirDomingos ? liquidacion.multiplicadorDomingo : $t('General.No')
                }}
              </dd>
            </div>
            <div>
              <dt class="text-xs text-muted-foreground">{{ $t('Empleados.MultFeriado') }}</dt>
              <dd class="tabular-nums">
                {{
                  liquidacion.incluirFeriados ? liquidacion.multiplicadorFeriado : $t('General.No')
                }}
              </dd>
            </div>
          </dl>
        </div>

        <div class="rounded-md border border-border p-3 text-sm">
          <h4 class="mb-2 font-semibold">{{ $t('Liquidaciones.Adelantos') }}</h4>
          <p v-if="!liquidacion.adelantos?.length" class="text-xs text-muted-foreground">
            {{ $t('Liquidaciones.SinAdelantos') }}
          </p>
          <ul v-else class="divide-y divide-border">
            <li
              v-for="adelanto in (liquidacion.adelantos ?? [])"
              :key="adelanto.id"
              class="flex items-center gap-3 py-2"
            >
              <DateText :value="adelanto.fecha" />
              <span class="flex-1">{{ adelanto.concepto }}</span>
              <MoneyText :value="adelanto.monto" />
            </li>
          </ul>
        </div>

        <dl class="ml-auto w-full max-w-sm space-y-1 text-sm">
          <div class="flex justify-between">
            <dt class="text-muted-foreground">{{ $t('Liquidaciones.TotalBruto') }}</dt>
            <dd><MoneyText :value="liquidacion.totalBruto" /></dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">{{ $t('Liquidaciones.TotalAdelantos') }}</dt>
            <dd><MoneyText :value="liquidacion.totalAdelantos" /></dd>
          </div>
          <div class="flex justify-between border-t border-border pt-1 font-medium">
            <dt>{{ $t('Liquidaciones.TotalNeto') }}</dt>
            <dd><MoneyText :value="liquidacion.totalNeto" /></dd>
          </div>
        </dl>

        <p v-if="!liquidacion.admiteCambioDeImportes" class="text-xs text-muted-foreground">
          {{ $t('Liquidaciones.ImportesCongelados') }}
        </p>

        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Liquidaciones.Observaciones') }}</span>
          <Textarea v-model="observaciones" rows="3" auto-resize />
        </label>
        <div class="flex justify-end">
          <Button :disabled="!observacionesCambiaron || guardando" @click="guardarObservaciones()">
            {{ $t('General.Save') }}
          </Button>
        </div>
      </div>
    </ListState>
  </section>
</template>
