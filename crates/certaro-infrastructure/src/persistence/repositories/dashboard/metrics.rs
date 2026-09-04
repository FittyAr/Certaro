use chrono::{DateTime, NaiveDate, Utc};
use certaro_application::ports::repositories::{EstadoBase, MovimientoResumen};
use certaro_application::{AppError, AppResult};
use certaro_domain::{time, EstadoFactura, EstadoProyecto, Money};
use sea_orm::{DbBackend, FromQueryResult, Statement, Value};

use crate::persistence::mappers;
use super::common::{
    desde_producto, estados_abiertos_de_trabajo, placeholders, TamanoRow, MONTO_CONSOLIDADO_SQL,
};
use super::SeaOrmDashboardRepository;

#[derive(Debug, FromQueryResult)]
struct ResumenRow {
    es_ingreso: bool,
    suma_bruta: Option<i64>,
    cantidad: i64,
}

impl SeaOrmDashboardRepository {
    pub(super) async fn impl_resumen_rango(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
    ) -> AppResult<MovimientoResumen> {
        let sql = format!(
            "SELECT tm.es_ingreso                  AS es_ingreso,
                    SUM({MONTO_CONSOLIDADO_SQL})   AS suma_bruta,
                    COUNT(m.id)                    AS cantidad
               FROM movimientos m
               JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
              WHERE m.is_deleted = 0
                AND m.fecha >= ?1
                AND m.fecha <= ?2
              GROUP BY tm.es_ingreso"
        );

        let rows = ResumenRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            [
                Value::from(time::to_storage(desde)),
                Value::from(time::to_storage(hasta)),
            ],
        ))
        .all(self.conn())
        .await
        .map_err(AppError::persistence)?;

        let mut ingresos = Money::ZERO;
        let mut gastos = Money::ZERO;
        let mut cantidad = 0_u64;

        for row in rows {
            let monto = desde_producto(row.suma_bruta)?;
            if row.es_ingreso {
                ingresos = monto;
            } else {
                gastos = monto;
            }
            cantidad += row.cantidad.max(0) as u64;
        }

        Ok(MovimientoResumen {
            total_ingresos: ingresos,
            total_gastos: gastos,
            balance: ingresos.checked_sub(gastos).map_err(AppError::from)?,
            cantidad,
        })
    }

    pub(super) async fn impl_clientes_activos(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
    ) -> AppResult<u64> {
        let sql = "
            SELECT COUNT(DISTINCT m.cliente_id) AS total
              FROM movimientos m
              JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
             WHERE m.is_deleted = 0
               AND m.cliente_id IS NOT NULL
               AND tm.es_ingreso = 1
               AND m.fecha >= ?1
               AND m.fecha <= ?2";
        self.scalar(
            sql,
            vec![
                Value::from(time::to_storage(desde)),
                Value::from(time::to_storage(hasta)),
            ],
        )
        .await
    }

    pub(super) async fn impl_trabajos_pendientes(&self) -> AppResult<u64> {
        let estados = estados_abiertos_de_trabajo();
        let sql = format!(
            "SELECT COUNT(id) AS total FROM trabajos
              WHERE is_deleted = 0 AND estado IN ({})",
            placeholders(1, estados.len())
        );
        self.scalar(&sql, estados.into_iter().map(Value::from).collect())
            .await
    }

    pub(super) async fn impl_proyectos_pausadas(&self) -> AppResult<u64> {
        let sql = "SELECT COUNT(id) AS total FROM proyectos
                    WHERE is_deleted = 0 AND estado = ?1";
        self.scalar(sql, vec![Value::from(EstadoProyecto::Pausada.as_i32())])
            .await
    }

    pub(super) async fn impl_facturas_vencidas(&self, umbral: NaiveDate) -> AppResult<u64> {
        let sql = "
            SELECT COUNT(f.id) AS total
              FROM facturas f
             WHERE f.is_deleted = 0
               AND (
                     f.estado = ?1
                     OR (f.estado = ?2 AND f.fecha <= ?3)
                     OR (f.estado = ?2 AND f.fecha_vencimiento IS NOT NULL
                         AND f.fecha_vencimiento < ?4)
                   )
               AND f.total > COALESCE((
                     SELECT SUM(p.monto) FROM pagos_factura p
                      WHERE p.factura_id = f.id AND p.is_deleted = 0
                   ), 0)";
        self.scalar(
            sql,
            vec![
                Value::from(EstadoFactura::Vencida.as_i32()),
                Value::from(EstadoFactura::Emitida.as_i32()),
                Value::from(mappers::civil_to_storage(umbral)),
                Value::from(mappers::civil_to_storage(umbral)),
            ],
        )
        .await
    }

    pub(super) async fn impl_liquidaciones_pendientes(&self, anio: i32, mes: u32) -> AppResult<u64> {
        let sql = "
            SELECT COUNT(e.id) AS total
              FROM empleados e
             WHERE e.is_deleted = 0
               AND e.activo = 1
               AND NOT EXISTS (
                     SELECT 1 FROM liquidaciones l
                      WHERE l.empleado_id = e.id
                        AND l.is_deleted = 0
                        AND CAST(strftime('%Y', l.fecha_fin) AS INTEGER) = ?1
                        AND CAST(strftime('%m', l.fecha_fin) AS INTEGER) = ?2
                    )";
        self.scalar(sql, vec![Value::from(anio), Value::from(mes as i32)])
            .await
    }

    pub(super) async fn impl_estado_base(&self) -> AppResult<EstadoBase> {
        let tamano = TamanoRow::find_by_statement(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT (SELECT page_count FROM pragma_page_count()) * (SELECT page_size FROM pragma_page_size()) AS bytes",
        ))
        .one(self.conn())
        .await;

        let migraciones = self
            .scalar(
                "SELECT COUNT(version) AS total FROM seaql_migrations",
                Vec::new(),
            )
            .await
            .unwrap_or(0);

        Ok(match tamano {
            Ok(row) => EstadoBase {
                healthy: true,
                migraciones,
                tamano_bytes: row.map_or(0, |r| r.bytes),
            },
            Err(_) => EstadoBase {
                healthy: false,
                migraciones,
                tamano_bytes: 0,
            },
        })
    }
}
