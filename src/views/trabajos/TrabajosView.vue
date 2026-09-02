<script setup lang="ts">
import Column from 'primevue/column'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import FieldError from '@/components/domain/FieldError.vue'
import Divider from 'primevue/divider'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useClientesStore } from '@/stores/useClientesStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import {
  useTrabajosStore,
  type EstadoTrabajo,
  type TrabajoFiltro,
  type TrabajoInput,
  type TrabajoListItem,
} from '@/stores/useTrabajosStore'

/**
 * Jobs. See `docs/09-modulos-funcionales.md` §3.5.
 *
 * The customer filter goes through the site: a job has no customer of its own, and the legacy
 * denormalised column is what made this filter return the wrong rows.
 */

const { t } = useI18n()
const router = useRouter()
const { confirmDelete } = useConfirmDelete()
const { notify } = useApiError()
const store = useTrabajosStore()
const proyectos = useProyectosStore()
const clientes = useClientesStore()

const table = useServerTable<TrabajoFiltro, TrabajoListItem>({
  key: 'trabajos',
  initialFilter: { texto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fechaInicio', dir: 'Desc' },
})

const opcionesProyecto = ref<LookupItem[]>([])
const opcionesCliente = ref<LookupItem[]>([])

const estadoOptions = computed<{ label: string; value: EstadoTrabajo }[]>(() =>
  (['Presupuestado', 'EnProceso', 'Pausado', 'Finalizado', 'Cancelado'] as const).map((value) => ({
    label: t(`State.Trabajo.${value}`),
    value,
  })),
)

/** Narrowing by customer narrows the site selector too, which is the usual way in. */
watch(
  () => table.filter.value.clienteId,
  async (clienteId) => {
    try {
      opcionesProyecto.value = await proyectos.lookup(clienteId, undefined, 200)
    } catch (e) {
      notify(e)
    }
  },
)

type Model = TrabajoInput & { rowVersion?: string }

function hoy(): string {
  return new Date().toISOString().slice(0, 10)
}

const drawer = useCrudDrawer<Model>({
  entityKey: 'Entity.Trabajo',
  empty: () => ({
    proyectoId: '',
    descripcion: '',
    fechaInicio: hoy(),
    fechaFin: null,
    presupuesto: '0.0000',
  }),
  load: async (id) => {
    const d = await store.fetchOne(id)
    return {
      proyectoId: d.proyectoId,
      descripcion: d.descripcion,
      fechaInicio: d.fechaInicio,
      fechaFin: d.fechaFin,
      presupuesto: d.presupuesto,
      rowVersion: d.audit.rowVersion,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

async function cambiarEstado(row: TrabajoListItem, destino: EstadoTrabajo): Promise<void> {
  try {
    await store.transition(row.id, destino, row.rowVersion)
    table.reload()
  } catch (e) {
    notify(e)
  }
}

const filtrosActivos = computed(() =>
  Boolean(
    table.filter.value.texto ||
    table.filter.value.proyectoId ||
    table.filter.value.clienteId ||
    table.filter.value.estado ||
    table.filter.value.fechaDesde ||
    table.filter.value.fechaHasta,
  ),
)

/** The work orders of a job live under it: there is no global list of sheets. */
function verOrdenes(row: TrabajoListItem): void {
  void router.push({ name: 'trabajo-ordenes', params: { trabajoId: row.id } })
}

function onDelete(row: TrabajoListItem): void {
  confirmDelete({
    entityKey: 'Entity.Trabajo',
    label: row.descripcion,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

function trabajoContextMenu(row: TrabajoListItem) {
  return [
    { label: t('General.Edit'), icon: 'pi pi-pencil', command: () => drawer.openEdit(row.id) },
    { label: t('Ordenes.Title'), icon: 'pi pi-file-edit', command: () => verOrdenes(row) },
    { separator: true },
    { label: t('General.Delete'), icon: 'pi pi-trash', command: () => onDelete(row) },
  ]
}

useShortcuts({ 'ctrl+n': () => drawer.openCreate() })

onMounted(async () => {
  table.start()
  try {
    ;[opcionesProyecto.value, opcionesCliente.value] = await Promise.all([
      proyectos.lookup(undefined, undefined, 200),
      clientes.lookup(undefined, 200),
    ])
  } catch (e) {
    notify(e)
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Trabajos')" :subtitle="$t('Trabajos.Subtitle')">
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
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Trabajos.Cliente') }}</span>
        <Select
          v-model="table.filter.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Trabajos.Proyecto') }}</span>
        <Select
          v-model="table.filter.value.proyectoId"
          :options="opcionesProyecto"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Trabajos.Estado') }}</span>
        <Select
          v-model="table.filter.value.estado"
          :options="estadoOptions"
          option-label="label"
          option-value="value"
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Trabajos.Desde') }}</span>
        <DateInput v-model="table.filter.value.fechaDesde" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Trabajos.Hasta') }}</span>
        <DateInput v-model="table.filter.value.fechaHasta" />
      </label>
    </FilterBar>

    <Divider />

    <DataGrid
      :table="table"
      empty-key="Trabajos.Empty"
      class="flex-1"
      :context-menu-items="trabajoContextMenu"
      @row-edit="(row: any) => drawer.openEdit(row.id)"
    >
      <Column field="fechaInicio" :header="$t('Trabajos.FechaInicio')" sortable>
        <template #body="{ data }"><DateText :value="data.fechaInicio" /></template>
      </Column>
      <Column field="descripcion" :header="$t('Trabajos.Descripcion')" sortable />
      <Column field="proyectoNombre" :header="$t('Trabajos.Proyecto')" sortable>
        <template #body="{ data }">{{ data.proyectoNumero }} · {{ data.proyectoNombre }}</template>
      </Column>
      <Column field="clienteNombre" :header="$t('Trabajos.Cliente')" sortable />
      <Column field="presupuesto" :header="$t('Trabajos.Presupuesto')" sortable>
        <template #body="{ data }"><MoneyText :value="data.presupuesto" /></template>
      </Column>
      <Column field="estado" :header="$t('Trabajos.Estado')" sortable>
        <template #body="{ data }"><StatePill entity="Trabajo" :value="data.estado" /></template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button
            v-if="data.estado === 'Presupuestado' || data.estado === 'Pausado'"
            variant="ghost"
            size="sm"
            :title="$t('Actions.Trabajo.EnProceso')"
            @click="cambiarEstado(data, 'EnProceso')"
          >
            <AppIcon name="play" :size="14" />
          </Button>
          <Button
            v-if="data.estado === 'EnProceso'"
            variant="ghost"
            size="sm"
            :title="$t('Actions.Trabajo.Finalizado')"
            @click="cambiarEstado(data, 'Finalizado')"
          >
            <AppIcon name="check" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" :title="$t('Ordenes.Title')" @click="verOrdenes(data)">
            <AppIcon name="file-text" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :aria-label="$t('General.Edit')"
            @click="drawer.openEdit(data.id)"
          >
            <AppIcon name="pencil" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :aria-label="$t('General.Delete')"
            @click="onDelete(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <CrudDrawer :drawer="drawer" title-key="Entity.Trabajo">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Trabajos.Proyecto') }}</span>
        <Select
          v-model="drawer.model.value.proyectoId"
          :options="opcionesProyecto"
          option-label="label"
          option-value="id"
          filter
          :invalid="Boolean(drawer.fieldErrors.value.proyectoId)"
        />
        <FieldError id="trab-proyecto-error" :message="drawer.fieldErrors.value.proyectoId" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Trabajos.Descripcion') }}</span>
        <Textarea
          v-model="drawer.model.value.descripcion"
          rows="3"
          auto-resize
          :invalid="Boolean(drawer.fieldErrors.value.descripcion)"
          aria-describedby="trab-descripcion-error"
        />
        <FieldError id="trab-descripcion-error" :message="drawer.fieldErrors.value.descripcion" />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Trabajos.FechaInicio') }}</span>
          <DateInput
            v-model="drawer.model.value.fechaInicio"
            :invalid="Boolean(drawer.fieldErrors.value.fechaInicio)"
          />
          <FieldError id="trab-inicio-error" :message="drawer.fieldErrors.value.fechaInicio" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Trabajos.FechaFin') }}</span>
          <DateInput
            v-model="drawer.model.value.fechaFin"
            :invalid="Boolean(drawer.fieldErrors.value.fechaFin)"
          />
          <FieldError id="trab-fin-error" :message="drawer.fieldErrors.value.fechaFin" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Trabajos.Presupuesto') }}</span>
        <MoneyInput
          v-model="drawer.model.value.presupuesto"
          :min="0"
          :invalid="Boolean(drawer.fieldErrors.value.presupuesto)"
        />
        <FieldError id="trab-presupuesto-error" :message="drawer.fieldErrors.value.presupuesto" />
      </label>
    </CrudDrawer>
  </section>
</template>
