<script setup lang="ts">
import Column from 'primevue/column'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import FieldError from '@/components/domain/FieldError.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import {
  useTiposMovimientoStore,
  type TipoMovimientoFiltro,
  type TipoMovimientoInput,
  type TipoMovimientoListItem,
} from '@/stores/useTiposMovimientoStore'

/**
 * Administration of the movement types. See `docs/09-modulos-funcionales.md` §3.14.
 *
 * The four seeded rows cannot be deleted and cannot change their sign: the historical balance was
 * computed with it, so flipping it would rewrite every past total.
 */

const { t } = useI18n()
const { confirmDelete } = useConfirmDelete()
const store = useTiposMovimientoStore()

const table = useServerTable<TipoMovimientoFiltro, TipoMovimientoListItem>({
  key: 'tiposMovimiento',
  initialFilter: { texto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'nombre', dir: 'Asc' },
})

const drawer = useCrudDrawer<TipoMovimientoInput & { rowVersion?: string }>({
  entityKey: 'Entity.TipoMovimiento',
  empty: () => ({ nombre: '', descripcion: null, esIngreso: false }),
  load: async (id) => {
    const detalle = await store.fetchOne(id)
    return {
      nombre: detalle.nombre,
      descripcion: detalle.descripcion,
      esIngreso: detalle.esIngreso,
      rowVersion: detalle.audit.rowVersion,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

const signoOptions = computed(() => [
  { label: t('TiposMovimiento.Signo.Ingreso'), value: true },
  { label: t('TiposMovimiento.Signo.Gasto'), value: false },
])

const filtrosActivos = computed(
  () => Boolean(table.filter.value.texto) || table.filter.value.esIngreso !== undefined,
)

function onDelete(row: TipoMovimientoListItem): void {
  confirmDelete({
    entityKey: 'Entity.TipoMovimiento',
    label: row.nombre,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

useShortcuts({ 'ctrl+n': () => drawer.openCreate() })

onMounted(() => table.start())
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.MovementTypes')" :subtitle="$t('TiposMovimiento.Subtitle')">
      <template #actions>
        <Button @click="drawer.openCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
        <HelpButton topic-id="tipos-movimiento-overview" title="Ayuda sobre Tipos de Movimiento de Caja" />
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.Search') }}</span>
        <InputText v-model="table.filter.value.texto" :placeholder="$t('General.Search')" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('TiposMovimiento.Signo.Label') }}</span>
        <Select
          v-model="table.filter.value.esIngreso"
          :options="signoOptions"
          option-label="label"
          option-value="value"
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
    </FilterBar>

    <DataGrid
      :table="table"
      empty-key="TiposMovimiento.Empty"
      class="flex-1"
      @row-edit="(row: any) => drawer.openEdit(row.id)"
    >
      <Column field="nombre" :header="$t('TiposMovimiento.Nombre')" sortable />
      <Column field="descripcion" :header="$t('TiposMovimiento.Descripcion')">
        <template #body="{ data }">
          <span class="text-muted-foreground">{{ data.descripcion ?? '—' }}</span>
        </template>
      </Column>
      <Column field="esIngreso" :header="$t('TiposMovimiento.Signo.Label')" sortable>
        <template #body="{ data }">
          {{
            data.esIngreso ? $t('TiposMovimiento.Signo.Ingreso') : $t('TiposMovimiento.Signo.Gasto')
          }}
        </template>
      </Column>
      <Column field="movimientosCount" :header="$t('TiposMovimiento.EnUso')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.movimientosCount }}</span>
        </template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button
            variant="ghost"
            size="sm"
            :aria-label="$t('General.Edit')"
            @click="drawer.openEdit(data.id)"
          >
            <AppIcon name="pencil" :size="14" />
          </Button>
          <!-- Disabled rather than hidden: the user sees the action exists and why it is off. -->
          <Button
            variant="ghost"
            size="sm"
            :disabled="!data.puedeEliminarse"
            :title="data.esSistema ? $t('TiposMovimiento.NoBorrableSistema') : undefined"
            :aria-label="$t('General.Delete')"
            @click="onDelete(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <CrudDrawer :drawer="drawer" title-key="Entity.TipoMovimiento">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('TiposMovimiento.Nombre') }}</span>
        <InputText
          v-model="drawer.model.value.nombre"
          :invalid="Boolean(drawer.fieldErrors.value.nombre)"
          aria-describedby="tm-nombre-error"
        />
        <FieldError id="tm-nombre-error" :message="drawer.fieldErrors.value.nombre" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('TiposMovimiento.Descripcion') }}</span>
        <Textarea v-model="drawer.model.value.descripcion" rows="3" auto-resize />
      </label>

      <label class="flex items-center gap-2">
        <ToggleSwitch v-model="drawer.model.value.esIngreso" />
        <span class="text-sm">{{ $t('TiposMovimiento.EsIngreso') }}</span>
      </label>
    </CrudDrawer>
  </section>
</template>
