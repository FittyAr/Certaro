<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import { onMounted, ref } from 'vue'

import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useFeriadosStore, type Feriado } from '@/stores/useFeriadosStore'

/**
 * The holiday calendar the settlement reads. See `docs/13-servicios-externos-y-archivos.md` §3.
 *
 * A hand-added holiday wins over the service: syncing inserts what is missing and never overwrites
 * a manual row.
 */

const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useFeriadosStore()

const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
const sincronizando = ref(false)
const mensaje = ref<string | null>(null)

const nuevo = ref<{ fecha: string; nombre: string }>({
  fecha: new Date().toISOString().slice(0, 10),
  nombre: '',
})

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    await store.fetch()
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

async function sincronizar(): Promise<void> {
  if (sincronizando.value) return
  sincronizando.value = true
  mensaje.value = null
  try {
    const result = await store.sync()
    // A failed year is not an error: the calendar stays as it was, and the user is told.
    mensaje.value =
      result.aniosConError > 0 ? 'Feriados.SincronizadoConError' : 'Feriados.SincronizadoOk'
    resultado.value = result
  } catch (e) {
    notify(e)
  } finally {
    sincronizando.value = false
  }
}

const resultado = ref<{ agregados: number; total: number; aniosConError: number } | null>(null)

async function agregar(): Promise<void> {
  if (!nuevo.value.nombre.trim() || !nuevo.value.fecha) return
  try {
    await store.add({ fecha: nuevo.value.fecha, nombre: nuevo.value.nombre })
    nuevo.value.nombre = ''
  } catch (e) {
    notify(e)
  }
}

function quitar(feriado: Feriado): void {
  confirmDelete({
    entityKey: 'Entity.Feriado',
    label: feriado.nombre,
    action: () => store.remove(feriado.fecha),
  })
}

onMounted(cargar)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Feriados')" :subtitle="$t('Feriados.Subtitle')">
      <template #actions>
        <Button variant="outline" :disabled="sincronizando" @click="sincronizar()">
          <AppIcon name="refresh-cw" :size="16" />
          {{ $t('Feriados.Sincronizar') }}
        </Button>
        <HelpButton topic-id="feriados-overview" title="Ayuda sobre Calendario de Feriados" />
      </template>
    </PageHeader>

    <div
      class="flex flex-wrap items-end gap-4 rounded-lg border border-border bg-surface-card p-4 shadow-sm"
    >
      <label class="flex flex-col gap-1.5">
        <span class="text-xs font-medium text-foreground">{{ $t('Feriados.Anio') }}</span>
        <InputNumber
          :model-value="store.anio"
          :use-grouping="false"
          :min="2000"
          :max="2100"
          class="w-28"
          @update:model-value="store.fetch(Number($event))"
        />
      </label>
      <label class="flex flex-col gap-1.5">
        <span class="text-xs font-medium text-foreground">{{ $t('Feriados.Fecha') }}</span>
        <DateInput v-model="nuevo.fecha" />
      </label>
      <label class="flex min-w-[240px] flex-1 flex-col gap-1.5">
        <span class="text-xs font-medium text-foreground">{{ $t('Feriados.Nombre') }}</span>
        <InputText v-model="nuevo.nombre" class="w-full" />
      </label>
      <Button
        :disabled="!nuevo.nombre.trim() || !nuevo.fecha"
        class="flex items-center gap-2"
        @click="agregar()"
      >
        <AppIcon name="plus" :size="16" />
        {{ $t('Feriados.Agregar') }}
      </Button>
    </div>

    <p class="text-xs text-muted-foreground">{{ $t('Feriados.SincronizarAyuda') }}</p>
    <p v-if="mensaje && resultado" class="text-xs font-medium text-primary">
      {{ $t(mensaje, resultado) }}
    </p>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="(store.feriados?.length ?? 0) === 0"
      :is-filtered="false"
      empty-key="Feriados.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <DataTable :value="store.feriados" data-key="fecha" size="small" class="text-sm">
        <Column field="fecha" :header="$t('Feriados.Fecha')">
          <template #body="{ data }"><DateText :value="data.fecha" /></template>
        </Column>
        <Column field="nombre" :header="$t('Feriados.Nombre')" />
        <Column field="tipo" :header="$t('Feriados.Tipo')">
          <template #body="{ data }">{{ data.tipo ?? '—' }}</template>
        </Column>
        <Column field="origen" :header="$t('Feriados.Origen.Manual')">
          <template #body="{ data }">{{ $t(`Feriados.Origen.${data.origen}`) }}</template>
        </Column>
        <Column :header="$t('General.Actions')" :style="{ width: '5rem' }">
          <template #body="{ data }">
            <Button
              variant="ghost"
              size="sm"
              :aria-label="$t('General.Delete')"
              @click="quitar(data)"
            >
              <AppIcon name="trash-2" :size="14" />
            </Button>
          </template>
        </Column>
      </DataTable>
    </ListState>
  </section>
</template>
