//! DTOs for authentication, authorization and user management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub usuario: UsuarioDto,
    pub roles: Vec<String>,
    pub permisos: Vec<String>,
    pub requiere_2fa: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioDto {
    pub id: Uuid,
    pub email: String,
    pub nombre_completo: String,
    pub activo: bool,
    pub requiere_2fa: bool,
    pub ultimo_login: Option<DateTime<Utc>>,
    pub row_version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioConDetalleDto {
    pub usuario: UsuarioDto,
    pub roles: Vec<RolDto>,
    pub permisos: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearUsuarioInput {
    pub email: String,
    pub nombre_completo: String,
    pub password: Option<String>,
    pub roles: Vec<Uuid>,
    #[serde(default)]
    pub requiere_2fa: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarUsuarioInput {
    pub nombre_completo: String,
    pub password: Option<String>,
    pub activo: bool,
    pub requiere_2fa: bool,
    pub roles: Vec<Uuid>,
    pub row_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolDto {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub es_sistema: bool,
    pub prioridad: i32,
    pub row_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermisoDto {
    pub id: Uuid,
    pub modulo: String,
    pub accion: String,
    pub recurso: Option<String>,
    pub clave: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RolConPermisosDto {
    pub rol: RolDto,
    pub permisos: Vec<PermisoDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearRolInput {
    pub nombre: String,
    pub descripcion: Option<String>,
    #[serde(default)]
    pub prioridad: i32,
    #[serde(default)]
    pub permisos: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarRolInput {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub prioridad: i32,
    pub permisos: Vec<Uuid>,
    pub row_version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SesionDto {
    pub id: Uuid,
    pub expira_en: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configurar2faResponse {
    pub secret: String,
    pub otpauth_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Verificar2faInput {
    pub secret: String,
    pub code: String,
}
