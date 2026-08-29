<script setup lang="ts">
import Column from 'primevue/column'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref } from 'vue'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import FieldError from '@/components/domain/FieldError.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import type { LookupItem } from '@/stores/useCatalogStore'
import {
  useCategoriasStore,
  type CategoriaFiltro,
  type CategoriaInput,
  type CategoriaListItem,
} from '@/stores/useCategoriasStore'

/**
 * Administration of the categories. See `docs/09-modulos-funcionales.md` §3.13.
 *
 * Categories form a one-level-per-step hierarchy (RC-04). A category with movements or with
 * children is not deletable, and the list says so before the user tries.
 */

const { confirmDelete } = useConfirmDelete()
const store = useCategoriasStore()

const table = useServerTable<CategoriaFiltro, CategoriaListItem>({
  key: 'categorias',
  initialFilter: { texto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'nombre', dir: 'Asc' },
})

const padres = ref<LookupItem[]>([])

async function cargarPadres(): Promise<void> {
  padres.value = await store.lookup(undefined, 100)
}

const drawer = useCrudDrawer<CategoriaInput & { rowVersion?: string; id?: string }>({
  entityKey: 'Entity.Categoria',
  empty: () => ({
    nombre: '',
    descripcion: null,
    colorHex: null,
    icono: null,
    categoriaPadreId: null,
  }),
  load: async (id) => {
    const detalle = await store.fetchOne(id)
    return {
      id: detalle.id,
      nombre: detalle.nombre,
      descripcion: detalle.descripcion,
      colorHex: detalle.colorHex,
      icono: detalle.icono,
      categoriaPadreId: detalle.categoriaPadreId,
      rowVersion: detalle.audit.rowVersion,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => {
    void table.reload()
    void cargarPadres()
  },
})

/** A category cannot be its own parent, so it is left out of its own selector. */
const opcionesPadre = computed(() =>
  padres.value.filter((option) => option.id !== drawer.model.value.id),
)

const filtrosActivos = computed(
  () => Boolean(table.filter.value.texto) || Boolean(table.filter.value.soloRaiz),
)

function onDelete(row: CategoriaListItem): void {
  confirmDelete({
    entityKey: 'Entity.Categoria',
    label: row.nombre,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

function motivoNoBorrable(row: CategoriaListItem): string | undefined {
  if (row.puedeEliminarse) return undefined
  return row.hijasCount > 0 ? 'Categorias.NoBorrableConHijas' : 'Categorias.NoBorrableEnUso'
}

useShortcuts({ 'ctrl+n': () => drawer.openCreate() })

onMounted(() => {
  table.start()
  void cargarPadres()
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Categories')" :subtitle="$t('Categorias.Subtitle')">
      <template #actions>
        <Button @click="drawer.openCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.Search') }}</span>
        <InputText v-model="table.filter.value.texto" :placeholder="$t('General.Search')" />
      </label>
      <label class="flex items-center gap-2 pb-2">
        <input v-model="table.filter.value.soloRaiz" type="checkbox" class="accent-primary" />
        <span class="text-sm">{{ $t('Categorias.SoloRaiz') }}</span>
      </label>
    </FilterBar>

    <DataGrid
      :table="table"
      empty-key="Categorias.Empty"
      class="flex-1"
      @row-edit="(row) => drawer.openEdit(row.id)"
    >
      <Column field="nombre" :header="$t('Categorias.Nombre')" sortable>
        <template #body="{ data }">
          <span class="flex items-center gap-2">
            <span
              v-if="data.colorHex"
              class="inline-block size-3 rounded-full border border-border"
              :style="{ backgroundColor: data.colorHex }"
            />
            {{ data.nombre }}
          </span>
        </template>
      </Column>
      <Column field="categoriaPadreNombre" :header="$t('Categorias.Padre')">
        <template #body="{ data }">
          <span class="text-muted-foreground">{{ data.categoriaPadreNombre ?? '—' }}</span>
        </template>
      </Column>
      <Column field="hijasCount" :header="$t('Categorias.Hijas')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.hijasCount }}</span>
        </template>
      </Column>
      <Column field="movimientosCount" :header="$t('Categorias.EnUso')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.movimientosCount }}</span>
        </template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button variant="ghost" size="sm" @click="drawer.openEdit(data.id)">
            <AppIcon name="pencil" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :disabled="!data.puedeEliminarse"
            :title="motivoNoBorrable(data) ? $t(motivoNoBorrable(data)!) : undefined"
            @click="onDelete(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <CrudDrawer :drawer="drawer" title-key="Entity.Categoria">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Categorias.Nombre') }}</span>
        <InputText
          v-model="drawer.model.value.nombre"
          :invalid="Boolean(drawer.fieldErrors.value.nombre)"
          aria-describedby="cat-nombre-error"
        />
        <FieldError id="cat-nombre-error" :message="drawer.fieldErrors.value.nombre" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Categorias.Padre') }}</span>
        <Select
          v-model="drawer.model.value.categoriaPadreId"
          :options="opcionesPadre"
          option-label="label"
          option-value="id"
          show-clear
          filter
          :placeholder="$t('Categorias.SinPadre')"
        />
        <FieldError id="cat-padre-error" :message="drawer.fieldErrors.value.categoriaPadreId" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Categorias.Color') }}</span>
        <input
          v-model="drawer.model.value.colorHex"
          type="color"
          class="h-9 w-16 rounded border border-border bg-transparent"
        />
        <FieldError id="cat-color-error" :message="drawer.fieldErrors.value.colorHex" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Categorias.Descripcion') }}</span>
        <Textarea v-model="drawer.model.value.descripcion" rows="3" auto-resize />
      </label>
    </CrudDrawer>
  </section>
</template>
