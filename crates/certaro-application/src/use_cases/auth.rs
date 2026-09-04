//! Use cases for Authentication, RBAC, Sessions, and 2FA.

use std::sync::Arc;
use chrono::Duration;
use uuid::Uuid;

use crate::dtos::auth::{
    ActualizarRolInput, ActualizarUsuarioInput, Configurar2faResponse, CrearRolInput,
    CrearUsuarioInput, LoginRequest, LoginResponse, PermisoDto, RolConPermisosDto, RolDto,
    UsuarioConDetalleDto, UsuarioDto, Verificar2faInput,
};
use crate::error::AppError;
use crate::ports::auth::{PasswordHasher, TokenPort, TotpPort};
use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;
use crate::ports::settings::SettingsStore;
use crate::result::AppResult;
use certaro_domain::entities::{Rol, Sesion, Usuario};
use certaro_domain::RowVersion;

pub struct AuthService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
    #[allow(dead_code)]
    settings: Arc<dyn SettingsStore>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenPort>,
    totp: Arc<dyn TotpPort>,
}

impl AuthService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
        settings: Arc<dyn SettingsStore>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenPort>,
        totp: Arc<dyn TotpPort>,
    ) -> Self {
        Self {
            uow,
            clock,
            ids,
            settings,
            hasher,
            tokens,
            totp,
        }
    }

    /// Authenticates with email and password, checks 2FA, and creates a session.
    pub async fn login(
        &self,
        req: LoginRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<LoginResponse> {
        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;

        let mut user = tx
            .usuarios()
            .find_by_email(&req.email.trim().to_lowercase())
            .await?
            .ok_or_else(|| AppError::conflict("INVALID_CREDENTIALS", "Validation.Auth.InvalidCredentials"))?;

        if !user.activo {
            return Err(AppError::conflict("USER_INACTIVE", "Validation.Auth.UserInactive"));
        }

        if user.esta_bloqueado(now) {
            return Err(AppError::conflict("USER_LOCKED", "Validation.Auth.UserLocked"));
        }

        let pwd_hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| AppError::conflict("PASSWORD_NOT_SET", "Validation.Auth.PasswordNotSet"))?;

        let valid_pwd = self.hasher.verify_password(&req.password, pwd_hash)?;
        if !valid_pwd {
            user.registrar_fallo(5, 15, now);
            tx.usuarios().update(&user, user.audit.row_version).await?;
            tx.commit().await?;
            return Err(AppError::conflict("INVALID_CREDENTIALS", "Validation.Auth.InvalidCredentials"));
        }

        // Check 2FA if required
        if user.requiere_2fa {
            let secret = user
                .totp_secret
                .as_deref()
                .ok_or_else(|| AppError::conflict("2FA_NOT_CONFIGURED", "Validation.Auth.2faNotConfigured"))?;
            let code = req
                .totp_code
                .as_deref()
                .ok_or_else(|| AppError::conflict("2FA_CODE_REQUIRED", "Validation.Auth.2faCodeRequired"))?;

            if !self.totp.verify_code(secret, code) {
                return Err(AppError::conflict("2FA_INVALID", "Validation.Auth.Invalid2faCode"));
            }
        }

        // Reset failed login count and record login time
        user.reset_intentos(now);
        tx.usuarios().update(&user, user.audit.row_version).await?;

        // Create session
        let raw_token = self.tokens.generate_token();
        let token_hash = self.tokens.hash_token(&raw_token);
        let session = Sesion {
            id: self.ids.new_id(),
            usuario_id: user.id,
            token_hash,
            expira_en: now + Duration::days(7),
            ip,
            user_agent,
            created_at: now,
        };
        tx.sesiones().insert(&session).await?;

        // Query user roles and permissions
        let roles = tx.roles().get_roles_for_usuario(user.id).await?;
        let permisos = tx.permisos().get_permisos_for_usuario(user.id).await?;

        tx.commit().await?;

        let user_dto = UsuarioDto {
            id: user.id,
            email: user.email,
            nombre_completo: user.nombre_completo,
            activo: user.activo,
            requiere_2fa: user.requiere_2fa,
            ultimo_login: user.ultimo_login,
            row_version: user.audit.row_version.to_hex(),
        };

        Ok(LoginResponse {
            token: raw_token,
            usuario: user_dto,
            roles: roles.into_iter().map(|r| r.nombre).collect(),
            permisos: permisos.into_iter().map(|p| p.clave).collect(),
            requiere_2fa: user.requiere_2fa,
        })
    }

    pub async fn logout(&self, token: &str) -> AppResult<()> {
        let token_hash = self.tokens.hash_token(token);
        let tx = self.uow.begin().await?;
        tx.sesiones().delete_by_token_hash(&token_hash).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn validar_sesion(&self, token: &str) -> AppResult<Option<UsuarioConDetalleDto>> {
        let now = self.clock.now_utc();
        let token_hash = self.tokens.hash_token(token);
        let tx = self.uow.begin().await?;

        let sesion = match tx.sesiones().find_by_token_hash(&token_hash).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        if sesion.esta_expirada(now) {
            tx.sesiones().delete_by_token_hash(&token_hash).await?;
            tx.commit().await?;
            return Ok(None);
        }

        let user = match tx.usuarios().find_by_id(sesion.usuario_id).await? {
            Some(u) if u.activo => u,
            _ => return Ok(None),
        };

        let roles = tx.roles().get_roles_for_usuario(user.id).await?;
        let permisos = tx.permisos().get_permisos_for_usuario(user.id).await?;

        let rol_dtos = roles
            .into_iter()
            .map(|r| RolDto {
                id: r.id,
                nombre: r.nombre,
                descripcion: r.descripcion,
                es_sistema: r.es_sistema,
                prioridad: r.prioridad,
                row_version: r.audit.row_version.to_hex(),
            })
            .collect();

        Ok(Some(UsuarioConDetalleDto {
            usuario: UsuarioDto {
                id: user.id,
                email: user.email,
                nombre_completo: user.nombre_completo,
                activo: user.activo,
                requiere_2fa: user.requiere_2fa,
                ultimo_login: user.ultimo_login,
                row_version: user.audit.row_version.to_hex(),
            },
            roles: rol_dtos,
            permisos: permisos.into_iter().map(|p| p.clave).collect(),
        }))
    }

    pub async fn configurar_2fa(&self, usuario_id: Uuid) -> AppResult<Configurar2faResponse> {
        let secret = self.totp.generate_secret();
        let tx = self.uow.begin().await?;
        let user = tx
            .usuarios()
            .find_by_id(usuario_id)
            .await?
            .ok_or_else(|| AppError::not_found("Usuario", usuario_id))?;

        let otpauth_url = format!(
            "otpauth://totp/Certaro:{}?secret={}&issuer=Certaro",
            user.email, secret
        );

        Ok(Configurar2faResponse { secret, otpauth_url })
    }

    pub async fn activar_2fa(&self, usuario_id: Uuid, input: Verificar2faInput) -> AppResult<()> {
        if !self.totp.verify_code(&input.secret, &input.code) {
            return Err(AppError::conflict("INVALID_2FA_CODE", "Validation.Auth.Invalid2faCode"));
        }

        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        let mut user = tx
            .usuarios()
            .find_by_id(usuario_id)
            .await?
            .ok_or_else(|| AppError::not_found("Usuario", usuario_id))?;

        user.requiere_2fa = true;
        user.totp_secret = Some(input.secret);
        user.audit.touch(now);

        tx.usuarios().update(&user, user.audit.row_version).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn desactivar_2fa(&self, usuario_id: Uuid) -> AppResult<()> {
        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        let mut user = tx
            .usuarios()
            .find_by_id(usuario_id)
            .await?
            .ok_or_else(|| AppError::not_found("Usuario", usuario_id))?;

        user.requiere_2fa = false;
        user.totp_secret = None;
        user.audit.touch(now);

        tx.usuarios().update(&user, user.audit.row_version).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn listar_usuarios(&self) -> AppResult<Vec<UsuarioDto>> {
        let tx = self.uow.begin().await?;
        let rows = tx.usuarios().list_all().await?;
        Ok(rows
            .into_iter()
            .map(|u| UsuarioDto {
                id: u.id,
                email: u.email,
                nombre_completo: u.nombre_completo,
                activo: u.activo,
                requiere_2fa: u.requiere_2fa,
                ultimo_login: u.ultimo_login,
                row_version: u.audit.row_version.to_hex(),
            })
            .collect())
    }

    pub async fn obtener_usuario(&self, id: Uuid) -> AppResult<UsuarioConDetalleDto> {
        let tx = self.uow.begin().await?;
        let user = tx
            .usuarios()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Usuario", id))?;

        let roles = tx.roles().get_roles_for_usuario(id).await?;
        let permisos = tx.permisos().get_permisos_for_usuario(id).await?;

        let rol_dtos = roles
            .into_iter()
            .map(|r| RolDto {
                id: r.id,
                nombre: r.nombre,
                descripcion: r.descripcion,
                es_sistema: r.es_sistema,
                prioridad: r.prioridad,
                row_version: r.audit.row_version.to_hex(),
            })
            .collect();

        Ok(UsuarioConDetalleDto {
            usuario: UsuarioDto {
                id: user.id,
                email: user.email,
                nombre_completo: user.nombre_completo,
                activo: user.activo,
                requiere_2fa: user.requiere_2fa,
                ultimo_login: user.ultimo_login,
                row_version: user.audit.row_version.to_hex(),
            },
            roles: rol_dtos,
            permisos: permisos.into_iter().map(|p| p.clave).collect(),
        })
    }

    pub async fn crear_usuario(&self, input: CrearUsuarioInput) -> AppResult<UsuarioDto> {
        let now = self.clock.now_utc();
        let email = input.email.trim().to_lowercase();
        let tx = self.uow.begin().await?;

        if tx.usuarios().find_by_email(&email).await?.is_some() {
            return Err(AppError::conflict("DUPLICATE_EMAIL", "Validation.Auth.EmailAlreadyExists"));
        }

        let password_hash = match input.password {
            Some(ref pwd) if !pwd.trim().is_empty() => Some(self.hasher.hash_password(pwd)?),
            _ => None,
        };

        let id = self.ids.new_id();
        let user = Usuario {
            id,
            email: email.clone(),
            nombre_completo: input.nombre_completo.trim().to_owned(),
            password_hash,
            activo: true,
            requiere_2fa: input.requiere_2fa,
            totp_secret: None,
            ultimo_login: None,
            intentos_fallidos: 0,
            bloqueado_hasta: None,
            audit: certaro_domain::entities::Audit::new(now),
        };

        tx.usuarios().insert(&user).await?;

        for role_id in input.roles {
            tx.roles().assign_rol_to_usuario(id, role_id, now).await?;
        }

        tx.commit().await?;

        Ok(UsuarioDto {
            id: user.id,
            email: user.email,
            nombre_completo: user.nombre_completo,
            activo: user.activo,
            requiere_2fa: user.requiere_2fa,
            ultimo_login: user.ultimo_login,
            row_version: user.audit.row_version.to_hex(),
        })
    }

    pub async fn actualizar_usuario(
        &self,
        id: Uuid,
        input: ActualizarUsuarioInput,
    ) -> AppResult<UsuarioDto> {
        let now = self.clock.now_utc();
        let esperado = RowVersion::parse_hex(&input.row_version)
            .map_err(|e| AppError::unexpected(anyhow::anyhow!("Invalid row version: {e}")))?;
        let tx = self.uow.begin().await?;

        let mut user = tx
            .usuarios()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Usuario", id))?;

        user.nombre_completo = input.nombre_completo.trim().to_owned();
        user.activo = input.activo;
        user.requiere_2fa = input.requiere_2fa;

        if let Some(ref pwd) = input.password {
            if !pwd.trim().is_empty() {
                user.password_hash = Some(self.hasher.hash_password(pwd)?);
            }
        }

        user.audit.touch(now);
        tx.usuarios().update(&user, esperado).await?;

        // Reassign roles
        let current_roles = tx.roles().get_roles_for_usuario(id).await?;
        for cr in current_roles {
            if !input.roles.contains(&cr.id) {
                tx.roles().remove_rol_from_usuario(id, cr.id).await?;
            }
        }
        for nr in input.roles {
            tx.roles().assign_rol_to_usuario(id, nr, now).await?;
        }

        tx.commit().await?;

        Ok(UsuarioDto {
            id: user.id,
            email: user.email,
            nombre_completo: user.nombre_completo,
            activo: user.activo,
            requiere_2fa: user.requiere_2fa,
            ultimo_login: user.ultimo_login,
            row_version: user.audit.row_version.to_hex(),
        })
    }

    pub async fn eliminar_usuario(&self, id: Uuid, version: RowVersion) -> AppResult<()> {
        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        tx.usuarios().soft_delete(id, version, now).await?;
        tx.sesiones().delete_by_usuario(id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn listar_roles(&self) -> AppResult<Vec<RolDto>> {
        let tx = self.uow.begin().await?;
        let rows = tx.roles().list_all().await?;
        Ok(rows
            .into_iter()
            .map(|r| RolDto {
                id: r.id,
                nombre: r.nombre,
                descripcion: r.descripcion,
                es_sistema: r.es_sistema,
                prioridad: r.prioridad,
                row_version: r.audit.row_version.to_hex(),
            })
            .collect())
    }

    pub async fn obtener_rol(&self, id: Uuid) -> AppResult<RolConPermisosDto> {
        let tx = self.uow.begin().await?;
        let rol = tx
            .roles()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Rol", id))?;

        let permisos = tx.permisos().get_permisos_for_rol(id).await?;

        Ok(RolConPermisosDto {
            rol: RolDto {
                id: rol.id,
                nombre: rol.nombre,
                descripcion: rol.descripcion,
                es_sistema: rol.es_sistema,
                prioridad: rol.prioridad,
                row_version: rol.audit.row_version.to_hex(),
            },
            permisos: permisos
                .into_iter()
                .map(|p| PermisoDto {
                    id: p.id,
                    modulo: p.modulo,
                    accion: p.accion,
                    recurso: p.recurso,
                    clave: p.clave,
                })
                .collect(),
        })
    }

    pub async fn crear_rol(&self, input: CrearRolInput) -> AppResult<RolDto> {
        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;

        if tx.roles().find_by_nombre(input.nombre.trim()).await?.is_some() {
            return Err(AppError::conflict("DUPLICATE_ROLE", "Validation.Role.NameAlreadyExists"));
        }

        let id = self.ids.new_id();
        let rol = Rol {
            id,
            nombre: input.nombre.trim().to_owned(),
            descripcion: input.descripcion,
            es_sistema: false,
            prioridad: input.prioridad,
            audit: certaro_domain::entities::Audit::new(now),
        };

        tx.roles().insert(&rol).await?;

        for p_id in input.permisos {
            tx.permisos().assign_permiso_to_rol(id, p_id).await?;
        }

        tx.commit().await?;

        Ok(RolDto {
            id: rol.id,
            nombre: rol.nombre,
            descripcion: rol.descripcion,
            es_sistema: rol.es_sistema,
            prioridad: rol.prioridad,
            row_version: rol.audit.row_version.to_hex(),
        })
    }

    pub async fn actualizar_rol(&self, id: Uuid, input: ActualizarRolInput) -> AppResult<RolDto> {
        let now = self.clock.now_utc();
        let esperado = RowVersion::parse_hex(&input.row_version)
            .map_err(|e| AppError::unexpected(anyhow::anyhow!("Invalid row version: {e}")))?;
        let tx = self.uow.begin().await?;

        let mut rol = tx
            .roles()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Rol", id))?;

        rol.nombre = input.nombre.trim().to_owned();
        rol.descripcion = input.descripcion;
        rol.prioridad = input.prioridad;
        rol.audit.touch(now);

        tx.roles().update(&rol, esperado).await?;

        // Reassign permissions
        let current_permisos = tx.permisos().get_permisos_for_rol(id).await?;
        for cp in current_permisos {
            if !input.permisos.contains(&cp.id) {
                tx.permisos().remove_permiso_from_rol(id, cp.id).await?;
            }
        }
        for np in input.permisos {
            tx.permisos().assign_permiso_to_rol(id, np).await?;
        }

        tx.commit().await?;

        Ok(RolDto {
            id: rol.id,
            nombre: rol.nombre,
            descripcion: rol.descripcion,
            es_sistema: rol.es_sistema,
            prioridad: rol.prioridad,
            row_version: rol.audit.row_version.to_hex(),
        })
    }

    pub async fn eliminar_rol(&self, id: Uuid, version: RowVersion) -> AppResult<()> {
        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        tx.roles().soft_delete(id, version, now).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn listar_permisos(&self) -> AppResult<Vec<PermisoDto>> {
        let tx = self.uow.begin().await?;
        let rows = tx.permisos().list_all().await?;
        Ok(rows
            .into_iter()
            .map(|p| PermisoDto {
                id: p.id,
                modulo: p.modulo,
                accion: p.accion,
                recurso: p.recurso,
                clave: p.clave,
            })
            .collect())
    }
}
