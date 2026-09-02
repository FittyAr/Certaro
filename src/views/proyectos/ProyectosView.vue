<script setup lang="ts">
import Divider from 'primevue/divider'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { useConfirm } from 'primevue/useconfirm'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import FieldError from '@/components/domain/FieldError.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import ProyectosTreeTable from '@/components/domain/ProyectosTreeTable.vue'
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
  useProyectosStore,
  type EstadoProyecto,
  type ProyectoFiltro,
  type ProyectoInput,
  type ProyectoListItem,
} from '@/stores/useProyectosStore'

/**
 * Sites. See `docs/09-modulos-funcionales.md` §3.4.
 *
 * The state is not a field of the form: it moves only along the documented edges, so the actions
 * come from what the backend says is reachable rather than from a dropdown of the whole enum.
 */

const { t } = useI18n()
const confirm = useConfirm()
const router = useRouter()
const { confirmDelete } = useConfirmDelete()
const { notify } = useApiError()
const store = useProyectosStore()
const clientes = useClientesStore()

const table = useServerTable<ProyectoFiltro, ProyectoListItem>({
  key: 'proyectos',
  initialFilter: { texto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'numero', dir: 'Desc' },
})

const opcionesCliente = ref<LookupItem[]>([])

const estadoOptions = computed<{ label: string; value: EstadoProyecto }[]>(() =>
  (['Activa', 'Pausada', 'Finalizada', 'Cancelada'] as const).map((value) => ({
    label: t(`State.Proyecto.${value}`),
    value,
  })),
)

type Model = ProyectoInput & { rowVersion?: string }

const drawer = useCrudDrawer<Model>({
  entityKey: 'Entity.Proyecto',
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
async function cambiarEstado(row: ProyectoListItem, destino: EstadoProyecto): Promise<void> {
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
    message: t('Proyectos.ConfirmarCascada', { count: row.trabajosCount }),
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

function onDelete(row: ProyectoListItem): void {
  confirmDelete({
    entityKey: 'Entity.Proyecto',
    label: `${row.numero} · ${row.nombre}`,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

function onTrabajoNavigate(trabajo: { id: string }): void {
  void router.push({ name: 'trabajo-detalle', params: { trabajoId: trabajo.id } })
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
    <PageHeader :title="$t('Menu.Proyectos')" :subtitle="$t('Proyectos.Subtitle')">
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
        <InputText v-model="table.filter.value.texto" :placeholder="$t('Proyectos.BuscarHint')" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Cliente') }}</span>
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
        <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Estado') }}</span>
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
        <span class="text-xs text-muted-foreground">{{ $t('Proyectos.SoloActivas') }}</span>
      </label>
    </FilterBar>

    <Divider />

    <ProyectosTreeTable
      :table="table"
      class="flex-1"
      @proyecto-edit="(row) => drawer.openEdit(row.id)"
      @proyecto-delete="onDelete"
      @proyecto-transition="(row, destino) => cambiarEstado(row, destino as EstadoProyecto)"
      @trabajo-navigate="onTrabajoNavigate"
    />

    <CrudDrawer :drawer="drawer" title-key="Entity.Proyecto">
      <div class="grid grid-cols-[8rem_1fr] gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Proyectos.Numero') }}</span>
          <InputNumber
            v-model="drawer.model.value.numero"
            :min="1"
            :use-grouping="false"
            :invalid="Boolean(drawer.fieldErrors.value.numero)"
            fluid
            input-class="tabular-nums"
          />
          <FieldError id="proyecto-numero-error" :message="drawer.fieldErrors.value.numero" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Proyectos.Nombre') }}</span>
          <InputText
            v-model="drawer.model.value.nombre"
            :invalid="Boolean(drawer.fieldErrors.value.nombre)"
            aria-describedby="proyecto-nombre-error"
          />
          <FieldError id="proyecto-nombre-error" :message="drawer.fieldErrors.value.nombre" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Proyectos.Cliente') }}</span>
        <Select
          v-model="drawer.model.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          filter
          :invalid="Boolean(drawer.fieldErrors.value.clienteId)"
        />
        <FieldError id="proyecto-cliente-error" :message="drawer.fieldErrors.value.clienteId" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Proyectos.Direccion') }}</span>
        <InputText v-model="drawer.model.value.direccion" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Proyectos.Localidad') }}</span>
        <InputText v-model="drawer.model.value.localidad" />
      </label>
    </CrudDrawer>
  </section>
</template>
