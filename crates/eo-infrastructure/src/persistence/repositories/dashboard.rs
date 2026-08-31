//! Aggregated reads for the dashboard and the commercial analysis.
//! See `docs/06-casos-de-uso-y-formulas.md` §4.5, §4.6, §7 y §9.
//!
//! Unlike the CRUD repositories these are written as parameterised SQL rather than through the
//! query builder. The queries are grouped aggregations over three or four joined tables, and
//! expressed as builder calls they became unreadable without becoming any safer: every value here
//! is still bound, and no identifier is ever interpolated.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use eo_application::ports::repositories::{
    DashboardRepository, EstadoBase, FacturaPendiente, MovimientoResumen, RentabilidadFila,
    SortDir, TotalMensual, TotalPorNombre,
};
use eo_application::{AppError, AppResult};
use eo_domain::{time, EstadoFactura, EstadoObra, EstadoTrabajo, Money};
use sea_orm::{DatabaseTransaction, DbBackend, FromQueryResult, Statement, Value};
use uuid::Uuid;

use crate::persistence::mappers;
use crate::persistence::repositories::estado_deuda_ids;

pub struct SeaOrmDashboardRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmDashboardRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }

    async fn scalar(&self, sql: &str, values: Vec<Value>) -> AppResult<u64> {
        let row = ConteoRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            sql,
            values,
        ))
        .one(self.conn())
        .await
        .map_err(AppError::persistence)?;
        Ok(row.map_or(0, |r| r.total.max(0) as u64))
    }
}

#[derive(Debug, FromQueryResult)]
struct ConteoRow {
    total: i64,
}

#[derive(Debug, FromQueryResult)]
struct ResumenRow {
    es_ingreso: bool,
    suma_bruta: Option<i64>,
    cantidad: i64,
}

#[derive(Debug, FromQueryResult)]
struct TotalRow {
    id: Option<String>,
    nombre: String,
    suma_bruta: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct MensualRow {
    mes: i32,
    es_ingreso: bool,
    suma_bruta: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct RentabilidadRow {
    id: String,
    etiqueta: String,
    contexto: String,
    ingresos: Option<i64>,
    gastos: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct PendienteRow {
    id: String,
    cliente_id: String,
    cliente_nombre: String,
    numero: String,
    fecha: String,
    fecha_vencimiento: Option<String>,
    estado: i32,
    total: i64,
    pagado: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct TamanoRow {
    bytes: i64,
}

/// The product of two values scaled by 10 000 comes back scaled by 100 000 000.
fn desde_producto(suma: Option<i64>) -> AppResult<Money> {
    Money::from_product_sum(i128::from(suma.unwrap_or(0))).map_err(AppError::from)
}

/// `?1, ?2, …` for an `IN` list of `n` bound values, offset by the parameters already used.
fn placeholders(desde: usize, n: usize) -> String {
    (desde..desde + n)
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn estados_abiertos_de_trabajo() -> Vec<i32> {
    EstadoTrabajo::ALL
        .iter()
        .filter(|e| e.esta_abierto())
        .map(|e| e.as_i32())
        .collect()
}

/// `LIMIT` clause, or nothing when the caller asked for every row.
fn limite_sql(limite: u64) -> String {
    if limite == 0 {
        String::new()
    } else {
        format!(" LIMIT {limite}")
    }
}

#[async_trait]
impl DashboardRepository for SeaOrmDashboardRepository {
    async fn resumen_rango(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
    ) -> AppResult<MovimientoResumen> {
        let sql = "
            SELECT tm.es_ingreso                  AS es_ingreso,
                   SUM(m.monto * m.cantidad)      AS suma_bruta,
                   COUNT(m.id)                    AS cantidad
              FROM movimientos m
              JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
             WHERE m.is_deleted = 0
               AND m.fecha >= ?1
               AND m.fecha <= ?2
             GROUP BY tm.es_ingreso";

        let rows = ResumenRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            sql,
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

    async fn clientes_activos(&self, desde: DateTime<Utc>, hasta: DateTime<Utc>) -> AppResult<u64> {
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

    async fn trabajos_pendientes(&self) -> AppResult<u64> {
        let estados = estados_abiertos_de_trabajo();
        let sql = format!(
            "SELECT COUNT(id) AS total FROM trabajos
              WHERE is_deleted = 0 AND estado IN ({})",
            placeholders(1, estados.len())
        );
        self.scalar(&sql, estados.into_iter().map(Value::from).collect())
            .await
    }

    async fn obras_pausadas(&self) -> AppResult<u64> {
        let sql = "SELECT COUNT(id) AS total FROM obras
                    WHERE is_deleted = 0 AND estado = ?1";
        self.scalar(sql, vec![Value::from(EstadoObra::Pausada.as_i32())])
            .await
    }

    async fn facturas_vencidas(&self, umbral: NaiveDate) -> AppResult<u64> {
        // The outstanding balance is required in both arms: an invoice left in `Vencida` after
        // being collected is a stale state, not a debt, and counting it would raise a false alarm.
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

    async fn liquidaciones_pendientes(&self, anio: i32, mes: u32) -> AppResult<u64> {
        // `strftime` over the stored civil date: the period is compared by calendar month, which
        // is what doc 06 §9.4 asks for even though the rest of the dashboard rolls.
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

    async fn top_clientes(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>> {
        // Grouped by id and not by name: two customers can share a name, and the screen needs the
        // id to navigate.
        let sql = format!(
            "SELECT c.id                       AS id,
                    c.nombre                   AS nombre,
                    SUM(m.monto * m.cantidad)  AS suma_bruta
               FROM movimientos m
               JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
               JOIN clientes c          ON c.id = m.cliente_id
              WHERE m.is_deleted = 0
                AND tm.es_ingreso = 1
                AND m.fecha >= ?1
                AND m.fecha <= ?2
              GROUP BY c.id, c.nombre
              ORDER BY suma_bruta DESC{}",
            limite_sql(limite)
        );

        let rows = TotalRow::find_by_statement(Statement::from_sql_and_values(
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

        rows.into_iter()
            .map(|row| {
                Ok(TotalPorNombre {
                    id: row.id.as_deref().map(mappers::uuid).transpose()?,
                    nombre: row.nombre,
                    total: desde_producto(row.suma_bruta)?,
                })
            })
            .collect()
    }

    async fn gastos_por_categoria(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>> {
        let sql = format!(
            "SELECT cat.id                     AS id,
                    cat.nombre                 AS nombre,
                    SUM(m.monto * m.cantidad)  AS suma_bruta
               FROM movimientos m
               JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
               JOIN categorias cat      ON cat.id = m.categoria_id
              WHERE m.is_deleted = 0
                AND tm.es_ingreso = 0
                AND m.fecha >= ?1
                AND m.fecha <= ?2
              GROUP BY cat.id, cat.nombre
              ORDER BY suma_bruta DESC{}",
            limite_sql(limite)
        );

        let rows = TotalRow::find_by_statement(Statement::from_sql_and_values(
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

        rows.into_iter()
            .map(|row| {
                Ok(TotalPorNombre {
                    id: row.id.as_deref().map(mappers::uuid).transpose()?,
                    nombre: row.nombre,
                    total: desde_producto(row.suma_bruta)?,
                })
            })
            .collect()
    }

    async fn serie_mensual(&self, anio: i32) -> AppResult<Vec<TotalMensual>> {
        let sql = "
            SELECT CAST(strftime('%m', m.fecha) AS INTEGER) AS mes,
                   tm.es_ingreso                            AS es_ingreso,
                   SUM(m.monto * m.cantidad)                AS suma_bruta
              FROM movimientos m
              JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
             WHERE m.is_deleted = 0
               AND CAST(strftime('%Y', m.fecha) AS INTEGER) = ?1
             GROUP BY mes, tm.es_ingreso";

        let rows = MensualRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            sql,
            [Value::from(anio)],
        ))
        .all(self.conn())
        .await
        .map_err(AppError::persistence)?;

        // The twelve months always come back, empty ones as zero: the chart draws a full year and
        // filling the gaps in the frontend would put arithmetic there.
        let mut serie: Vec<TotalMensual> = (1..=12)
            .map(|mes| TotalMensual {
                mes,
                ingresos: Money::ZERO,
                gastos: Money::ZERO,
            })
            .collect();

        for row in rows {
            let Some(slot) = usize::try_from(row.mes)
                .ok()
                .and_then(|m| m.checked_sub(1))
                .and_then(|i| serie.get_mut(i))
            else {
                continue;
            };
            let monto = desde_producto(row.suma_bruta)?;
            if row.es_ingreso {
                slot.ingresos = monto;
            } else {
                slot.gastos = monto;
            }
        }

        Ok(serie)
    }

    async fn rentabilidad_obras(
        &self,
        dir: SortDir,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>> {
        // Indirect imputation: a movement reaches a site only through its job, so one without
        // `trabajo_id` is imputed nowhere (doc 06 §7.1).
        let orden = match dir {
            SortDir::Asc => "ASC",
            SortDir::Desc => "DESC",
        };
        let sql = format!(
            "SELECT o.id     AS id,
                    o.nombre AS etiqueta,
                    ''       AS contexto,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 1
                                      THEN m.monto * m.cantidad END), 0) AS ingresos,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 0
                                      THEN m.monto * m.cantidad END), 0) AS gastos
               FROM obras o
               LEFT JOIN trabajos t         ON t.obra_id = o.id AND t.is_deleted = 0
               LEFT JOIN movimientos m      ON m.trabajo_id = t.id AND m.is_deleted = 0
               LEFT JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
              WHERE o.is_deleted = 0
              GROUP BY o.id, o.nombre
              ORDER BY (ingresos - gastos) {orden}, o.nombre ASC{}",
            limite_sql(limite)
        );

        let rows = RentabilidadRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            [],
        ))
        .all(self.conn())
        .await
        .map_err(AppError::persistence)?;

        rows.into_iter().map(fila).collect()
    }

    async fn rentabilidad_trabajos(
        &self,
        obra_id: Option<Uuid>,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>> {
        let mut values: Vec<Value> = Vec::new();
        let filtro = match obra_id {
            Some(id) => {
                values.push(Value::from(id.to_string()));
                " AND t.obra_id = ?1"
            }
            None => "",
        };
        let sql = format!(
            "SELECT t.id          AS id,
                    t.descripcion AS etiqueta,
                    o.nombre      AS contexto,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 1
                                      THEN m.monto * m.cantidad END), 0) AS ingresos,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 0
                                      THEN m.monto * m.cantidad END), 0) AS gastos
               FROM trabajos t
               JOIN obras o                  ON o.id = t.obra_id
               LEFT JOIN movimientos m       ON m.trabajo_id = t.id AND m.is_deleted = 0
               LEFT JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
              WHERE t.is_deleted = 0{filtro}
              GROUP BY t.id, t.descripcion, o.nombre
              ORDER BY (ingresos - gastos) DESC, t.descripcion ASC{}",
            limite_sql(limite)
        );

        let rows = RentabilidadRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            values,
        ))
        .all(self.conn())
        .await
        .map_err(AppError::persistence)?;

        rows.into_iter().map(fila).collect()
    }

    async fn facturas_pendientes(
        &self,
        cliente_id: Option<Uuid>,
        incluir_pagadas: bool,
    ) -> AppResult<Vec<FacturaPendiente>> {
        let estados = if incluir_pagadas {
            // Everything except the states that are not receivables at all: a draft was never
            // sent and a voided invoice is not owed.
            EstadoFactura::ALL
                .iter()
                .filter(|e| !matches!(e, EstadoFactura::Borrador | EstadoFactura::Anulada))
                .map(|e| e.as_i32())
                .collect::<Vec<_>>()
        } else {
            estado_deuda_ids()
        };

        let mut values: Vec<Value> = estados.iter().copied().map(Value::from).collect();
        let mut sql = format!(
            "SELECT f.id                AS id,
                    f.cliente_id        AS cliente_id,
                    c.nombre            AS cliente_nombre,
                    f.numero            AS numero,
                    f.fecha             AS fecha,
                    f.fecha_vencimiento AS fecha_vencimiento,
                    f.estado            AS estado,
                    f.total             AS total,
                    (SELECT SUM(p.monto) FROM pagos_factura p
                      WHERE p.factura_id = f.id AND p.is_deleted = 0) AS pagado
               FROM facturas f
               JOIN clientes c ON c.id = f.cliente_id
              WHERE f.is_deleted = 0
                AND f.estado IN ({})",
            placeholders(1, estados.len())
        );

        if let Some(id) = cliente_id {
            sql.push_str(&format!(" AND f.cliente_id = ?{}", values.len() + 1));
            values.push(Value::from(id.to_string()));
        }
        sql.push_str(" ORDER BY f.fecha DESC, f.numero DESC");

        let rows = PendienteRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            values,
        ))
        .all(self.conn())
        .await
        .map_err(AppError::persistence)?;

        rows.into_iter()
            .map(|row| {
                Ok(FacturaPendiente {
                    id: mappers::uuid(&row.id)?,
                    cliente_id: mappers::uuid(&row.cliente_id)?,
                    cliente_nombre: row.cliente_nombre,
                    numero: row.numero,
                    fecha: mappers::civil(&row.fecha)?,
                    fecha_vencimiento: mappers::civil_opt(row.fecha_vencimiento.as_deref())?,
                    estado: EstadoFactura::from_i32(row.estado).map_err(AppError::from)?,
                    total: Money::from_raw(row.total),
                    pagado: Money::from_raw(row.pagado.unwrap_or(0)),
                })
            })
            .collect()
    }

    async fn estado_base(&self) -> AppResult<EstadoBase> {
        // `SELECT 1` would tell us the handle is alive; asking for the page count tells us the
        // file is readable, which is what "healthy" means to the user.
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

fn fila(row: RentabilidadRow) -> AppResult<RentabilidadFila> {
    let ingresos = desde_producto(row.ingresos)?;
    let gastos = desde_producto(row.gastos)?;
    Ok(RentabilidadFila {
        id: mappers::uuid(&row.id)?,
        etiqueta: row.etiqueta,
        contexto: row.contexto,
        ingresos,
        gastos,
        rentabilidad: ingresos.checked_sub(gastos).map_err(AppError::from)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_placeholders_se_numeran_desde_el_offset() {
        assert_eq!(placeholders(1, 3), "?1, ?2, ?3");
        assert_eq!(placeholders(4, 2), "?4, ?5");
    }

    #[test]
    fn el_limite_cero_no_emite_clausula() {
        assert_eq!(limite_sql(0), "");
        assert_eq!(limite_sql(5), " LIMIT 5");
    }

    #[test]
    fn los_estados_abiertos_de_trabajo_excluyen_los_terminales() {
        let abiertos = estados_abiertos_de_trabajo();
        assert!(!abiertos.contains(&EstadoTrabajo::Finalizado.as_i32()));
        assert!(!abiertos.contains(&EstadoTrabajo::Cancelado.as_i32()));
        assert_eq!(abiertos.len(), 3);
    }
}
