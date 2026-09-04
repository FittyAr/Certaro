use certaro_domain::Decimal4;
use serde::{Deserialize, Serialize};
use super::super::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SettlementConfig {
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    pub incluir_sabado: bool,
    pub incluir_domingo: bool,
    pub incluir_feriado: bool,
    pub periodo_por_defecto_dias: u32,
    pub sincronizar_feriados_al_iniciar: bool,
    /// The current year plus the next ones.
    pub anios_feriados_a_sincronizar: u8,
    /// Maximum number of days the attendance grid may query in one request.
    /// 92 covers the last 3 months and keeps the matrix bounded.
    pub asistencia_max_rango_dias: u32,
}

impl Default for SettlementConfig {
    fn default() -> Self {
        Self {
            multiplicador_sabado: Decimal4::from_raw(15_000),
            multiplicador_domingo: Decimal4::from_raw(20_000),
            multiplicador_feriado: Decimal4::from_raw(20_000),
            incluir_sabado: false,
            incluir_domingo: false,
            incluir_feriado: false,
            periodo_por_defecto_dias: 15,
            sincronizar_feriados_al_iniciar: true,
            anios_feriados_a_sincronizar: 2,
            asistencia_max_rango_dias: 92,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DashboardConfig {
    pub last_period: DashboardPeriod,
    pub privacy_mode: bool,
    pub casas_dolar: Vec<String>,
    pub cotizacion_por_defecto: String,
    pub top_clientes_cantidad: u8,
    pub ultimos_movimientos_cantidad: u8,
    pub proyectos_ranking_cantidad: u8,
    pub top_categorias_cantidad: u8,
    /// A drop steeper than this, in percent, raises the falling-income alert (doc 06 §9.11).
    pub alerta_caida_ingresos_pct: Decimal4,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            last_period: DashboardPeriod::Mensual,
            privacy_mode: false,
            casas_dolar: vec!["oficial".to_owned(), "blue".to_owned()],
            cotizacion_por_defecto: "blue".to_owned(),
            top_clientes_cantidad: 5,
            ultimos_movimientos_cantidad: 10,
            proyectos_ranking_cantidad: 5,
            top_categorias_cantidad: 5,
            alerta_caida_ingresos_pct: Decimal4::from_raw(200_000),
        }
    }
}
