<script setup lang="ts">
import Column from 'primevue/column'
import { useI18n } from 'vue-i18n'
import DataGrid from '@/components/domain/DataGrid.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { ServerTable } from '@/composables/useServerTable'
import type { ClienteFiltro, ClienteListItem } from '@/stores/useClientesStore'

defineProps<{
  table: ServerTable<ClienteFiltro, ClienteListItem>
}>()

const emit = defineEmits<{
  (e: 'verFicha', id: string): void
  (e: 'verCuenta', id: string): void
  (e: 'edit', id: string): void
  (e: 'delete', row: ClienteListItem): void
}>()

const { t } = useI18n()

function clienteContextMenu(row: ClienteListItem) {
  return [
    {
      label: t('Clientes.VerFicha') || 'Ver Ficha de Cliente',
      icon: 'pi pi-eye',
      command: () => emit('verFicha', row.id),
    },
    { label: t('General.Edit'), icon: 'pi pi-pencil', command: () => emit('edit', row.id) },
    {
      label: t('Comercial.CuentaCorriente.Title'),
      icon: 'pi pi-wallet',
      command: () => emit('verCuenta', row.id),
    },
    { separator: true },
    {
      label: t('General.Delete'),
      icon: 'pi pi-trash',
      disabled: !row.puedeEliminarse,
      command: () => emit('delete', row),
    },
  ]
}
</script>

<template>
  <DataGrid
    :table="table"
    empty-key="Clientes.Empty"
    class="flex-1"
    :context-menu-items="clienteContextMenu"
    @row-edit="(row: any) => emit('edit', row.id)"
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
          :title="$t('Clientes.VerFicha') || 'Ver Ficha de Cliente'"
          @click="emit('verFicha', data.id)"
        >
          <AppIcon name="eye" :size="14" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :title="$t('Comercial.CuentaCorriente.Title')"
          @click="emit('verCuenta', data.id)"
        >
          <AppIcon name="wallet" :size="14" />
        </Button>
        <Button variant="ghost" size="sm" @click="emit('edit', data.id)">
          <AppIcon name="pencil" :size="14" />
        </Button>
        <!-- Disabled rather than hidden: the user sees the action exists and why it is off. -->
        <Button
          variant="ghost"
          size="sm"
          :disabled="!data.puedeEliminarse"
          :title="!data.puedeEliminarse ? $t('Clientes.NoBorrable') : undefined"
          @click="emit('delete', data)"
        >
          <AppIcon name="trash-2" :size="14" />
        </Button>
      </div>
    </template>
  </DataGrid>
</template>
