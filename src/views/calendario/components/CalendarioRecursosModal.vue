<script setup lang="ts">
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { ref } from 'vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import {
  useCalendarioStore,
  type TipoRecurso,
} from '@/stores/useCalendarioStore'

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const tipoRecursoOptions = [
  { label: 'Empleado', value: 'Empleado' },
  { label: 'Vehículo', value: 'Vehiculo' },
  { label: 'Herramienta', value: 'Herramienta' },
  { label: 'Proyecto', value: 'Proyecto' },
]

const { notify } = useApiError()
const store = useCalendarioStore()

const formRecursoNombre = ref('')
const formRecursoTipo = ref<TipoRecurso>('Empleado')
const formRecursoGrupoId = ref<string | null>(null)
const editandoRecursoId = ref<string | null>(null)

async function guardarRecurso() {
  if (!formRecursoNombre.value.trim()) return

  try {
    if (editandoRecursoId.value) {
      const existente = store.recursos.find((r) => r.id === editandoRecursoId.value)
      if (existente) {
        await store.actualizarRecurso(existente.id, {
          nombre: formRecursoNombre.value.trim(),
          tipo: formRecursoTipo.value,
          grupoId: formRecursoGrupoId.value,
          activo: existente.activo,
          rowVersion: existente.rowVersion,
        })
      }
    } else {
      await store.crearRecurso({
        nombre: formRecursoNombre.value.trim(),
        tipo: formRecursoTipo.value,
        grupoId: formRecursoGrupoId.value,
      })
    }
    formRecursoNombre.value = ''
    editandoRecursoId.value = null
  } catch (err: unknown) {
    notify(err)
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    header="Gestión de Recursos"
    class="w-full max-w-xl"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="flex flex-col gap-4">
      <!-- Form new resource -->
      <form class="p-3 border border-border rounded-lg bg-muted/20 flex flex-col gap-3" @submit.prevent="guardarRecurso">
        <div class="font-semibold text-xs">
          {{ editandoRecursoId ? 'Editar Recurso' : 'Nuevo Recurso' }}
        </div>
        <div class="grid grid-cols-3 gap-2">
          <div class="col-span-2">
            <InputText
              v-model="formRecursoNombre"
              required
              placeholder="Nombre (ej. Camioneta 01, Cuadrilla A)"
              class="w-full"
            />
          </div>
          <div>
            <Select
              v-model="formRecursoTipo"
              :options="tipoRecursoOptions"
              option-label="label"
              option-value="value"
              class="w-full"
            />
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <Button
            v-if="editandoRecursoId"
            type="button"
            variant="ghost"
            size="sm"
            @click="editandoRecursoId = null; formRecursoNombre = ''"
          >
            {{ $t('General.Cancel') }}
          </Button>
          <Button type="submit" size="sm">
            {{ editandoRecursoId ? 'Actualizar' : 'Guardar Recurso' }}
          </Button>
        </div>
      </form>

      <!-- List of active resources -->
      <div class="max-h-60 overflow-y-auto border border-border rounded-lg divide-y divide-border">
        <div
          v-for="rec in store.recursos"
          :key="rec.id"
          class="p-2.5 flex items-center justify-between hover:bg-muted/10 text-xs"
        >
          <div>
            <span class="font-medium">{{ rec.nombre }}</span>
            <span class="text-muted-foreground text-[10px] ml-2">({{ rec.tipo }})</span>
          </div>
          <div class="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              @click="
                editandoRecursoId = rec.id;
                formRecursoNombre = rec.nombre;
                formRecursoTipo = rec.tipo;
                formRecursoGrupoId = rec.grupoId;
              "
            >
              <AppIcon name="pencil" :size="14" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              @click="store.eliminarRecurso(rec.id, rec.rowVersion)"
            >
              <AppIcon name="trash-2" :size="14" />
            </Button>
          </div>
        </div>
      </div>
    </div>
    <template #footer>
      <Button
        type="button"
        variant="outline"
        size="sm"
        @click="emit('update:visible', false)"
      >
        {{ $t('General.Close') }}
      </Button>
    </template>
  </Dialog>
</template>
