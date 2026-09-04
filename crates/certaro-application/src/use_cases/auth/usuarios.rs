use uuid::Uuid;

use crate::dtos::auth::{ActualizarUsuarioInput, CrearUsuarioInput, RolDto, UsuarioConDetalleDto, UsuarioDto};
use crate::error::AppError;
use crate::result::AppResult;
use certaro_domain::entities::Usuario;
use certaro_domain::RowVersion;

use super::AuthService;

impl AuthService {
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
}
