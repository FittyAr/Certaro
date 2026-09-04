use uuid::Uuid;

use crate::dtos::auth::{ActualizarRolInput, CrearRolInput, PermisoDto, RolConPermisosDto, RolDto};
use crate::error::AppError;
use crate::result::AppResult;
use certaro_domain::entities::Rol;
use certaro_domain::RowVersion;

use super::AuthService;

impl AuthService {
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
