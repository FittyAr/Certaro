//! Tauri commands for Authentication, Authorization, Users, and Roles.

use certaro_application::dtos::auth::{
    ActualizarRolInput, ActualizarUsuarioInput, Configurar2faResponse, CrearRolInput,
    CrearUsuarioInput, LoginRequest, LoginResponse, PermisoDto, RolConPermisosDto, RolDto,
    UsuarioConDetalleDto, UsuarioDto, Verificar2faInput,
};
use certaro_domain::RowVersion;
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthModeDto {
    pub is_sqlite_mode: bool,
    pub requires_login: bool,
}

#[tauri::command]
pub async fn auth_get_mode(state: State<'_, AppState>) -> ApiResult<AuthModeDto> {
    let is_sqlite = state.is_sqlite_mode();
    handle(
        "auth_get_mode",
        Ok(AuthModeDto {
            is_sqlite_mode: is_sqlite,
            requires_login: !is_sqlite,
        }),
    )
}

#[tauri::command]
pub async fn auth_current_user(
    state: State<'_, AppState>,
    token: Option<String>,
) -> ApiResult<Option<UsuarioConDetalleDto>> {
    // In SQLite desktop mode, authentication is completely bypassed
    if state.is_sqlite_mode() {
        let super_admin_id = Uuid::from_u128(0x999);
        let outcome = match state.services() {
            Ok(services) => services.auth.obtener_usuario(super_admin_id).await.map(Some),
            Err(e) => Err(e),
        };
        return handle("auth_current_user", outcome);
    }

    let token_str = match token {
        Some(t) if !t.trim().is_empty() => t,
        _ => return handle("auth_current_user", Ok(None)),
    };

    let outcome = match state.services() {
        Ok(services) => services.auth.validar_sesion(&token_str).await,
        Err(e) => Err(e),
    };
    handle("auth_current_user", outcome)
}

#[tauri::command]
pub async fn auth_login(state: State<'_, AppState>, req: LoginRequest) -> ApiResult<LoginResponse> {
    let outcome = match state.services() {
        Ok(services) => services.auth.login(req, None, None).await,
        Err(e) => Err(e),
    };
    handle("auth_login", outcome)
}

#[tauri::command]
pub async fn auth_logout(state: State<'_, AppState>, token: String) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.auth.logout(&token).await,
        Err(e) => Err(e),
    };
    handle("auth_logout", outcome)
}

#[tauri::command]
pub async fn auth_configurar_2fa(
    state: State<'_, AppState>,
    usuario_id: Uuid,
) -> ApiResult<Configurar2faResponse> {
    let outcome = match state.services() {
        Ok(services) => services.auth.configurar_2fa(usuario_id).await,
        Err(e) => Err(e),
    };
    handle("auth_configurar_2fa", outcome)
}

#[tauri::command]
pub async fn auth_activar_2fa(
    state: State<'_, AppState>,
    usuario_id: Uuid,
    input: Verificar2faInput,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.auth.activar_2fa(usuario_id, input).await,
        Err(e) => Err(e),
    };
    handle("auth_activar_2fa", outcome)
}

#[tauri::command]
pub async fn auth_desactivar_2fa(state: State<'_, AppState>, usuario_id: Uuid) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.auth.desactivar_2fa(usuario_id).await,
        Err(e) => Err(e),
    };
    handle("auth_desactivar_2fa", outcome)
}

#[tauri::command]
pub async fn usuarios_list(state: State<'_, AppState>) -> ApiResult<Vec<UsuarioDto>> {
    let outcome = match state.services() {
        Ok(services) => services.auth.listar_usuarios().await,
        Err(e) => Err(e),
    };
    handle("usuarios_list", outcome)
}

#[tauri::command]
pub async fn usuarios_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<UsuarioConDetalleDto> {
    let outcome = match state.services() {
        Ok(services) => services.auth.obtener_usuario(id).await,
        Err(e) => Err(e),
    };
    handle("usuarios_get", outcome)
}

#[tauri::command]
pub async fn usuarios_create(
    state: State<'_, AppState>,
    input: CrearUsuarioInput,
) -> ApiResult<UsuarioDto> {
    let outcome = match state.services() {
        Ok(services) => services.auth.crear_usuario(input).await,
        Err(e) => Err(e),
    };
    handle("usuarios_create", outcome)
}

#[tauri::command]
pub async fn usuarios_update(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarUsuarioInput,
) -> ApiResult<UsuarioDto> {
    let outcome = match state.services() {
        Ok(services) => services.auth.actualizar_usuario(id, input).await,
        Err(e) => Err(e),
    };
    handle("usuarios_update", outcome)
}

#[tauri::command]
pub async fn usuarios_delete(
    state: State<'_, AppState>,
    id: Uuid,
    version: String,
) -> ApiResult<()> {
    let expected = match RowVersion::parse_hex(&version) {
        Ok(v) => v,
        Err(e) => {
            return handle(
                "usuarios_delete",
                Err(certaro_application::AppError::unexpected(anyhow::anyhow!(e))),
            )
        }
    };
    let outcome = match state.services() {
        Ok(services) => services.auth.eliminar_usuario(id, expected).await,
        Err(e) => Err(e),
    };
    handle("usuarios_delete", outcome)
}

#[tauri::command]
pub async fn roles_list(state: State<'_, AppState>) -> ApiResult<Vec<RolDto>> {
    let outcome = match state.services() {
        Ok(services) => services.auth.listar_roles().await,
        Err(e) => Err(e),
    };
    handle("roles_list", outcome)
}

#[tauri::command]
pub async fn roles_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<RolConPermisosDto> {
    let outcome = match state.services() {
        Ok(services) => services.auth.obtener_rol(id).await,
        Err(e) => Err(e),
    };
    handle("roles_get", outcome)
}

#[tauri::command]
pub async fn roles_create(state: State<'_, AppState>, input: CrearRolInput) -> ApiResult<RolDto> {
    let outcome = match state.services() {
        Ok(services) => services.auth.crear_rol(input).await,
        Err(e) => Err(e),
    };
    handle("roles_create", outcome)
}

#[tauri::command]
pub async fn roles_update(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarRolInput,
) -> ApiResult<RolDto> {
    let outcome = match state.services() {
        Ok(services) => services.auth.actualizar_rol(id, input).await,
        Err(e) => Err(e),
    };
    handle("roles_update", outcome)
}

#[tauri::command]
pub async fn roles_delete(state: State<'_, AppState>, id: Uuid, version: String) -> ApiResult<()> {
    let expected = match RowVersion::parse_hex(&version) {
        Ok(v) => v,
        Err(e) => {
            return handle(
                "roles_delete",
                Err(certaro_application::AppError::unexpected(anyhow::anyhow!(e))),
            )
        }
    };
    let outcome = match state.services() {
        Ok(services) => services.auth.eliminar_rol(id, expected).await,
        Err(e) => Err(e),
    };
    handle("roles_delete", outcome)
}

#[tauri::command]
pub async fn permisos_list(state: State<'_, AppState>) -> ApiResult<Vec<PermisoDto>> {
    let outcome = match state.services() {
        Ok(services) => services.auth.listar_permisos().await,
        Err(e) => Err(e),
    };
    handle("permisos_list", outcome)
}
