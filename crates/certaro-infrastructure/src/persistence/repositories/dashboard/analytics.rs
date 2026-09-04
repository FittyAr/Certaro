use chrono::{DateTime, Utc};
use uuid::Uuid;

use certaro_application::ports::repositories::{
    FacturaPendiente, RentabilidadFila, SortDir, TotalMensual, TotalPorNombre,
};
use certaro_application::{AppError, AppResult};
use certaro_domain::{time, EstadoFactura, Money};
use sea_orm::{DbBackend, FromQueryResult, Statement, Value};

use crate::persistence::mappers;
use crate::persistence::repositories::estado_deuda_ids;
use super::common::{desde_producto, limite_sql, placeholders, MONTO_CONSOLIDADO_SQL};
use super::SeaOrmDashboardRepository;

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

impl SeaOrmDashboardRepository {
    pub(super) async fn impl_top_clientes(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>> {
        let sql = format!(
            "SELECT c.id                       AS id,
                    c.nombre                   AS nombre,
                    SUM({MONTO_CONSOLIDADO_SQL}) AS suma_bruta
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

    pub(super) async fn impl_gastos_por_categoria(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>> {
        let sql = format!(
            "SELECT cat.id                     AS id,
                    cat.nombre                 AS nombre,
                    SUM({MONTO_CONSOLIDADO_SQL}) AS suma_bruta
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

    pub(super) async fn impl_serie_mensual(&self, anio: i32) -> AppResult<Vec<TotalMensual>> {
        let sql = format!(
            "SELECT CAST(strftime('%m', m.fecha) AS INTEGER) AS mes,
                    tm.es_ingreso                            AS es_ingreso,
                    SUM({MONTO_CONSOLIDADO_SQL})             AS suma_bruta
               FROM movimientos m
               JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
              WHERE m.is_deleted = 0
                AND CAST(strftime('%Y', m.fecha) AS INTEGER) = ?1
              GROUP BY mes, tm.es_ingreso"
        );

        let rows = MensualRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            [Value::from(anio)],
        ))
        .all(self.conn())
        .await
        .map_err(AppError::persistence)?;

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

    pub(super) async fn impl_rentabilidad_proyectos(
        &self,
        dir: SortDir,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>> {
        let orden = match dir {
            SortDir::Asc => "ASC",
            SortDir::Desc => "DESC",
        };
        let sql = format!(
            "SELECT o.id     AS id,
                    o.nombre AS etiqueta,
                    ''       AS contexto,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 1
                                      THEN {MONTO_CONSOLIDADO_SQL} END), 0) AS ingresos,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 0
                                      THEN {MONTO_CONSOLIDADO_SQL} END), 0) AS gastos
               FROM proyectos o
               LEFT JOIN trabajos t         ON t.proyecto_id = o.id AND t.is_deleted = 0
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

    pub(super) async fn impl_rentabilidad_trabajos(
        &self,
        proyecto_id: Option<Uuid>,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>> {
        let mut values: Vec<Value> = Vec::new();
        let filtro = match proyecto_id {
            Some(id) => {
                values.push(Value::from(id.to_string()));
                " AND t.proyecto_id = ?1"
            }
            None => "",
        };
        let sql = format!(
            "SELECT t.id          AS id,
                    t.descripcion AS etiqueta,
                    o.nombre      AS contexto,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 1
                                      THEN {MONTO_CONSOLIDADO_SQL} END), 0) AS ingresos,
                    COALESCE(SUM(CASE WHEN tm.es_ingreso = 0
                                      THEN {MONTO_CONSOLIDADO_SQL} END), 0) AS gastos
               FROM trabajos t
               JOIN proyectos o                  ON o.id = t.proyecto_id
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

    pub(super) async fn impl_facturas_pendientes(
        &self,
        cliente_id: Option<Uuid>,
        incluir_pagadas: bool,
    ) -> AppResult<Vec<FacturaPendiente>> {
        let estados = if incluir_pagadas {
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
}
