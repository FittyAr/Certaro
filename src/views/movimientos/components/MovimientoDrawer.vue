<script setup lang="ts">
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DateInput from '@/components/domain/DateInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useMovimientosStore, type Moneda, type MovimientoInput } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'

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

const { t } = useI18n()
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

const monedaOptions = computed<{ label: string; value: Moneda }[]>(() => [
  { label: t('Movimientos.Moneda.Ars'), value: 'Ars' },
  { label: t('Movimientos.Moneda.Usd'), value: 'Usd' },
])

const pideCotizacion = computed(() => drawer.model.value.moneda === 'Usd')

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

    <!-- Imputación opcional a Cliente / Proyecto / Trabajo -->
    <div class="space-y-3 rounded-md border border-border/70 bg-muted/20 p-3">
      <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        {{ $t('Movimientos.ImputacionOpcional') }}
      </span>

      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Cliente') }}</span>
        <Select
          v-model="drawer.model.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.None')"
          @change="onClienteChange()"
        />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Proyecto') }}</span>
          <Select
            v-model="selectedProyectoId"
            :options="opcionesProyecto"
            option-label="label"
            option-value="id"
            filter
            show-clear
            :placeholder="$t('General.None')"
            @change="onProyectoChange()"
          />
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Trabajo') }}</span>
          <Select
            v-model="drawer.model.value.trabajoId"
            :options="opcionesTrabajo"
            option-label="label"
            option-value="id"
            filter
            show-clear
            :placeholder="$t('General.None')"
            :disabled="!selectedProyectoId && opcionesTrabajo.length === 0"
          />
        </label>
      </div>
      <p
        v-if="selectedProyectoId && opcionesTrabajo.length === 0"
        class="rounded-md border border-warning/30 bg-warning/10 p-2 text-xs text-warning"
      >
        {{ $t('Movimientos.ProyectoSinTrabajosAviso') || 'Este proyecto no tiene trabajos creados aún. Recuerda crear al menos un trabajo en el proyecto para que los gastos se imputen a la caja y rentabilidad de la obra.' }}
      </p>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Moneda.Label') }}</span>
        <Select
          v-model="drawer.model.value.moneda"
          :options="monedaOptions"
          option-label="label"
          option-value="value"
          @change="!pideCotizacion && (drawer.model.value.cotizacionAplicada = null)"
        />
      </label>

      <label v-if="pideCotizacion" class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Cotizacion') }}</span>
        <MoneyInput
          :model-value="drawer.model.value.cotizacionAplicada ?? '0.0000'"
          :min="0"
          :invalid="Boolean(drawer.fieldErrors.value.cotizacionAplicada)"
          @update:model-value="(value) => (drawer.model.value.cotizacionAplicada = value)"
        />
        <FieldError
          id="mov-cotizacion-error"
          :message="drawer.fieldErrors.value.cotizacionAplicada"
        />
      </label>
    </div>
  </CrudDrawer>
</template>
