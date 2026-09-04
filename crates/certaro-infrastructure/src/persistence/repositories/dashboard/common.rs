use certaro_application::{AppError, AppResult};
use certaro_domain::{EstadoTrabajo, Money};
use sea_orm::FromQueryResult;

pub const MONTO_CONSOLIDADO_SQL: &str = "(CASE WHEN m.moneda = 1 AND m.cotizacion_aplicada IS NOT NULL AND m.cotizacion_aplicada > 0 THEN (m.monto * m.cotizacion_aplicada / 10000) * m.cantidad ELSE m.monto * m.cantidad END)";

#[derive(Debug, FromQueryResult)]
pub struct ConteoRow {
    pub total: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct TamanoRow {
    pub bytes: i64,
}

/// The product of two values scaled by 10 000 comes back scaled by 100 000 000.
pub fn desde_producto(suma: Option<i64>) -> AppResult<Money> {
    Money::from_product_sum(i128::from(suma.unwrap_or(0))).map_err(AppError::from)
}

/// `?1, ?2, …` for an `IN` list of `n` bound values, offset by the parameters already used.
pub fn placeholders(desde: usize, n: usize) -> String {
    (desde..desde + n)
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn estados_abiertos_de_trabajo() -> Vec<i32> {
    EstadoTrabajo::ALL
        .iter()
        .filter(|e| e.esta_abierto())
        .map(|e| e.as_i32())
        .collect()
}

/// `LIMIT` clause, or nothing when the caller asked for every row.
pub fn limite_sql(limite: u64) -> String {
    if limite == 0 {
        String::new()
    } else {
        format!(" LIMIT {limite}")
    }
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
