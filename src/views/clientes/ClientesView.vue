<script setup lang="ts">
import Column from 'primevue/column'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import FieldError from '@/components/domain/FieldError.vue'
import Divider from 'primevue/divider'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import {
  useClientesStore,
  type ClienteContactoInput,
  type ClienteFiltro,
  type ClienteInput,
  type ClienteListItem,
} from '@/stores/useClientesStore'

/**
 * Customers. See `docs/09-modulos-funcionales.md` §3.3.
 *
 * Contacts are edited inside this form and saved with the customer: they are one aggregate, and
 * the legacy system's separate contact screen is what let a customer end up with two "main" ones.
 */

const { confirmDelete } = useConfirmDelete()
const { t } = useI18n()
const router = useRouter()
const store = useClientesStore()

function verCuenta(clienteId: string): void {
  void router.push({ name: 'cliente-cuenta', params: { clienteId } })
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

function agregarContacto(): void {
  const contactos = drawer.model.value.contactos
  contactos.push({
    etiqueta: '',
    email: '',
    nombre: null,
    telefono: null,
    // The first contact is the main one; after that the user decides.
    esPrincipal: contactos.length === 0,
  })
}

function quitarContacto(indice: number): void {
  drawer.model.value.contactos.splice(indice, 1)
}

/** Exactly one contact can be the main one, so choosing a new one clears the previous. */
function marcarPrincipal(contacto: ClienteContactoInput): void {
  for (const otro of drawer.model.value.contactos) {
    otro.esPrincipal = otro === contacto
  }
}

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

function clienteContextMenu(row: ClienteListItem) {
  return [
    { label: t('General.Edit'), icon: 'pi pi-pencil', command: () => drawer.openEdit(row.id) },
    {
      label: t('Comercial.CuentaCorriente.Title'),
      icon: 'pi pi-wallet',
      command: () => verCuenta(row.id),
    },
    { separator: true },
    {
      label: t('General.Delete'),
      icon: 'pi pi-trash',
      disabled: !row.puedeEliminarse,
      command: () => onDelete(row),
    },
  ]
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
      <label class="flex items-center gap-2 self-end pb-2">
        <ToggleSwitch v-model="table.filter.value.soloConDeuda" />
        <span class="text-xs text-muted-foreground">{{ $t('Clientes.SoloConDeuda') }}</span>
      </label>
    </FilterBar>

    <Divider />

    <DataGrid
      :table="table"
      empty-key="Clientes.Empty"
      class="flex-1"
      :context-menu-items="clienteContextMenu"
      @row-edit="(row: any) => drawer.openEdit(row.id)"
    >
      <Column field="nombre" :header="$t('Clientes.Nombre')" sortable />
      <Column field="cuit" :header="$t('Clientes.Cuit')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.cuit ?? '—' }}</span>
        </template>
      </Column>
      <Column field="telefono" :header="$t('Clientes.Telefono')">
        <template #body="{ data }">{{ data.telefono ?? '—' }}</template>
      </Column>
      <Column field="email" :header="$t('Clientes.Email')">
        <template #body="{ data }">{{ data.email ?? '—' }}</template>
      </Column>
      <Column field="proyectosCount" :header="$t('Clientes.Proyectos')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.proyectosCount }}</span>
        </template>
      </Column>
      <Column field="deuda" :header="$t('Clientes.Deuda')" sortable>
        <template #body="{ data }"><MoneyText :value="data.deuda" /></template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button
            variant="ghost"
            size="sm"
            :title="$t('Comercial.CuentaCorriente.Title')"
            @click="verCuenta(data.id)"
          >
            <AppIcon name="wallet" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" @click="drawer.openEdit(data.id)">
            <AppIcon name="pencil" :size="14" />
          </Button>
          <!-- Disabled rather than hidden: the user sees the action exists and why it is off. -->
          <Button
            variant="ghost"
            size="sm"
            :disabled="!data.puedeEliminarse"
            :title="!data.puedeEliminarse ? $t('Clientes.NoBorrable') : undefined"
            @click="onDelete(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <CrudDrawer :drawer="drawer" title-key="Entity.Cliente">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Clientes.Nombre') }}</span>
        <InputText
          v-model="drawer.model.value.nombre"
          :invalid="Boolean(drawer.fieldErrors.value.nombre)"
          aria-describedby="cli-nombre-error"
        />
        <FieldError id="cli-nombre-error" :message="drawer.fieldErrors.value.nombre" />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Clientes.Cuit') }}</span>
          <InputText
            v-model="drawer.model.value.cuit"
            :invalid="Boolean(drawer.fieldErrors.value.cuit)"
            aria-describedby="cli-cuit-error"
          />
          <FieldError id="cli-cuit-error" :message="drawer.fieldErrors.value.cuit" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Clientes.CondicionIva') }}</span>
          <InputText v-model="drawer.model.value.condicionIva" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Clientes.Direccion') }}</span>
        <InputText v-model="drawer.model.value.direccion" />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Clientes.Telefono') }}</span>
          <InputText v-model="drawer.model.value.telefono" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Clientes.Email') }}</span>
          <InputText
            v-model="drawer.model.value.email"
            :invalid="Boolean(drawer.fieldErrors.value.email)"
            aria-describedby="cli-email-error"
          />
          <FieldError id="cli-email-error" :message="drawer.fieldErrors.value.email" />
        </label>
      </div>

      <div class="flex flex-col gap-2 border-t border-border pt-3">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium">{{ $t('Clientes.Contactos') }}</span>
          <Button variant="secondary" size="sm" @click="agregarContacto()">
            <AppIcon name="plus" :size="14" />
            {{ $t('Clientes.AgregarContacto') }}
          </Button>
        </div>

        <p v-if="!drawer.model.value.contactos?.length" class="text-xs text-muted-foreground">
          {{ $t('Clientes.SinContactos') }}
        </p>

        <div
          v-for="(contacto, indice) in (drawer.model.value.contactos ?? [])"
          :key="contacto.id ?? indice"
          class="flex flex-col gap-2 rounded-md border border-border p-3"
        >
          <div class="grid grid-cols-2 gap-2">
            <label class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">{{ $t('Clientes.Etiqueta') }}</span>
              <InputText
                v-model="contacto.etiqueta"
                :invalid="Boolean(drawer.fieldErrors.value[`contactos[${indice}].etiqueta`])"
              />
              <FieldError
                :id="`cli-contacto-${indice}-etiqueta-error`"
                :message="drawer.fieldErrors.value[`contactos[${indice}].etiqueta`]"
              />
            </label>
            <label class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">{{ $t('Clientes.Email') }}</span>
              <InputText
                v-model="contacto.email"
                :invalid="Boolean(drawer.fieldErrors.value[`contactos[${indice}].email`])"
              />
              <FieldError
                :id="`cli-contacto-${indice}-email-error`"
                :message="drawer.fieldErrors.value[`contactos[${indice}].email`]"
              />
            </label>
          </div>
          <div class="grid grid-cols-2 gap-2">
            <label class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">{{ $t('Clientes.Nombre') }}</span>
              <InputText v-model="contacto.nombre" />
            </label>
            <label class="flex flex-col gap-1">
              <span class="text-xs text-muted-foreground">{{ $t('Clientes.Telefono') }}</span>
              <InputText v-model="contacto.telefono" />
            </label>
          </div>
          <div class="flex items-center justify-between">
            <label class="flex items-center gap-2">
              <ToggleSwitch
                :model-value="contacto.esPrincipal"
                @update:model-value="marcarPrincipal(contacto)"
              />
              <span class="text-xs">{{ $t('Clientes.EsPrincipal') }}</span>
            </label>
            <Button variant="ghost" size="sm" @click="quitarContacto(indice)">
              <AppIcon name="trash-2" :size="14" />
            </Button>
          </div>
        </div>
      </div>
    </CrudDrawer>
  </section>
</template>
