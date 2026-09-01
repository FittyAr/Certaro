<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useMovimientosStore, type MovimientoListItem } from '@/stores/useMovimientosStore'
const route = useRoute()
const { notify } = useApiError()
const store = useMovimientosStore()
const obraId = computed(() => String(route.params.obraId ?? ''))
const items = ref<MovimientoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const res = await store.fetchPaged({
      page: 1,
      pageSize: 50,
      filtro: { obraId: obraId.value } as unknown as Record<string, unknown>,
      sortDir: 'Desc',
    })
    items.value = res.items
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}
onMounted(cargar)
</script>
<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Movimientos')" subtitle="Caja de obra" />
    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="items.length === 0"
      :is-filtered="false"
      empty-key="Movimientos.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <DataTable :value="items" data-key="id" size="small" class="text-sm">
        <Column field="fecha" header="Fecha" />
        <Column field="concepto" :header="$t('Movimientos.Concepto')" />
        <Column field="total" :header="$t('Movimientos.Total')">
          <template #body="{ data }"><MoneyText :value="data.total" /></template>
        </Column>
      </DataTable>
    </ListState>
  </section>
</template>
