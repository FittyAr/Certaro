use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use certaro_application::ports::repositories::AsistenciaRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::AsistenciaEmpleado;
use certaro_domain::time;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::persistence::mappers::{self, asistencia_empleado as mapper};
use crate::persistence::models::asistencia_empleado::{Column, Entity};

pub struct SeaOrmAsistenciaRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmAsistenciaRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl AsistenciaRepository for SeaOrmAsistenciaRepository {
    /// Deliberately without the `is_deleted` filter: the unique index covers deleted rows, so the
    /// upsert has to see a cleared cell in order to revive it instead of colliding with it.
    async fn find_por_empleado_fecha(
        &self,
        empleado_id: Uuid,
        fecha: NaiveDate,
    ) -> AppResult<Option<AsistenciaEmpleado>> {
        let found = Entity::find()
            .filter(Column::EmpleadoId.eq(empleado_id.to_string()))
            .filter(Column::Fecha.eq(mappers::civil_to_storage(fecha)))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn del_periodo(
        &self,
        desde: NaiveDate,
        hasta: NaiveDate,
        empleados: &[Uuid],
    ) -> AppResult<Vec<AsistenciaEmpleado>> {
        let mut query = Entity::find()
            .filter(Column::IsDeleted.eq(false))
            .filter(Column::Fecha.gte(mappers::civil_to_storage(desde)))
            .filter(Column::Fecha.lte(mappers::civil_to_storage(hasta)));

        if !empleados.is_empty() {
            query = query.filter(
                Column::EmpleadoId.is_in(empleados.iter().map(Uuid::to_string).collect::<Vec<_>>()),
            );
        }

        let rows = query
            .order_by_asc(Column::EmpleadoId)
            .order_by_asc(Column::Fecha)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn insert(&self, entity: &AsistenciaEmpleado) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    /// No row version: the identity is `(empleado_id, fecha)` and the last click wins. Optimistic
    /// concurrency on a grid with no save button would only produce conflicts nobody can resolve.
    async fn update(&self, entity: &AsistenciaEmpleado) -> AppResult<()> {
        let result = Entity::update_many()
            .set(mapper::to_active(entity))
            .filter(Column::Id.eq(entity.id.to_string()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::not_found("AsistenciaEmpleado", entity.id));
        }
        Ok(())
    }

    async fn soft_delete_por_empleado_fecha(
        &self,
        empleado_id: Uuid,
        fecha: NaiveDate,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        Entity::update_many()
            .col_expr(Column::IsDeleted, Expr::value(true))
            .col_expr(Column::DeletedAt, Expr::value(time::to_storage(at)))
            .col_expr(Column::UpdatedAt, Expr::value(time::to_storage(at)))
            .filter(Column::EmpleadoId.eq(empleado_id.to_string()))
            .filter(Column::Fecha.eq(mappers::civil_to_storage(fecha)))
            .filter(Column::IsDeleted.eq(false))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
