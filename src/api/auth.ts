import { callCommand } from './client'
import type { RowVersion, Uuid } from './types'

export interface AuthModeDto {
  isSqliteMode: boolean
  requiresLogin: boolean
}

export interface UsuarioDto {
  id: Uuid
  email: string
  nombreCompleto: string
  activo: boolean
  requiere2fa: boolean
  ultimoLogin: string | null
  rowVersion: RowVersion
}

export interface RolDto {
  id: Uuid
  nombre: string
  descripcion: string | null
  esSistema: boolean
  prioridad: number
  rowVersion: RowVersion
}

export interface PermisoDto {
  id: Uuid
  modulo: string
  accion: string
  recurso: string | null
  clave: string
}

export interface UsuarioConDetalleDto {
  usuario: UsuarioDto
  roles: RolDto[]
  permisos: string[]
}

export interface RolConPermisosDto {
  rol: RolDto
  permisos: PermisoDto[]
}

export interface LoginRequest {
  email: string
  password: string
  totpCode?: string | null
}

export interface LoginResponse {
  token: string
  usuario: UsuarioDto
  roles: string[]
  permisos: string[]
  requiere2fa: boolean
}

export interface CrearUsuarioInput {
  email: string
  nombreCompleto: string
  password?: string | null
  roles: Uuid[]
  requiere2fa: boolean
}

export interface ActualizarUsuarioInput {
  nombreCompleto: string
  password?: string | null
  activo: boolean
  requiere2fa: boolean
  roles: Uuid[]
  rowVersion: RowVersion
}

export interface CrearRolInput {
  nombre: string
  descripcion?: string | null
  prioridad: number
  permisos: Uuid[]
}

export interface ActualizarRolInput {
  nombre: string
  descripcion?: string | null
  prioridad: number
  permisos: Uuid[]
  rowVersion: RowVersion
}

export interface Configurar2faResponse {
  secret: string
  otpauthUrl: string
}

export interface Verificar2faInput {
  secret: string
  code: string
}

export function getAuthMode(): Promise<AuthModeDto> {
  return callCommand('auth_get_mode')
}

export function getCurrentUser(token?: string | null): Promise<UsuarioConDetalleDto | null> {
  return callCommand('auth_current_user', { token })
}

export function login(req: LoginRequest): Promise<LoginResponse> {
  return callCommand('auth_login', { req })
}

export function logout(token: string): Promise<void> {
  return callCommand('auth_logout', { token })
}

export function configurar2fa(usuarioId: Uuid): Promise<Configurar2faResponse> {
  return callCommand('auth_configurar_2fa', { usuarioId })
}

export function activar2fa(usuarioId: Uuid, input: Verificar2faInput): Promise<void> {
  return callCommand('auth_activar_2fa', { usuarioId, input })
}

export function desactivar2fa(usuarioId: Uuid): Promise<void> {
  return callCommand('auth_desactivar_2fa', { usuarioId })
}

export function listUsuarios(): Promise<UsuarioDto[]> {
  return callCommand('usuarios_list')
}

export function getUsuario(id: Uuid): Promise<UsuarioConDetalleDto> {
  return callCommand('usuarios_get', { id })
}

export function createUsuario(input: CrearUsuarioInput): Promise<UsuarioDto> {
  return callCommand('usuarios_create', { input })
}

export function updateUsuario(id: Uuid, input: ActualizarUsuarioInput): Promise<UsuarioDto> {
  return callCommand('usuarios_update', { id, input })
}

export function deleteUsuario(id: Uuid, version: string): Promise<void> {
  return callCommand('usuarios_delete', { id, version })
}

export function listRoles(): Promise<RolDto[]> {
  return callCommand('roles_list')
}

export function getRol(id: Uuid): Promise<RolConPermisosDto> {
  return callCommand('roles_get', { id })
}

export function createRol(input: CrearRolInput): Promise<RolDto> {
  return callCommand('roles_create', { input })
}

export function updateRol(id: Uuid, input: ActualizarRolInput): Promise<RolDto> {
  return callCommand('roles_update', { id, input })
}

export function deleteRol(id: Uuid, version: string): Promise<void> {
  return callCommand('roles_delete', { id, version })
}

export function listPermisos(): Promise<PermisoDto[]> {
  return callCommand('permisos_list')
}
