<script setup lang="ts">
import Dialog from 'primevue/dialog'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { ref, watch } from 'vue'
import DateInput from '@/components/domain/DateInput.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import { useAsistenciaStore, type TipoJornada } from '@/stores/useAsistenciaStore'

const props = defineProps<{
  visible: boolean
  empleadosOpciones: LookupItem[]
  opcionesProyecto: LookupItem[]
  initialEmpleadoId?: string
  initialProyectoId?: string | null
  initialDesde: string
  initialHasta: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void
  (e: 'saved'): void
}>()

const { notify } = useApiError()
const store = useAsistenciaStore()
const trabajosStore = useTrabajosStore()

const TIPOS: TipoJornada[] = ['Completa', 'Media', 'Falta', 'FaltaJustificada', 'Feriado']
const guardando = ref(false)
const todaLaCuadrilla = ref(false)
const opcionesTrabajo = ref<LookupItem[]>([])

const rango = ref<{
  empleadoId: string
  proyectoId: string | null
  trabajoId: string | null
  desde: string
  hasta: string
  tipoJornada: TipoJornada
  soloDiasHabiles: boolean
}>({
  empleadoId: '',
  proyectoId: null,
  trabajoId: null,
  desde: '',
  hasta: '',
  tipoJornada: 'Completa',
  soloDiasHabiles: true,
})

watch(
  () => props.visible,
  async (isOpen) => {
    if (isOpen) {
      todaLaCuadrilla.value = false
      rango.value = {
        empleadoId: props.initialEmpleadoId ?? props.empleadosOpciones[0]?.id ?? '',
        proyectoId: props.initialProyectoId ?? null,
        trabajoId: null,
        desde: props.initialDesde,
        hasta: props.initialHasta,
        tipoJornada: 'Completa',
        soloDiasHabiles: true,
      }
      if (rango.value.proyectoId) {
        await onProyectoRangoChange(rango.value.proyectoId)
      } else {
        opcionesTrabajo.value = []
      }
    }
  },
  { immediate: true },
)

async function onProyectoRangoChange(pId: string | null): Promise<void> {
  rango.value.trabajoId = null
  if (!pId) {
    opcionesTrabajo.value = []
    return
  }
  try {
    opcionesTrabajo.value = await trabajosStore.lookup(pId)
    if (opcionesTrabajo.value.length === 1 && opcionesTrabajo.value[0]) {
      rango.value.trabajoId = opcionesTrabajo.value[0].id
    }
  } catch {
    opcionesTrabajo.value = []
  }
}

async function guardarRango(): Promise<void> {
  if (guardando.value) return
  if (!todaLaCuadrilla.value && !rango.value.empleadoId) return
  if (todaLaCuadrilla.value && props.empleadosOpciones.length === 0) return

  guardando.value = true
  try {
    if (todaLaCuadrilla.value) {
      await store.cargarRangoCuadrilla(
        props.empleadosOpciones.map((emp) => ({
          empleadoId: emp.id,
          desde: rango.value.desde,
          hasta: rango.value.hasta,
          tipoJornada: rango.value.tipoJornada,
          soloDiasHabiles: rango.value.soloDiasHabiles,
          trabajoId: rango.value.trabajoId,
        })),
      )
    } else {
      await store.cargarRango({
        empleadoId: rango.value.empleadoId,
        desde: rango.value.desde,
        hasta: rango.value.hasta,
        tipoJornada: rango.value.tipoJornada,
        soloDiasHabiles: rango.value.soloDiasHabiles,
        trabajoId: rango.value.trabajoId,
      })
    }
    emit('update:visible', false)
    emit('saved')
  } catch (e) {
    notify(e)
  } finally {
    guardando.value = false
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="$t('Asistencia.CargaMasiva')"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="flex w-80 flex-col gap-3">
      <!-- Opción: Toda la cuadrilla o individual -->
      <label class="flex items-center gap-2 cursor-pointer select-none rounded border border-border/70 bg-muted/20 p-2 text-sm">
        <ToggleSwitch v-model="todaLaCuadrilla" />
        <span class="font-medium text-foreground/90">
          {{ $t('Asistencia.TodaCuadrilla') }}
        </span>
      </label>

      <label v-if="!todaLaCuadrilla" class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Empleados.Nombre') }}</span>
        <Select
          v-model="rango.empleadoId"
          :options="empleadosOpciones"
          option-label="label"
          option-value="id"
          filter
        />
      </label>
      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('General.From') }}</span>
          <DateInput v-model="rango.desde" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('General.To') }}</span>
          <DateInput v-model="rango.hasta" />
        </label>
      </div>
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Asistencia.TipoJornada') }}</span>
        <Select
          v-model="rango.tipoJornada"
          :options="TIPOS"
          :option-label="(o: TipoJornada) => $t(`TipoJornada.${o}`)"
        />
      </label>

      <!-- Imputación opcional a Proyecto / Trabajo -->
      <div class="space-y-2 rounded-md border border-border/70 bg-muted/20 p-2.5">
        <span class="text-xs font-semibold text-muted-foreground">
          Imputación a Obra (Opcional)
        </span>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Menu.Proyectos') }}</span>
          <Select
            v-model="rango.proyectoId"
            :options="opcionesProyecto"
            option-label="label"
            option-value="id"
            filter
            show-clear
            :placeholder="$t('General.None')"
            @change="onProyectoRangoChange(rango.proyectoId)"
          />
        </label>
        <label v-if="opcionesTrabajo.length > 0" class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Menu.Trabajos') }}</span>
          <Select
            v-model="rango.trabajoId"
            :options="opcionesTrabajo"
            option-label="label"
            option-value="id"
            filter
            show-clear
            :placeholder="$t('General.None')"
          />
        </label>
      </div>

      <label class="flex items-center gap-2 cursor-pointer select-none">
        <ToggleSwitch v-model="rango.soloDiasHabiles" />
        <span class="text-sm font-medium text-foreground/90">{{ $t('Asistencia.SoloDiasHabiles') }}</span>
      </label>
    </div>

    <template #footer>
      <Button variant="outline" :disabled="guardando" @click="emit('update:visible', false)">
        {{ $t('General.Cancel') }}
      </Button>
      <Button :disabled="guardando" @click="guardarRango()">{{ $t('General.Save') }}</Button>
    </template>
  </Dialog>
</template>
