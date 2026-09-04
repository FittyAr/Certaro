<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ListState from '@/components/domain/ListState.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useShortcuts } from '@/composables/useShortcuts'
import {
  useOrdenesTrabajoStore,
  type OrdenTrabajoListItem,
} from '@/stores/useOrdenesTrabajoStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import OrdenesTable from './components/OrdenesTable.vue'
import OrdenFormModal from './components/OrdenFormModal.vue'

/**
 * Work orders of one job: the itemised quote certificates are issued against.
 * See `docs/09-modulos-funcionales.md` §3.6.
 */

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useOrdenesTrabajoStore()
const trabajos = useTrabajosStore()

const trabajoId = computed(() => String(route.params.trabajoId ?? ''))

const rows = ref<OrdenTrabajoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
const trabajoDescripcion = ref('')

const editorOpen = ref(false)
const ordenIdEdicion = ref<string | null>(null)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    rows.value = await store.fetchDeTrabajo(trabajoId.value)
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

watch(trabajoId, cargar)

function abrirNuevo(): void {
  ordenIdEdicion.value = null
  editorOpen.value = true
}

function abrirEdicion(id: string): void {
  ordenIdEdicion.value = id
  editorOpen.value = true
}

function onDelete(row: OrdenTrabajoListItem): void {
  confirmDelete({
    entityKey: 'Entity.OrdenTrabajo',
    label: row.titulo,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => cargar(),
  })
}

function abrirDetalle(row: OrdenTrabajoListItem): void {
  void router.push({ name: 'orden-detalle', params: { ordenId: row.id } })
}

useShortcuts({ 'ctrl+n': abrirNuevo })

onMounted(async () => {
  await cargar()
  try {
    const trabajo = await trabajos.fetchOne(trabajoId.value)
    trabajoDescripcion.value = trabajo.descripcion
  } catch (e) {
    notify(e)
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Ordenes.Title')" :subtitle="trabajoDescripcion">
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />
          {{ $t('General.Back') }}
        </Button>
        <Button @click="abrirNuevo()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
        <HelpButton topic-id="ordenes-overview" title="Ayuda sobre Órdenes de Trabajo" />
      </template>
    </PageHeader>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="(rows?.length ?? 0) === 0"
      :is-filtered="false"
      empty-key="Ordenes.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <OrdenesTable
        :rows="rows"
        @detalle="abrirDetalle"
        @editar="abrirEdicion"
        @borrar="onDelete"
      />
    </ListState>

    <OrdenFormModal
      v-model:visible="editorOpen"
      :trabajo-id="trabajoId"
      :orden-id="ordenIdEdicion"
      @saved="cargar()"
    />
  </section>
</template>
