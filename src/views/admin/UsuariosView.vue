<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import PageHeader from '@/components/domain/PageHeader.vue'
import ListState from '@/components/domain/ListState.vue'
import { useApiError, type ApiError } from '@/composables/useApiError'
import {
  useAuthStore,
  type UsuarioDto,
  type RolDto,
  type CrearUsuarioInput,
  type ActualizarUsuarioInput,
} from '@/stores/useAuthStore'

const { notify } = useApiError()
const authStore = useAuthStore()

const usuarios = ref<UsuarioDto[]>([])
const roles = ref<RolDto[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

// Modal state
const showModal = ref(false)
const isEditing = ref(false)
const selectedUser = ref<UsuarioDto | null>(null)

const form = ref<{
  email: string
  nombreCompleto: string
  password: string
  roles: string[]
  activo: boolean
  requiere2fa: boolean
}>({
  email: '',
  nombreCompleto: '',
  password: '',
  roles: [],
  activo: true,
  requiere2fa: false,
})

async function cargar() {
  loading.value = true
  error.value = null
  try {
    const [uList, rList] = await Promise.all([authStore.listUsuarios(), authStore.listRoles()])
    usuarios.value = uList
    roles.value = rList
  } catch (e: any) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

function openCreate() {
  isEditing.value = false
  selectedUser.value = null
  form.value = {
    email: '',
    nombreCompleto: '',
    password: '',
    roles: [],
    activo: true,
    requiere2fa: false,
  }
  showModal.value = true
}

function openEdit(u: UsuarioDto) {
  isEditing.value = true
  selectedUser.value = u
  form.value = {
    email: u.email,
    nombreCompleto: u.nombreCompleto,
    password: '',
    roles: [],
    activo: u.activo,
    requiere2fa: u.requiere2fa,
  }
  showModal.value = true
}

async function handleSave() {
  if (!form.value.nombreCompleto.trim()) return
  loading.value = true
  try {
    if (isEditing.value && selectedUser.value) {
      const input: ActualizarUsuarioInput = {
        nombreCompleto: form.value.nombreCompleto.trim(),
        password: form.value.password ? form.value.password : null,
        activo: form.value.activo,
        requiere2fa: form.value.requiere2fa,
        roles: form.value.roles,
        rowVersion: selectedUser.value.rowVersion,
      }
      await authStore.updateUsuario(selectedUser.value.id, input)
    } else {
      const input: CrearUsuarioInput = {
        email: form.value.email.trim(),
        nombreCompleto: form.value.nombreCompleto.trim(),
        password: form.value.password ? form.value.password : null,
        roles: form.value.roles,
        requiere2fa: form.value.requiere2fa,
      }
      await authStore.createUsuario(input)
    }
    showModal.value = false
    await cargar()
  } catch (e: any) {
    alert(e?.message || 'Error al guardar usuario')
  } finally {
    loading.value = false
  }
}

async function handleDelete(u: UsuarioDto) {
  if (!confirm(`¿Estás seguro de eliminar el usuario ${u.nombreCompleto}?`)) return
  loading.value = true
  try {
    await authStore.deleteUsuario(u.id, u.rowVersion)
    await cargar()
  } catch (e: any) {
    alert(e?.message || 'Error al eliminar usuario')
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
      title="Gestión de Usuarios"
      subtitle="Administración de cuentas, acceso y roles en Certaro Enterprise"
    >
      <template #actions>
        <Button @click="openCreate" class="gap-2">
          <AppIcon name="UserPlus" class="w-4 h-4" />
          <span>Nuevo Usuario</span>
        </Button>
      </template>
    </PageHeader>

    <div class="bg-card border border-border rounded-xl overflow-hidden shadow-sm">
      <DataTable
        :value="usuarios"
        :loading="loading"
        responsiveLayout="scroll"
        class="text-sm"
      >
        <template #empty>
          <ListState
            :loading="loading"
            :first-load="firstLoad"
            :error="error"
            :is-empty="usuarios.length === 0"
            :is-filtered="false"
            empty-key="General.Empty"
            @retry="cargar"
          />
        </template>

        <Column field="nombreCompleto" header="Nombre Completo" sortable>
          <template #body="{ data }">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-full bg-primary/10 text-primary flex items-center justify-center font-bold text-xs uppercase">
                {{ data.nombreCompleto.slice(0, 2) }}
              </div>
              <span class="font-medium text-foreground">{{ data.nombreCompleto }}</span>
            </div>
          </template>
        </Column>

        <Column field="email" header="Correo Electrónico" sortable>
          <template #body="{ data }">
            <span class="font-mono text-xs text-muted-foreground">{{ data.email }}</span>
          </template>
        </Column>

        <Column field="activo" header="Estado">
          <template #body="{ data }">
            <span
              :class="[
                'inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium',
                data.activo ? 'bg-success/10 text-success border border-success/20' : 'bg-destructive/10 text-destructive border border-destructive/20'
              ]"
            >
              <span class="w-1.5 h-1.5 rounded-full" :class="data.activo ? 'bg-success' : 'bg-destructive'" />
              {{ data.activo ? 'Activo' : 'Inactivo' }}
            </span>
          </template>
        </Column>

        <Column field="requiere2fa" header="2FA">
          <template #body="{ data }">
            <span
              :class="[
                'inline-flex items-center px-2 py-0.5 rounded text-xs',
                data.requiere2fa ? 'bg-primary/10 text-primary' : 'text-muted-foreground'
              ]"
            >
              {{ data.requiere2fa ? 'Habilitado' : 'No' }}
            </span>
          </template>
        </Column>

        <Column header="Acciones" style="width: 120px">
          <template #body="{ data }">
            <div class="flex items-center gap-1">
              <button
                @click="openEdit(data)"
                class="p-1.5 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                title="Editar"
              >
                <AppIcon name="Pencil" class="w-4 h-4" />
              </button>
              <button
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

    <!-- Modal Formulario -->
    <div
      v-if="showModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
    >
      <div class="bg-card border border-border rounded-2xl w-full max-w-lg p-6 shadow-2xl space-y-5">
        <div class="flex items-center justify-between border-b border-border pb-4">
          <h2 class="text-lg font-bold text-foreground">
            {{ isEditing ? 'Editar Usuario' : 'Nuevo Usuario' }}
          </h2>
          <button @click="showModal = false" class="text-muted-foreground hover:text-foreground">
            <AppIcon name="X" class="w-5 h-5" />
          </button>
        </div>

        <form @submit.prevent="handleSave" class="space-y-4">
          <div>
            <label class="block text-xs font-medium text-muted-foreground mb-1">Nombre Completo</label>
            <input
              v-model="form.nombreCompleto"
              required
              class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          <div v-if="!isEditing">
            <label class="block text-xs font-medium text-muted-foreground mb-1">Correo Electrónico</label>
            <input
              v-model="form.email"
              type="email"
              required
              class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block text-xs font-medium text-muted-foreground mb-1">
              {{ isEditing ? 'Nueva Contraseña (dejar vacío para no cambiar)' : 'Contraseña' }}
            </label>
            <input
              v-model="form.password"
              type="password"
              :required="!isEditing"
              placeholder="••••••••"
              class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block text-xs font-medium text-muted-foreground mb-2">Asignar Roles</label>
            <div class="space-y-1.5 max-h-36 overflow-y-auto border border-border rounded-lg p-2 bg-background">
              <label
                v-for="r in roles"
                :key="r.id"
                class="flex items-center gap-2 text-sm text-foreground hover:bg-muted/50 p-1.5 rounded cursor-pointer"
              >
                <input
                  type="checkbox"
                  :value="r.id"
                  v-model="form.roles"
                  class="rounded border-border text-primary focus:ring-primary"
                />
                <span>{{ r.nombre }}</span>
                <span v-if="r.descripcion" class="text-xs text-muted-foreground">({{ r.descripcion }})</span>
              </label>
            </div>
          </div>

          <div class="flex items-center gap-6 pt-2">
            <label class="flex items-center gap-2 text-sm text-foreground cursor-pointer">
              <input
                type="checkbox"
                v-model="form.activo"
                class="rounded border-border text-primary focus:ring-primary"
              />
              <span>Usuario Activo</span>
            </label>
            <label class="flex items-center gap-2 text-sm text-foreground cursor-pointer">
              <input
                type="checkbox"
                v-model="form.requiere2fa"
                class="rounded border-border text-primary focus:ring-primary"
              />
              <span>Exigir 2FA (TOTP)</span>
            </label>
          </div>

          <div class="flex justify-end gap-3 pt-4 border-t border-border">
            <Button variant="outline" type="button" @click="showModal = false">Cancelar</Button>
            <Button type="submit">Guardar</Button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
