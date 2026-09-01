//! Commercial analysis: account statement, ageing of receivables and profitability.
//! See `docs/06-casos-de-uso-y-formulas.md` §4.5, §4.6 y §7.
//!
//! The statement and the ageing read the same rows through `facturas_pendientes`, so the debt a
//! customer shows on one screen is the debt they show on the other.

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::NaiveDate;
use certaro_domain::Money;
use uuid::Uuid;

use crate::dtos::comercial::{
    AntiguedadDeuda, AntiguedadDeudaCliente, AntiguedadDeudaQuery, BucketsDeuda, CuentaCorriente,
    CuentaCorrienteFactura, CuentaCorrienteQuery,
};
use crate::dtos::dashboard::RentabilidadItem;
use crate::ports::repositories::{FacturaPendiente, SortDir, UnitOfWork};
use crate::ports::{ClockPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::finish_read;

pub struct ComercialService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    settings: Arc<dyn SettingsStore>,
}

impl ComercialService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            clock,
            settings,
        }
    }

    /// A customer's statement. An unknown or deleted customer yields an empty statement instead of
    /// an error: the screen navigates here from a stale link often enough that failing would be
    /// worse than showing nothing (doc 06 §4.5).
    pub async fn cuenta_corriente(
        &self,
        query: CuentaCorrienteQuery,
    ) -> AppResult<CuentaCorriente> {
        let hoy = self.clock.now_utc().date_naive();
        let dias_default = self
            .settings
            .snapshot()
            .business
            .factura_dias_vencimiento_default;

        let tx = self.uow.begin().await?;
        let outcome = async {
            let cliente = tx.clientes().find_by_id(query.cliente_id).await?;
            let facturas = tx
                .dashboard()
                .facturas_pendientes(Some(query.cliente_id), query.incluir_pagadas)
                .await?;
            Ok((cliente, facturas))
        }
        .await;
        let (cliente, pendientes) = finish_read(tx, outcome).await?;

        let mut filas = Vec::new();
        let mut total_facturado = Money::ZERO;
        let mut total_pagado = Money::ZERO;
        let mut saldo_total = Money::ZERO;

        for factura in pendientes {
            let saldo = factura.saldo()?;
            // A settled invoice is history, not a statement line, unless it was asked for.
            if saldo.is_zero() && !query.incluir_pagadas {
                continue;
            }
            total_facturado = total_facturado.checked_add(factura.total)?;
            total_pagado = total_pagado.checked_add(factura.pagado)?;
            saldo_total = saldo_total.checked_add(saldo)?;
            filas.push(CuentaCorrienteFactura {
                id: factura.id,
                numero: factura.numero.clone(),
                fecha: factura.fecha,
                fecha_vencimiento: factura.fecha_vencimiento,
                estado: factura.estado,
                total: factura.total,
                pagado: factura.pagado,
                saldo,
                dias_mora: dias_mora(&factura, saldo, hoy, dias_default),
            });
        }

        // Newest first: the statement is read from the last invoice backwards.
        filas.sort_by(|a, b| b.fecha.cmp(&a.fecha).then_with(|| b.numero.cmp(&a.numero)));

        Ok(CuentaCorriente {
            cliente_id: query.cliente_id,
            cliente_nombre: cliente.map(|c| c.nombre).unwrap_or_default(),
            total_facturado,
            total_pagado,
            saldo: saldo_total,
            facturas: filas,
        })
    }

    /// Ageing of the receivables, globally or for one customer. The bucket bounds come from
    /// configuration, and the four of them always add up to the total (doc 06 §4.6).
    pub async fn antiguedad_deuda(
        &self,
        query: AntiguedadDeudaQuery,
    ) -> AppResult<AntiguedadDeuda> {
        let config = self.settings.snapshot();
        let limites = config.business.buckets_antiguedad.clone();
        let dias_default = config.business.factura_dias_vencimiento_default;
        let corte = query
            .fecha_corte
            .unwrap_or_else(|| self.clock.now_utc().date_naive());

        let tx = self.uow.begin().await?;
        let result = tx
            .dashboard()
            .facturas_pendientes(query.cliente_id, false)
            .await;
        let pendientes = finish_read(tx, result).await?;

        let mut global = BucketsDeuda::default();
        let mut por_cliente: BTreeMap<Uuid, (String, BucketsDeuda)> = BTreeMap::new();

        for factura in pendientes {
            let saldo = factura.saldo()?;
            if !saldo.is_positive() {
                continue;
            }
            let dias = (corte - factura.fecha_base(dias_default)).num_days().max(0);
            global.add(dias, saldo, &limites)?;
            let entry = por_cliente
                .entry(factura.cliente_id)
                .or_insert_with(|| (factura.cliente_nombre.clone(), BucketsDeuda::default()));
            entry.1.add(dias, saldo, &limites)?;
        }

        let mut detalle: Vec<AntiguedadDeudaCliente> = por_cliente
            .into_iter()
            .map(|(cliente_id, (cliente_nombre, b))| AntiguedadDeudaCliente {
                cliente_id,
                cliente_nombre,
                total: b.total,
                bucket0a30: b.b0a30,
                bucket31a60: b.b31a60,
                bucket61a90: b.b61a90,
                bucket_mas90: b.mas90,
            })
            .collect();
        detalle.sort_by_key(|c| std::cmp::Reverse(c.total));

        Ok(AntiguedadDeuda {
            fecha_corte: corte,
            total: global.total,
            bucket0a30: global.b0a30,
            bucket31a60: global.b31a60,
            bucket61a90: global.b61a90,
            bucket_mas90: global.mas90,
            limites,
            detalle,
        })
    }

    /// Sites ranked by profitability, best first. `limite` of zero means every site.
    pub async fn rentabilidad_obras(
        &self,
        limite: Option<u64>,
    ) -> AppResult<Vec<RentabilidadItem>> {
        let limite = limite.unwrap_or(0);
        let tx = self.uow.begin().await?;
        let result = tx
            .dashboard()
            .rentabilidad_obras(SortDir::Desc, limite)
            .await;
        let filas = finish_read(tx, result).await?;
        Ok(filas.into_iter().map(RentabilidadItem::from).collect())
    }

    /// Jobs ranked by profitability, optionally within one site. Doc 06 §7.3: the legacy system
    /// had no such report, and the imputation chain is the same one the site ranking uses.
    pub async fn rentabilidad_trabajos(
        &self,
        obra_id: Option<Uuid>,
        limite: Option<u64>,
    ) -> AppResult<Vec<RentabilidadItem>> {
        let limite = limite.unwrap_or(0);
        let tx = self.uow.begin().await?;
        let result = tx.dashboard().rentabilidad_trabajos(obra_id, limite).await;
        let filas = finish_read(tx, result).await?;
        Ok(filas.into_iter().map(RentabilidadItem::from).collect())
    }
}

/// Days in arrears. Zero once the invoice is settled: a paid row is never late, however long it
/// took (doc 06 §4.5).
fn dias_mora(factura: &FacturaPendiente, saldo: Money, hoy: NaiveDate, dias_default: u32) -> i64 {
    if !saldo.is_positive() {
        return 0;
    }
    (hoy - factura.fecha_base(dias_default)).num_days().max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn money(v: &str) -> Money {
        Money::parse(v).unwrap()
    }

    fn limites() -> Vec<u32> {
        vec![30, 60, 90]
    }

    #[test]
    fn los_bordes_de_los_buckets_son_inclusivos_por_arriba() {
        let saldo = money("100.0000");
        let casos = [
            (0_i64, "b0a30"),
            (30, "b0a30"),
            (31, "b31a60"),
            (60, "b31a60"),
            (61, "b61a90"),
            (90, "b61a90"),
            (91, "mas90"),
        ];

        for (dias, esperado) in casos {
            let mut b = BucketsDeuda::default();
            b.add(dias, saldo, &limites()).unwrap();
            let obtenido = if b.b0a30 == saldo {
                "b0a30"
            } else if b.b31a60 == saldo {
                "b31a60"
            } else if b.b61a90 == saldo {
                "b61a90"
            } else {
                "mas90"
            };
            assert_eq!(obtenido, esperado, "con {dias} días");
        }
    }

    #[test]
    fn los_buckets_suman_el_total() {
        let mut b = BucketsDeuda::default();
        for (dias, monto) in [
            (10, "100.0000"),
            (45, "200.0000"),
            (80, "300.0000"),
            (400, "50.0000"),
        ] {
            b.add(dias, money(monto), &limites()).unwrap();
        }
        let suma = Money::try_sum([b.b0a30, b.b31a60, b.b61a90, b.mas90]).unwrap();
        assert_eq!(suma, b.total);
        assert_eq!(b.total, money("650.0000"));
    }

    #[test]
    fn una_configuracion_de_buckets_invalida_no_pierde_plata() {
        let mut b = BucketsDeuda::default();
        b.add(5, money("100.0000"), &[]).unwrap();
        assert_eq!(b.total, money("100.0000"));
        assert_eq!(b.mas90, money("100.0000"));
    }

    #[test]
    fn la_mora_se_cuenta_desde_el_vencimiento_cuando_existe() {
        let hoy = NaiveDate::from_ymd_opt(2026, 8, 29).unwrap();
        let factura = FacturaPendiente {
            id: Uuid::nil(),
            cliente_id: Uuid::nil(),
            cliente_nombre: String::new(),
            numero: "0001".to_owned(),
            fecha: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            fecha_vencimiento: NaiveDate::from_ymd_opt(2026, 8, 19),
            estado: certaro_domain::EstadoFactura::Emitida,
            total: money("1000.0000"),
            pagado: Money::ZERO,
        };

        assert_eq!(dias_mora(&factura, money("1000.0000"), hoy, 30), 10);
        // Settled: not late, no matter how old.
        assert_eq!(dias_mora(&factura, Money::ZERO, hoy, 30), 0);

        // With no due date the default term applies, as in the invoice list.
        let sin_vencimiento = FacturaPendiente {
            fecha_vencimiento: None,
            ..factura
        };
        assert_eq!(dias_mora(&sin_vencimiento, money("1000.0000"), hoy, 30), 59);
    }

    #[test]
    fn una_factura_todavia_no_vencida_no_tiene_mora() {
        let hoy = NaiveDate::from_ymd_opt(2026, 8, 29).unwrap();
        let factura = FacturaPendiente {
            id: Uuid::nil(),
            cliente_id: Uuid::nil(),
            cliente_nombre: String::new(),
            numero: "0002".to_owned(),
            fecha: hoy,
            fecha_vencimiento: NaiveDate::from_ymd_opt(2026, 9, 30),
            estado: certaro_domain::EstadoFactura::Emitida,
            total: money("500.0000"),
            pagado: Money::ZERO,
        };
        assert_eq!(dias_mora(&factura, money("500.0000"), hoy, 30), 0);
    }

    #[test]
    fn un_sobrepago_no_genera_saldo_negativo() {
        let factura = FacturaPendiente {
            id: Uuid::nil(),
            cliente_id: Uuid::nil(),
            cliente_nombre: String::new(),
            numero: "0003".to_owned(),
            fecha: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            fecha_vencimiento: None,
            estado: certaro_domain::EstadoFactura::Pagada,
            total: money("100.0000"),
            pagado: money("120.0000"),
        };
        assert_eq!(factura.saldo().unwrap(), Money::ZERO);
    }
}
