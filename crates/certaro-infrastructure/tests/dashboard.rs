//! End-to-end exercise of the dashboard and the commercial analysis against a real database:
//! the KPIs and their comparison, profitability with its indirect imputation, the account
//! statement and the ageing buckets.

#[path = "dashboard/common.rs"]
mod common;
#[path = "dashboard/kpis_series.rs"]
mod kpis_series;
#[path = "dashboard/rentabilidad.rs"]
mod rentabilidad;
#[path = "dashboard/comercial.rs"]
mod comercial;
#[path = "dashboard/sistema_movimientos.rs"]
mod sistema_movimientos;
