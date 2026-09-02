<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import PageHeader from '@/components/domain/PageHeader.vue'
import ListState from '@/components/domain/ListState.vue'
import { useApiError, type ApiError } from '@/composables/useApiError'
import {
  useAuthStore,
  type RolDto,
  type PermisoDto,
  type CrearRolInput,
  type ActualizarRolInput,
} from '@/stores/useAuthStore'

const { notify } = useApiError()
const authStore = useAuthStore()

const roles = ref<RolDto[]>([])
const permisos = ref<PermisoDto[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

// Modal state
const showModal = ref(false)
const isEditing = ref(false)
const selectedRol = ref<RolDto | null>(null)

const form = ref<{
  nombre: string
  descripcion: string
  prioridad: number
  permisos: string[]
}>({
  nombre: '',
  descripcion: '',
  prioridad: 50,
  permisos: [],
})

const permisosPorModulo = computed(() => {
  const map = new Map<string, PermisoDto[]>()
  for (const p of permisos.value) {
    const list = map.get(p.modulo) || []
    list.push(p)
    map.set(p.modulo, list)
  }
  return map
})

async function cargar() {
  loading.value = true
  error.value = null
  try {
    const [rList, pList] = await Promise.all([authStore.listRoles(), authStore.listPermisos()])
    roles.value = rList
    permisos.value = pList
  } catch (e: any) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

function openCreate() {
  isEditing.value = false
  selectedRol.value = null
  form.value = {
    nombre: '',
    descripcion: '',
    prioridad: 50,
    permisos: [],
  }
  showModal.value = true
}

async function openEdit(r: RolDto) {
  isEditing.value = true
  selectedRol.value = r
  loading.value = true
  try {
    const detail = await authStore.getRol(r.id)
    form.value = {
      nombre: detail.rol.nombre,
      descripcion: detail.rol.descripcion || '',
      prioridad: detail.rol.prioridad,
      permisos: detail.permisos.map((p) => p.id),
    }
    showModal.value = true
  } catch (e: any) {
    alert(e?.message || 'Error al cargar permisos del rol')
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  if (!form.value.nombre.trim()) return
  loading.value = true
  try {
    if (isEditing.value && selectedRol.value) {
      const input: ActualizarRolInput = {
        nombre: form.value.nombre.trim(),
        descripcion: form.value.descripcion ? form.value.descripcion.trim() : null,
        prioridad: form.value.prioridad,
        permisos: form.value.permisos,
        rowVersion: selectedRol.value.rowVersion,
      }
      await authStore.updateRol(selectedRol.value.id, input)
    } else {
      const input: CrearRolInput = {
        nombre: form.value.nombre.trim(),
        descripcion: form.value.descripcion ? form.value.descripcion.trim() : null,
        prioridad: form.value.prioridad,
        permisos: form.value.permisos,
      }
      await authStore.createRol(input)
    }
    showModal.value = false
    await cargar()
  } catch (e: any) {
    alert(e?.message || 'Error al guardar rol')
  } finally {
    loading.value = false
  }
}

async function handleDelete(r: RolDto) {
  if (r.esSistema) {
    alert('Los roles de sistema no pueden ser eliminados.')
    return
  }
  if (!confirm(`¿Estás seguro de eliminar el rol ${r.nombre}?`)) return
  loading.value = true
  try {
    await authStore.deleteRol(r.id, r.rowVersion)
    await cargar()
  } catch (e: any) {
    alert(e?.message || 'Error al eliminar rol')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  cargar()
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      title="Roles y Permisos"
      subtitle="Definición de perfiles de acceso y matriz de permisos por módulo"
    >
      <template #actions>
        <Button @click="openCreate" class="gap-2">
          <AppIcon name="ShieldPlus" class="w-4 h-4" />
          <span>Nuevo Rol</span>
        </Button>
      </template>
    </PageHeader>

    <div class="bg-card border border-border rounded-xl overflow-hidden shadow-sm">
      <DataTable
        :value="roles"
        :loading="loading"
        responsiveLayout="scroll"
        class="text-sm"
      >
        <template #empty>
          <ListState
            :loading="loading"
            :first-load="firstLoad"
            :error="error"
            :is-empty="roles.length === 0"
            :is-filtered="false"
            empty-key="General.Empty"
            @retry="cargar"
          />
        </template>

        <Column field="nombre" header="Nombre del Rol" sortable>
          <template #body="{ data }">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-primary/10 text-primary flex items-center justify-center font-bold text-xs">
                <AppIcon name="Shield" class="w-4 h-4" />
              </div>
              <div>
                <span class="font-medium text-foreground block">{{ data.nombre }}</span>
                <span v-if="data.descripcion" class="text-xs text-muted-foreground">{{ data.descripcion }}</span>
              </div>
            </div>
          </template>
        </Column>

        <Column field="prioridad" header="Prioridad" sortable>
          <template #body="{ data }">
            <span class="font-mono text-xs">{{ data.prioridad }}</span>
          </template>
        </Column>

        <Column field="esSistema" header="Tipo">
          <template #body="{ data }">
            <span
              :class="[
                'inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium',
                data.esSistema ? 'bg-warning/10 text-warning border border-warning/20' : 'bg-muted text-muted-foreground'
              ]"
            >
              {{ data.esSistema ? 'Sistema' : 'Personalizado' }}
            </span>
          </template>
        </Column>

        <Column header="Acciones" style="width: 120px">
          <template #body="{ data }">
            <div class="flex items-center gap-1">
              <button
                @click="openEdit(data)"
                class="p-1.5 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                title="Editar permisos"
              >
                <AppIcon name="Pencil" class="w-4 h-4" />
              </button>
              <button
                v-if="!data.esSistema"
                @click="handleDelete(data)"
                class="p-1.5 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
                title="Eliminar"
              >
                <AppIcon name="Trash2" class="w-4 h-4" />
              </button>
            </div>
          </template>
        </Column>
      </DataTable>
    </div>

    <!-- Modal Formulario de Rol y Permisos -->
    <div
      v-if="showModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
    >
      <div class="bg-card border border-border rounded-2xl w-full max-w-2xl max-h-[90vh] flex flex-col p-6 shadow-2xl space-y-4">
        <div class="flex items-center justify-between border-b border-border pb-3">
          <h2 class="text-lg font-bold text-foreground">
            {{ isEditing ? `Editar Rol: ${selectedRol?.nombre}` : 'Nuevo Rol' }}
          </h2>
          <button @click="showModal = false" class="text-muted-foreground hover:text-foreground">
            <AppIcon name="X" class="w-5 h-5" />
          </button>
        </div>

        <form @submit.prevent="handleSave" class="space-y-4 flex-1 overflow-y-auto pr-1">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-xs font-medium text-muted-foreground mb-1">Nombre del Rol</label>
              <input
                v-model="form.nombre"
                required
                :disabled="selectedRol?.esSistema"
                class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary disabled:opacity-50"
              />
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground mb-1">Prioridad (0 - 100)</label>
              <input
                v-model.number="form.prioridad"
                type="number"
                min="0"
                max="100"
                required
                class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          </div>

          <div>
            <label class="block text-xs font-medium text-muted-foreground mb-1">Descripción</label>
            <input
              v-model="form.descripcion"
              class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block text-xs font-semibold text-foreground mb-2">Matriz de Permisos</label>
            <div class="space-y-3 max-h-72 overflow-y-auto border border-border rounded-xl p-3 bg-background">
              <div
                v-for="[modulo, lista] in permisosPorModulo"
                :key="modulo"
                class="border border-border/50 rounded-lg p-3 bg-muted/20"
              >
                <div class="text-xs font-bold uppercase tracking-wider text-primary mb-2">
                  {{ modulo }}
                </div>
                <div class="grid grid-cols-2 gap-2">
                  <label
                    v-for="p in lista"
                    :key="p.id"
                    class="flex items-center gap-2 text-xs text-foreground cursor-pointer hover:bg-muted/50 p-1 rounded"
                  >
                    <input
                      type="checkbox"
                      :value="p.id"
                      v-model="form.permisos"
                      class="rounded border-border text-primary focus:ring-primary"
                    />
                    <span>{{ p.clave }}</span>
                  </label>
                </div>
              </div>
            </div>
          </div>

          <div class="flex justify-end gap-3 pt-4 border-t border-border">
            <Button variant="outline" type="button" @click="showModal = false">Cancelar</Button>
            <Button type="submit">Guardar Rol</Button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
