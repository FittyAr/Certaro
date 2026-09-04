<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'

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
  useClientesStore,
  type ClienteFiltro,
  type ClienteInput,
  type ClienteListItem,
} from '@/stores/useClientesStore'
import ClienteFormDrawer from './components/ClienteFormDrawer.vue'
import ClientesTable from './components/ClientesTable.vue'

/**
 * Customers. See `docs/09-modulos-funcionales.md` §3.3.
 *
 * Contacts are edited inside this form and saved with the customer: they are one aggregate, and
 * the legacy system's separate contact screen is what let a customer end up with two "main" ones.
 */

const { confirmDelete } = useConfirmDelete()
const router = useRouter()
const store = useClientesStore()

function verCuenta(clienteId: string): void {
  void router.push({ name: 'cliente-cuenta', params: { clienteId } })
}

function verFicha(clienteId: string): void {
  void router.push({ name: 'cliente-detalle', params: { clienteId } })
}

const table = useServerTable<ClienteFiltro, ClienteListItem>({
  key: 'clientes',
  initialFilter: { texto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'nombre', dir: 'Asc' },
})

type Model = ClienteInput & { rowVersion?: string }

function vacio(): Model {
  return {
    nombre: '',
    cuit: null,
    direccion: null,
    telefono: null,
    email: null,
    condicionIva: null,
    contactos: [],
  }
}

const drawer = useCrudDrawer<Model>({
  entityKey: 'Entity.Cliente',
  empty: vacio,
  load: async (id) => {
    const d = await store.fetchOne(id)
    return {
      nombre: d.nombre,
      cuit: d.cuit,
      direccion: d.direccion,
      telefono: d.telefono,
      email: d.email,
      condicionIva: d.condicionIva,
      contactos: d.contactos.map((c) => ({
        id: c.id,
        etiqueta: c.etiqueta,
        email: c.email,
        nombre: c.nombre,
        telefono: c.telefono,
        esPrincipal: c.esPrincipal,
      })),
      rowVersion: d.audit.rowVersion,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

const filtrosActivos = computed(
  () => Boolean(table.filter.value.texto) || table.filter.value.soloConDeuda === true,
)

function onDelete(row: ClienteListItem): void {
  confirmDelete({
    entityKey: 'Entity.Cliente',
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
    <PageHeader :title="$t('Menu.Clientes')" :subtitle="$t('Clientes.Subtitle')">
      <template #actions>
        <Button @click="drawer.openCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
        <HelpButton topic-id="clientes-overview" title="Ayuda sobre el Módulo de Clientes" />
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.Search') }}</span>
        <InputText v-model="table.filter.value.texto" :placeholder="$t('Clientes.BuscarHint')" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Clientes.CondicionIva') }}</span>
        <InputText v-model="table.filter.value.condicionIva" :placeholder="$t('General.All')" />
      </label>
      <label class="flex items-center gap-2 self-end pb-2 cursor-pointer select-none">
        <ToggleSwitch v-model="table.filter.value.soloConDeuda" />
        <span class="text-xs font-medium text-foreground/90">{{ $t('Clientes.SoloConDeuda') }}</span>
      </label>
    </FilterBar>

    <Divider />
    <ClientesTable
      :table="table"
      @ver-ficha="verFicha"
      @ver-cuenta="verCuenta"
      @edit="(id) => drawer.openEdit(id)"
      @delete="onDelete"
    />

    <ClienteFormDrawer :drawer="drawer" />
  </section>
</template>
