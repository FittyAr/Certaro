//! Domain entities for authentication, authorization and RBAC.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Audit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthProvider {
    Microsoft,
    Google,
    GitHub,
    Ldap,
}

impl AuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Microsoft => "microsoft",
            Self::Google => "google",
            Self::GitHub => "github",
            Self::Ldap => "ldap",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "microsoft" => Some(Self::Microsoft),
            "google" => Some(Self::Google),
            "github" => Some(Self::GitHub),
            "ldap" => Some(Self::Ldap),
            _ => None,
        }
    }
}

/// A system user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usuario {
    pub id: Uuid,
    pub email: String,
    pub nombre_completo: String,
    /// Nullable when user authenticates purely via OAuth or LDAP.
    pub password_hash: Option<String>,
    pub activo: bool,
    pub requiere_2fa: bool,
    pub totp_secret: Option<String>,
    pub ultimo_login: Option<DateTime<Utc>>,
    pub intentos_fallidos: u32,
    pub bloqueado_hasta: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Usuario {
    pub fn new(
        id: Uuid,
        email: String,
        nombre_completo: String,
        password_hash: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            nombre_completo,
            password_hash,
            activo: true,
            requiere_2fa: false,
            totp_secret: None,
            ultimo_login: None,
            intentos_fallidos: 0,
            bloqueado_hasta: None,
            audit: Audit::new(now),
        }
    }

    pub fn esta_bloqueado(&self, now: DateTime<Utc>) -> bool {
        if let Some(hasta) = self.bloqueado_hasta {
            now < hasta
        } else {
            false
        }
    }

    pub fn registrar_fallo(&mut self, max_intentos: u32, minutos_bloqueo: i64, now: DateTime<Utc>) {
        self.intentos_fallidos += 1;
        if self.intentos_fallidos >= max_intentos {
            self.bloqueado_hasta = Some(now + chrono::Duration::minutes(minutos_bloqueo));
        }
        self.audit.touch(now);
    }

    pub fn reset_intentos(&mut self, now: DateTime<Utc>) {
        self.intentos_fallidos = 0;
        self.bloqueado_hasta = None;
        self.ultimo_login = Some(now);
        self.audit.touch(now);
    }
}

/// A role assigned to users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rol {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub es_sistema: bool,
    pub prioridad: i32,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Rol {
    pub fn new(id: Uuid, nombre: String, descripcion: Option<String>, now: DateTime<Utc>) -> Self {
        Self {
            id,
            nombre,
            descripcion,
            es_sistema: false,
            prioridad: 0,
            audit: Audit::new(now),
        }
    }

    pub fn sistema(id: Uuid, nombre: &str, descripcion: &str, prioridad: i32, now: DateTime<Utc>) -> Self {
        Self {
            id,
            nombre: nombre.to_owned(),
            descripcion: Some(descripcion.to_owned()),
            es_sistema: true,
            prioridad,
            audit: Audit::new(now),
        }
    }
}

/// A granular action permission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permiso {
    pub id: Uuid,
    pub modulo: String,
    pub accion: String,
    pub recurso: Option<String>,
    pub clave: String,
}

impl Permiso {
    pub fn new(id: Uuid, modulo: &str, accion: &str, recurso: Option<&str>) -> Self {
        let clave = match recurso {
            Some(r) => format!("{modulo}:{accion}:{r}"),
            None => format!("{modulo}:{accion}"),
        };
        Self {
            id,
            modulo: modulo.to_owned(),
            accion: accion.to_owned(),
            recurso: recurso.map(ToOwned::to_owned),
            clave,
        }
    }
}

/// Junction table between user and role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioRol {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub rol_id: Uuid,
    #[serde(flatten)]
    pub audit: Audit,
}

/// Junction table between role and permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolPermiso {
    pub id: Uuid,
    pub rol_id: Uuid,
    pub permiso_id: Uuid,
}

/// Active user session with hashed token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sesion {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub token_hash: String,
    pub expira_en: DateTime<Utc>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Sesion {
    pub fn esta_expirada(&self, now: DateTime<Utc>) -> bool {
        now >= self.expira_en
    }
}

/// External authentication provider link for SSO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthExterno {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub proveedor: String,
    pub proveedor_user_id: String,
    pub email: Option<String>,
    pub vinculado_en: DateTime<Utc>,
}
