<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useProyectosStore, type ProyectoDetalle } from '@/stores/useProyectosStore'
import { useTrabajosStore, type TrabajoListItem } from '@/stores/useTrabajosStore'

const route = useRoute()
const { notify } = useApiError()
const store = useTrabajosStore()
const proyectos = useProyectosStore()

const proyectoId = computed(() => String(route.params.proyectoId ?? ''))
const proyecto = ref<ProyectoDetalle | null>(null)
const items = ref<TrabajoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const [res, proy] = await Promise.all([
      store.fetchPaged({
        page: 1,
        pageSize: 50,
        filtro: { proyectoId: proyectoId.value } as unknown as Record<string, unknown>,
        sortDir: 'Desc',
      }),
      proyectos.fetchOne(proyectoId.value).catch(() => null),
    ])
    items.value = res.items
    proyecto.value = proy
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
    <PageHeader
      :title="$t('Menu.Trabajos')"
      :subtitle="proyecto ? `${proyecto.nombre} · #${proyecto.numero}` : $t('Proyectos.Trabajos')"
    />
    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="items.length === 0"
      :is-filtered="false"
      empty-key="Trabajos.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <DataTable :value="items" data-key="id" size="small" class="text-sm">
        <Column field="descripcion" :header="$t('Trabajos.Descripcion')" />
        <Column field="estado" :header="$t('Trabajos.Estado')">
          <template #body="{ data }">
            <StatePill entity="Trabajo" :value="data.estado" />
          </template>
        </Column>
        <Column field="presupuesto" :header="$t('Trabajos.Presupuesto')">
          <template #body="{ data }">
            <MoneyText :value="data.presupuesto" />
          </template>
        </Column>
      </DataTable>
    </ListState>
  </section>
</template>
