<script setup lang="ts">
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, ref, watch } from 'vue'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DateInput from '@/components/domain/DateInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useMovimientosStore, type MovimientoInput } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import MovimientoImputacionSection from './MovimientoImputacionSection.vue'
import MovimientoMonedaSection from './MovimientoMonedaSection.vue'

const props = defineProps<{
  tipos: LookupItem[]
  categorias: LookupItem[]
  opcionesEmpleado: LookupItem[]
  opcionesCliente: LookupItem[]
  opcionesProyecto: LookupItem[]
}>()

const emit = defineEmits<{
  (e: 'saved'): void
}>()

const store = useMovimientosStore()
const proyectos = useProyectosStore()
const trabajos = useTrabajosStore()

const ADELANTO_ID = '00000000-0000-0000-0000-000000000003'

const opcionesTrabajo = ref<LookupItem[]>([])
const selectedProyectoId = ref<string | null>(null)
const opcionesProyecto = ref<LookupItem[]>([])

async function onClienteChange(): Promise<void> {
  const cId = drawer.model.value.clienteId
  selectedProyectoId.value = null
  drawer.model.value.trabajoId = null
  opcionesTrabajo.value = []
  if (cId) {
    opcionesProyecto.value = await proyectos.lookup(cId)
  } else {
    opcionesProyecto.value = await proyectos.lookup(undefined, undefined, 200)
  }
}

async function onProyectoChange(): Promise<void> {
  drawer.model.value.trabajoId = null
  if (!selectedProyectoId.value) {
    opcionesTrabajo.value = []
    return
  }
  try {
    opcionesTrabajo.value = await trabajos.lookup(selectedProyectoId.value)
    if (opcionesTrabajo.value.length > 0 && opcionesTrabajo.value[0]) {
      drawer.model.value.trabajoId = opcionesTrabajo.value[0].id
    }
    const p = await proyectos.fetchOne(selectedProyectoId.value)
    if (p?.clienteId) {
      drawer.model.value.clienteId = p.clienteId
    }
  } catch {
    opcionesTrabajo.value = []
  }
}

function vacio(): MovimientoInput & { rowVersion?: string } {
  selectedProyectoId.value = null
  opcionesTrabajo.value = []
  return {
    fecha: new Date().toISOString(),
    concepto: '',
    monto: '0.0000',
    cantidad: '1.0000',
    tipoMovimientoId: '',
    moneda: 'Ars',
    cotizacionAplicada: null,
    tipoConceptoPagoId: null,
    categoriaId: null,
    clienteId: null,
    trabajoId: null,
    empleadoId: null,
    facturaId: null,
  }
}

const drawer = useCrudDrawer<MovimientoInput & { rowVersion?: string }>({
  entityKey: 'Entity.Movimiento',
  empty: vacio,
  load: async (id) => {
    const d = await store.fetchOne(id)
    selectedProyectoId.value = null
    opcionesTrabajo.value = []
    if (d.trabajoId) {
      try {
        const trab = await trabajos.fetchOne(d.trabajoId)
        selectedProyectoId.value = trab.proyectoId
        opcionesTrabajo.value = await trabajos.lookup(trab.proyectoId)
      } catch {
        // Fallback
      }
    }
    return {
      fecha: d.fecha,
      concepto: d.concepto,
      monto: d.monto,
      cantidad: d.cantidad,
      tipoMovimientoId: d.tipoMovimientoId,
      moneda: d.moneda,
      cotizacionAplicada: d.cotizacionAplicada,
      tipoConceptoPagoId: d.tipoConceptoPagoId,
      categoriaId: d.categoriaId,
      clienteId: d.clienteId,
      trabajoId: d.trabajoId,
      empleadoId: d.empleadoId,
      facturaId: d.facturaId,
      rowVersion: d.rowVersion,
    }
  },
  create: (dto) => {
    if (
      selectedProyectoId.value &&
      !dto.trabajoId &&
      opcionesTrabajo.value.length > 0 &&
      opcionesTrabajo.value[0]
    ) {
      dto.trabajoId = opcionesTrabajo.value[0].id
    }
    return store.create(dto)
  },
  update: (id, dto) => {
    if (
      selectedProyectoId.value &&
      !dto.trabajoId &&
      opcionesTrabajo.value.length > 0 &&
      opcionesTrabajo.value[0]
    ) {
      dto.trabajoId = opcionesTrabajo.value[0].id
    }
    return store.update(id, dto, dto.rowVersion ?? '')
  },
  onSaved: () => emit('saved'),
})

const esAdelanto = computed(() => drawer.model.value.tipoMovimientoId === ADELANTO_ID)

watch(
  () => props.opcionesProyecto,
  (val) => {
    if (!drawer.model.value.clienteId) {
      opcionesProyecto.value = val
    }
  },
  { immediate: true },
)

async function openCreate(preset?: { proyectoId?: string; clienteId?: string }): Promise<void> {
  drawer.openCreate()
  if (preset?.proyectoId) {
    selectedProyectoId.value = preset.proyectoId
    await onProyectoChange()
    if (preset.clienteId) {
      drawer.model.value.clienteId = preset.clienteId
    }
  } else {
    opcionesProyecto.value = [...props.opcionesProyecto]
  }
}

async function openEdit(id: string): Promise<void> {
  drawer.openEdit(id)
  if (drawer.model.value.clienteId) {
    opcionesProyecto.value = await proyectos.lookup(drawer.model.value.clienteId)
  } else {
    opcionesProyecto.value = [...props.opcionesProyecto]
  }
}

defineExpose({
  openCreate,
  openEdit,
})
</script>

<template>
  <CrudDrawer :drawer="drawer" title-key="Entity.Movimiento">
    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Movimientos.Fecha') }}</span>
      <DateInput
        v-model="drawer.model.value.fecha"
        instant
        show-time
        :invalid="Boolean(drawer.fieldErrors.value.fecha)"
      />
      <FieldError id="mov-fecha-error" :message="drawer.fieldErrors.value.fecha" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Movimientos.Concepto') }}</span>
      <InputText
        v-model="drawer.model.value.concepto"
        :invalid="Boolean(drawer.fieldErrors.value.concepto)"
        aria-describedby="mov-concepto-error"
      />
      <FieldError id="mov-concepto-error" :message="drawer.fieldErrors.value.concepto" />
    </label>

    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Monto') }}</span>
        <MoneyInput
          v-model="drawer.model.value.monto"
          :min="0"
          :invalid="Boolean(drawer.fieldErrors.value.monto)"
        />
        <FieldError id="mov-monto-error" :message="drawer.fieldErrors.value.monto" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Cantidad') }}</span>
        <InputNumber
          :model-value="Number(drawer.model.value.cantidad)"
          :min="0"
          :min-fraction-digits="2"
          :max-fraction-digits="4"
          :invalid="Boolean(drawer.fieldErrors.value.cantidad)"
          fluid
          input-class="tabular-nums text-right"
          @update:model-value="(value) => (drawer.model.value.cantidad = (value ?? 0).toFixed(4))"
        />
        <FieldError id="mov-cantidad-error" :message="drawer.fieldErrors.value.cantidad" />
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Movimientos.Tipo') }}</span>
      <Select
        v-model="drawer.model.value.tipoMovimientoId"
        :options="tipos"
        option-label="label"
        option-value="id"
        :invalid="Boolean(drawer.fieldErrors.value.tipoMovimientoId)"
      />
      <FieldError id="mov-tipo-error" :message="drawer.fieldErrors.value.tipoMovimientoId" />
    </label>

    <!-- Empleado selector (Required if Adelanto, selectable anytime) -->
    <label v-if="esAdelanto || drawer.model.value.empleadoId || drawer.open.value" class="flex flex-col gap-1">
      <span class="text-sm">
        {{ $t('Movimientos.Empleado') }}
        <span v-if="esAdelanto" class="text-destructive">*</span>
      </span>
      <Select
        v-model="drawer.model.value.empleadoId"
        :options="opcionesEmpleado"
        option-label="label"
        option-value="id"
        filter
        show-clear
        :placeholder="esAdelanto ? $t('Movimientos.EmpleadoRequeridoPlaceholder') : $t('General.None')"
        :invalid="Boolean(drawer.fieldErrors.value.empleadoId)"
      />
      <FieldError id="mov-empleado-error" :message="drawer.fieldErrors.value.empleadoId" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Movimientos.Categoria') }}</span>
      <Select
        v-model="drawer.model.value.categoriaId"
        :options="categorias"
        option-label="label"
        option-value="id"
        filter
        :invalid="Boolean(drawer.fieldErrors.value.categoriaId)"
      />
      <FieldError id="mov-categoria-error" :message="drawer.fieldErrors.value.categoriaId" />
    </label>

    <MovimientoImputacionSection
      v-model:cliente-id="drawer.model.value.clienteId"
      v-model:selected-proyecto-id="selectedProyectoId"
      v-model:trabajo-id="drawer.model.value.trabajoId"
      :opciones-cliente="opcionesCliente"
      :opciones-proyecto="opcionesProyecto"
      :opciones-trabajo="opcionesTrabajo"
      @cliente-change="onClienteChange"
      @proyecto-change="onProyectoChange"
    />

    <MovimientoMonedaSection
      v-model:moneda="drawer.model.value.moneda"
      v-model:cotizacion-aplicada="drawer.model.value.cotizacionAplicada"
      :field-error-cotizacion="drawer.fieldErrors.value.cotizacionAplicada"
    />
  </CrudDrawer>
</template>
