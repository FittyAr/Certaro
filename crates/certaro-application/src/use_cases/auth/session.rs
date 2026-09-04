use chrono::Duration;
use uuid::Uuid;

use crate::dtos::auth::{
    Configurar2faResponse, LoginRequest, LoginResponse, RolDto, UsuarioConDetalleDto, UsuarioDto,
    Verificar2faInput,
};
use crate::error::AppError;
use crate::result::AppResult;
use certaro_domain::entities::Sesion;

use super::AuthService;

impl AuthService {
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
}
