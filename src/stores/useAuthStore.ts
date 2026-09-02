import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  getAuthMode,
  getCurrentUser,
  login as apiLogin,
  logout as apiLogout,
  listUsuarios as apiListUsuarios,
  getUsuario as apiGetUsuario,
  createUsuario as apiCreateUsuario,
  updateUsuario as apiUpdateUsuario,
  deleteUsuario as apiDeleteUsuario,
  listRoles as apiListRoles,
  getRol as apiGetRol,
  createRol as apiCreateRol,
  updateRol as apiUpdateRol,
  deleteRol as apiDeleteRol,
  listPermisos as apiListPermisos,
  configurar2fa as apiConfigurar2fa,
  activar2fa as apiActivar2fa,
  desactivar2fa as apiDesactivar2fa,
  type UsuarioDto,
  type RolDto,
  type PermisoDto,
  type UsuarioConDetalleDto,
  type RolConPermisosDto,
  type LoginRequest,
  type CrearUsuarioInput,
  type ActualizarUsuarioInput,
  type CrearRolInput,
  type ActualizarRolInput,
  type Configurar2faResponse,
  type Verificar2faInput,
} from '@/api/auth'
import type { Uuid } from '@/api/types'

export type {
  UsuarioDto,
  RolDto,
  PermisoDto,
  UsuarioConDetalleDto,
  RolConPermisosDto,
  LoginRequest,
  LoginResponse,
  CrearUsuarioInput,
  ActualizarUsuarioInput,
  CrearRolInput,
  ActualizarRolInput,
  Configurar2faResponse,
  Verificar2faInput,
} from '@/api/auth'

const TOKEN_KEY = 'certaro_auth_token'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<UsuarioDto | null>(null)
  const roles = ref<RolDto[]>([])
  const permisos = ref<string[]>([])
  const token = ref<string | null>(localStorage.getItem(TOKEN_KEY))
  const isSqliteMode = ref(true)
  const requiresLogin = ref(false)
  const initialized = ref(false)
  const loading = ref(false)

  const isAuthenticated = computed(() => {
    if (isSqliteMode.value) return true
    return !!user.value
  })

  function hasPermission(permiso: string): boolean {
    if (isSqliteMode.value) return true
    if (roles.value.some((r) => r.nombre === 'Administrador' || r.prioridad >= 100)) return true
    return permisos.value.includes(permiso)
  }

  function hasRole(rolNombre: string): boolean {
    if (isSqliteMode.value) return true
    return roles.value.some((r) => r.nombre === rolNombre)
  }

  async function init() {
    loading.value = true
    try {
      const mode = await getAuthMode()
      isSqliteMode.value = mode.isSqliteMode
      requiresLogin.value = mode.requiresLogin

      if (isSqliteMode.value) {
        // Bypass login: fetch Super Admin user profile directly
        const res = await getCurrentUser()
        if (res) {
          user.value = res.usuario
          roles.value = res.roles
          permisos.value = res.permisos
        }
      } else if (token.value) {
        // Validate existing token
        const res = await getCurrentUser(token.value)
        if (res) {
          user.value = res.usuario
          roles.value = res.roles
          permisos.value = res.permisos
        } else {
          // Token expired or invalid
          token.value = null
          localStorage.removeItem(TOKEN_KEY)
          user.value = null
          roles.value = []
          permisos.value = []
        }
      }
    } catch (e) {
      console.error('Error initializing auth store:', e)
    } finally {
      loading.value = false
      initialized.value = true
    }
  }

  async function login(req: LoginRequest) {
    loading.value = true
    try {
      const res = await apiLogin(req)
      token.value = res.token
      localStorage.setItem(TOKEN_KEY, res.token)
      user.value = res.usuario
      permisos.value = res.permisos
      // Fetch full details to populate roles
      const detail = await getCurrentUser(res.token)
      if (detail) {
        roles.value = detail.roles
      }
      return res
    } finally {
      loading.value = false
    }
  }

  async function logout() {
    if (token.value) {
      try {
        await apiLogout(token.value)
      } catch (e) {
        console.warn('Error during logout:', e)
      }
    }
    token.value = null
    localStorage.removeItem(TOKEN_KEY)
    user.value = null
    roles.value = []
    permisos.value = []
  }

  // Users management
  function listUsuarios(): Promise<UsuarioDto[]> {
    return apiListUsuarios()
  }

  function getUsuario(id: Uuid): Promise<UsuarioConDetalleDto> {
    return apiGetUsuario(id)
  }

  function createUsuario(input: CrearUsuarioInput): Promise<UsuarioDto> {
    return apiCreateUsuario(input)
  }

  function updateUsuario(id: Uuid, input: ActualizarUsuarioInput): Promise<UsuarioDto> {
    return apiUpdateUsuario(id, input)
  }

  function deleteUsuario(id: Uuid, version: string): Promise<void> {
    return apiDeleteUsuario(id, version)
  }

  // Roles management
  function listRoles(): Promise<RolDto[]> {
    return apiListRoles()
  }

  function getRol(id: Uuid): Promise<RolConPermisosDto> {
    return apiGetRol(id)
  }

  function createRol(input: CrearRolInput): Promise<RolDto> {
    return apiCreateRol(input)
  }

  function updateRol(id: Uuid, input: ActualizarRolInput): Promise<RolDto> {
    return apiUpdateRol(id, input)
  }

  function deleteRol(id: Uuid, version: string): Promise<void> {
    return apiDeleteRol(id, version)
  }

  // Permissions
  function listPermisos(): Promise<PermisoDto[]> {
    return apiListPermisos()
  }

  // 2FA
  function configurar2fa(usuarioId: Uuid): Promise<Configurar2faResponse> {
    return apiConfigurar2fa(usuarioId)
  }

  function activar2fa(usuarioId: Uuid, input: Verificar2faInput): Promise<void> {
    return apiActivar2fa(usuarioId, input)
  }

  function desactivar2fa(usuarioId: Uuid): Promise<void> {
    return apiDesactivar2fa(usuarioId)
  }

  return {
    user,
    roles,
    permisos,
    token,
    isSqliteMode,
    requiresLogin,
    initialized,
    loading,
    isAuthenticated,
    hasPermission,
    hasRole,
    init,
    login,
    logout,
    listUsuarios,
    getUsuario,
    createUsuario,
    updateUsuario,
    deleteUsuario,
    listRoles,
    getRol,
    createRol,
    updateRol,
    deleteRol,
    listPermisos,
    configurar2fa,
    activar2fa,
    desactivar2fa,
  }
})
