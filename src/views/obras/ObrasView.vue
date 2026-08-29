<script setup lang="ts">
import Column from 'primevue/column'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { useConfirm } from 'primevue/useconfirm'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import FieldError from '@/components/domain/FieldError.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
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
import { useClientesStore } from '@/stores/useClientesStore'
import type { LookupItem } from '@/stores/useCatalogStore'
import {
  useObrasStore,
  type EstadoObra,
  type ObraFiltro,
  type ObraInput,
  type ObraListItem,
} from '@/stores/useObrasStore'

/**
 * Sites. See `docs/09-modulos-funcionales.md` §3.4.
 *
 * The state is not a field of the form: it moves only along the documented edges, so the actions
 * come from what the backend says is reachable rather than from a dropdown of the whole enum.
 */

const { t } = useI18n()
const confirm = useConfirm()
const { confirmDelete } = useConfirmDelete()
const { notify } = useApiError()
const store = useObrasStore()
const clientes = useClientesStore()

const table = useServerTable<ObraFiltro, ObraListItem>({
  key: 'obras',
  initialFilter: { texto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'numero', dir: 'Desc' },
})

const opcionesCliente = ref<LookupItem[]>([])

const estadoOptions = computed<{ label: string; value: EstadoObra }[]>(() =>
  (['Activa', 'Pausada', 'Finalizada', 'Cancelada'] as const).map((value) => ({
    label: t(`State.Obra.${value}`),
    value,
  })),
)

type Model = ObraInput & { rowVersion?: string }

const drawer = useCrudDrawer<Model>({
  entityKey: 'Entity.Obra',
  empty: () => ({ numero: 0, nombre: '', direccion: null, localidad: null, clienteId: '' }),
  load: async (id) => {
    const d = await store.fetchOne(id)
    return {
      numero: d.numero,
      nombre: d.nombre,
      direccion: d.direccion,
      localidad: d.localidad,
      clienteId: d.clienteId,
      rowVersion: d.audit.rowVersion,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

/** The create form arrives with the next free number filled in, so nobody has to guess it. */
async function abrirCreate(): Promise<void> {
  drawer.openCreate()
  try {
    drawer.model.value.numero = await store.siguienteNumero()
  } catch (e) {
    notify(e)
  }
}

/**
 * Finalising or cancelling a site with open jobs closes them too, so it is asked about first.
 * The answer is what `cascada` carries; without it the backend refuses the move.
 */
async function cambiarEstado(row: ObraListItem, destino: EstadoObra): Promise<void> {
  const cierra = destino === 'Finalizada' || destino === 'Cancelada'
  const aplicar = async (cascada: boolean) => {
    try {
      await store.transition(row.id, destino, row.rowVersion, cascada)
      table.reload()
    } catch (e) {
      notify(e)
    }
  }

  if (!cierra || row.trabajosCount === 0) {
    await aplicar(false)
    return
  }

  confirm.require({
    header: t('General.Confirm'),
    message: t('Obras.ConfirmarCascada', { count: row.trabajosCount }),
    acceptLabel: t('General.Continue'),
    rejectLabel: t('General.Cancel'),
    accept: () => void aplicar(true),
  })
}

const filtrosActivos = computed(() =>
  Boolean(
    table.filter.value.texto ||
    table.filter.value.clienteId ||
    table.filter.value.estado ||
    table.filter.value.soloActivas,
  ),
)

function onDelete(row: ObraListItem): void {
  confirmDelete({
    entityKey: 'Entity.Obra',
    label: `${row.numero} · ${row.nombre}`,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

useShortcuts({ 'ctrl+n': () => void abrirCreate() })

onMounted(async () => {
  table.start()
  try {
    opcionesCliente.value = await clientes.lookup(undefined, 200)
  } catch (e) {
    notify(e)
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Obras')" :subtitle="$t('Obras.Subtitle')">
      <template #actions>
        <Button @click="abrirCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.Search') }}</span>
        <InputText v-model="table.filter.value.texto" :placeholder="$t('Obras.BuscarHint')" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Obras.Cliente') }}</span>
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
        <span class="text-xs text-muted-foreground">{{ $t('Obras.Estado') }}</span>
        <Select
          v-model="table.filter.value.estado"
          :options="estadoOptions"
          option-label="label"
          option-value="value"
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex items-center gap-2 self-end pb-2">
        <ToggleSwitch v-model="table.filter.value.soloActivas" />
        <span class="text-xs text-muted-foreground">{{ $t('Obras.SoloActivas') }}</span>
      </label>
    </FilterBar>

    <DataGrid
      :table="table"
      empty-key="Obras.Empty"
      class="flex-1"
      @row-edit="(row) => drawer.openEdit(row.id)"
    >
      <Column field="numero" :header="$t('Obras.Numero')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.numero }}</span>
        </template>
      </Column>
      <Column field="nombre" :header="$t('Obras.Nombre')" sortable />
      <Column field="clienteNombre" :header="$t('Obras.Cliente')" sortable />
      <Column field="localidad" :header="$t('Obras.Localidad')">
        <template #body="{ data }">{{ data.localidad ?? '—' }}</template>
      </Column>
      <Column field="estado" :header="$t('Obras.Estado')" sortable>
        <template #body="{ data }"><StatePill entity="Obra" :value="data.estado" /></template>
      </Column>
      <Column field="trabajosCount" :header="$t('Obras.Trabajos')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.trabajosCount }}</span>
        </template>
      </Column>
      <Column field="rentabilidad" :header="$t('Obras.Rentabilidad')" sortable>
        <template #body="{ data }"><MoneyText :value="data.rentabilidad" colored /></template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button
            v-if="data.estado !== 'Finalizada' && data.estado !== 'Cancelada'"
            variant="ghost"
            size="sm"
            :title="$t('Actions.Obra.Finalizada')"
            @click="cambiarEstado(data, 'Finalizada')"
          >
            <AppIcon name="check" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" @click="drawer.openEdit(data.id)">
            <AppIcon name="pencil" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :disabled="!data.puedeEliminarse"
            :title="!data.puedeEliminarse ? $t('Obras.NoBorrable') : undefined"
            @click="onDelete(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <CrudDrawer :drawer="drawer" title-key="Entity.Obra">
      <div class="grid grid-cols-[8rem_1fr] gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Obras.Numero') }}</span>
          <InputNumber
            v-model="drawer.model.value.numero"
            :min="1"
            :use-grouping="false"
            :invalid="Boolean(drawer.fieldErrors.value.numero)"
            fluid
            input-class="tabular-nums"
          />
          <FieldError id="obra-numero-error" :message="drawer.fieldErrors.value.numero" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Obras.Nombre') }}</span>
          <InputText
            v-model="drawer.model.value.nombre"
            :invalid="Boolean(drawer.fieldErrors.value.nombre)"
            aria-describedby="obra-nombre-error"
          />
          <FieldError id="obra-nombre-error" :message="drawer.fieldErrors.value.nombre" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Obras.Cliente') }}</span>
        <Select
          v-model="drawer.model.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          filter
          :invalid="Boolean(drawer.fieldErrors.value.clienteId)"
        />
        <FieldError id="obra-cliente-error" :message="drawer.fieldErrors.value.clienteId" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Obras.Direccion') }}</span>
        <InputText v-model="drawer.model.value.direccion" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Obras.Localidad') }}</span>
        <InputText v-model="drawer.model.value.localidad" />
      </label>
    </CrudDrawer>
  </section>
</template>
