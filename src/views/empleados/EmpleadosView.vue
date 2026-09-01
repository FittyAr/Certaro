<script setup lang="ts">
import Column from 'primevue/column'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted } from 'vue'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import {
  useEmpleadosStore,
  type EmpleadoFiltro,
  type EmpleadoInput,
  type EmpleadoListItem,
  type FrecuenciaPago,
} from '@/stores/useEmpleadosStore'

/**
 * Employees. See `docs/09-modulos-funcionales.md` §3.9.
 *
 * The special-day multipliers live on the employee and not only in the configuration: two workers
 * can be paid differently for the same Sunday, and the settlement freezes whichever applied.
 */

const { confirmDelete } = useConfirmDelete()
const store = useEmpleadosStore()

const table = useServerTable<EmpleadoFiltro, EmpleadoListItem>({
  key: 'empleados',
  initialFilter: { texto: '', activo: true },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'nombre', dir: 'Asc' },
})

const FRECUENCIAS: FrecuenciaPago[] = ['Diario', 'Semanal', 'Quincenal', 'Mensual']

type Model = EmpleadoInput & { rowVersion?: string; tarifaSugerida?: string }

function hoy(): string {
  return new Date().toISOString().slice(0, 10)
}

function vacio(): Model {
  return {
    nombre: '',
    dni: null,
    cargo: null,
    sueldoBase: '0.0000',
    pagoFrecuencia: 'Quincenal',
    tarifaDiaria: '0.0000',
    multiplicadorSabado: '1.5000',
    multiplicadorDomingo: '2.0000',
    multiplicadorFeriado: '2.0000',
    email: null,
    telefono: null,
    fechaIngreso: hoy(),
    fechaEgreso: null,
    activo: true,
  }
}

const drawer = useCrudDrawer<Model>({
  entityKey: 'Entity.Empleado',
  empty: vacio,
  load: async (id) => {
    const d = await store.fetchOne(id)
    return {
      nombre: d.nombre,
      dni: d.dni,
      cargo: d.cargo,
      sueldoBase: d.sueldoBase,
      pagoFrecuencia: d.pagoFrecuencia,
      tarifaDiaria: d.tarifaDiaria,
      multiplicadorSabado: d.multiplicadorSabado,
      multiplicadorDomingo: d.multiplicadorDomingo,
      multiplicadorFeriado: d.multiplicadorFeriado,
      email: d.email,
      telefono: d.telefono,
      fechaIngreso: d.fechaIngreso,
      fechaEgreso: d.fechaEgreso,
      activo: d.activo,
      rowVersion: d.audit.rowVersion,
      tarifaSugerida: d.tarifaDiariaSugerida,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

/** Offered, never applied on its own: the rate the payroll uses is the one on the form. */
function usarSugerida(): void {
  const sugerida = drawer.model.value.tarifaSugerida
  if (sugerida) drawer.model.value.tarifaDiaria = sugerida
}

const filtrosActivos = computed(
  () =>
    Boolean(table.filter.value.texto) ||
    Boolean(table.filter.value.cargo) ||
    table.filter.value.activo !== true,
)

function onDelete(row: EmpleadoListItem): void {
  confirmDelete({
    entityKey: 'Entity.Empleado',
    label: row.nombre,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

useShortcuts({ 'ctrl+n': () => drawer.openCreate() })

onMounted(async () => {
  table.start()
  await store.fetchCargos()
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Empleados')" :subtitle="$t('Empleados.Subtitle')">
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
        <InputText v-model="table.filter.value.texto" :placeholder="$t('Empleados.BuscarHint')" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Empleados.Cargo') }}</span>
        <Select
          v-model="table.filter.value.cargo"
          :options="store.cargos"
          :placeholder="$t('General.All')"
          show-clear
        />
      </label>
      <label class="flex items-center gap-2 self-end pb-2">
        <ToggleSwitch v-model="table.filter.value.activo" />
        <span class="text-xs text-muted-foreground">{{ $t('Empleados.SoloActivos') }}</span>
      </label>
    </FilterBar>

    <DataGrid
      :table="table"
      empty-key="Empleados.Empty"
      class="flex-1"
      @row-edit="(row) => drawer.openEdit(row.id)"
    >
      <Column field="nombre" :header="$t('Empleados.Nombre')" sortable />
      <Column field="dni" :header="$t('Empleados.Dni')">
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.dni ?? '—' }}</span>
        </template>
      </Column>
      <Column field="cargo" :header="$t('Empleados.Cargo')" sortable>
        <template #body="{ data }">{{ data.cargo ?? '—' }}</template>
      </Column>
      <Column field="tarifaDiaria" :header="$t('Empleados.TarifaDiaria')" sortable>
        <template #body="{ data }"><MoneyText :value="data.tarifaDiaria" /></template>
      </Column>
      <Column field="pagoFrecuencia" :header="$t('Empleados.PagoFrecuencia')">
        <template #body="{ data }">
          {{ $t(`FrecuenciaPago.${data.pagoFrecuencia}`) }}
        </template>
      </Column>
      <Column field="fechaIngreso" :header="$t('Empleados.FechaIngreso')" sortable>
        <template #body="{ data }"><DateText :value="data.fechaIngreso" /></template>
      </Column>
      <Column field="activo" :header="$t('Empleados.Activo')">
        <template #body="{ data }">
          {{ data.activo ? $t('General.Yes') : $t('General.No') }}
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

    <CrudDrawer :drawer="drawer" title-key="Entity.Empleado">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Empleados.Nombre') }}</span>
        <InputText
          v-model="drawer.model.value.nombre"
          :invalid="Boolean(drawer.fieldErrors.value.nombre)"
          aria-describedby="emp-nombre-error"
        />
        <FieldError id="emp-nombre-error" :message="drawer.fieldErrors.value.nombre" />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.Dni') }}</span>
          <InputText
            v-model="drawer.model.value.dni"
            :invalid="Boolean(drawer.fieldErrors.value.dni)"
            aria-describedby="emp-dni-error"
          />
          <FieldError id="emp-dni-error" :message="drawer.fieldErrors.value.dni" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.Cargo') }}</span>
          <InputText v-model="drawer.model.value.cargo" />
        </label>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.SueldoBase') }}</span>
          <MoneyInput v-model="drawer.model.value.sueldoBase" :min="0" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.PagoFrecuencia') }}</span>
          <Select
            v-model="drawer.model.value.pagoFrecuencia"
            :options="FRECUENCIAS"
            :option-label="(o: FrecuenciaPago) => $t(`FrecuenciaPago.${o}`)"
          />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Empleados.TarifaDiaria') }}</span>
        <MoneyInput
          v-model="drawer.model.value.tarifaDiaria"
          :min="0"
          :invalid="Boolean(drawer.fieldErrors.value.tarifaDiaria)"
          aria-describedby="emp-tarifa-error"
        />
        <FieldError id="emp-tarifa-error" :message="drawer.fieldErrors.value.tarifaDiaria" />
        <button
          v-if="drawer.model.value.tarifaSugerida"
          type="button"
          class="self-start text-xs text-muted-foreground underline"
          @click="usarSugerida()"
        >
          {{ $t('Empleados.UsarTarifaSugerida', { valor: drawer.model.value.tarifaSugerida }) }}
        </button>
      </label>

      <div class="grid grid-cols-3 gap-3 border-t border-border pt-3">
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Empleados.MultSabado') }}</span>
          <DecimalInput v-model="drawer.model.value.multiplicadorSabado" :min="0" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Empleados.MultDomingo') }}</span>
          <DecimalInput v-model="drawer.model.value.multiplicadorDomingo" :min="0" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Empleados.MultFeriado') }}</span>
          <DecimalInput v-model="drawer.model.value.multiplicadorFeriado" :min="0" />
        </label>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.Email') }}</span>
          <InputText
            v-model="drawer.model.value.email"
            :invalid="Boolean(drawer.fieldErrors.value.email)"
            aria-describedby="emp-email-error"
          />
          <FieldError id="emp-email-error" :message="drawer.fieldErrors.value.email" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.Telefono') }}</span>
          <InputText v-model="drawer.model.value.telefono" />
        </label>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.FechaIngreso') }}</span>
          <DateInput v-model="drawer.model.value.fechaIngreso" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.FechaEgreso') }}</span>
          <DateInput
            v-model="drawer.model.value.fechaEgreso"
            :invalid="Boolean(drawer.fieldErrors.value.fechaEgreso)"
          />
          <FieldError id="emp-egreso-error" :message="drawer.fieldErrors.value.fechaEgreso" />
        </label>
      </div>

      <label class="flex items-center gap-2">
        <ToggleSwitch v-model="drawer.model.value.activo" />
        <span class="text-sm">{{ $t('Empleados.Activo') }}</span>
      </label>
    </CrudDrawer>
  </section>
</template>
