//! The mandatory state-machine tests of `docs/08-maquinas-de-estado.md` §8, domain half.
//!
//! The completeness tests matter more than the individual cases: they walk every ordered pair of
//! states, so adding a variant without deciding where it can go makes them fail instead of
//! silently leaving a hole.

use std::collections::HashSet;

use chrono::NaiveDate;
use certaro_domain::entities::{Factura, PagoFactura};
use certaro_domain::{
    recalcular_estado_factura, Audit, DomainError, EstadoFactura, EstadoObra, EstadoTrabajo, Money,
    StateMachine,
};
use uuid::Uuid;

fn ahora() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(0, 0).unwrap()
}

fn dia(d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, d).unwrap()
}

fn factura(estado: EstadoFactura, total: &str, pagos: &[&str]) -> Factura {
    Factura {
        id: Uuid::from_u128(1),
        numero: "0001-00000001".into(),
        fecha: dia(1),
        fecha_vencimiento: Some(dia(31)),
        cliente_id: Uuid::from_u128(2),
        estado,
        subtotal: Money::parse(total).unwrap(),
        iva: Money::ZERO,
        total: Money::parse(total).unwrap(),
        observaciones: None,
        pagos: pagos
            .iter()
            .map(|m| PagoFactura {
                id: Uuid::new_v4(),
                factura_id: Uuid::from_u128(1),
                fecha: dia(2),
                monto: Money::parse(m).unwrap(),
                medio_pago: "Efectivo".into(),
                audit: Audit::new(ahora()),
            })
            .collect(),
        audit: Audit::new(ahora()),
    }
}

/// Every pair `(from, to)` is either allowed and succeeds, or not allowed and fails with the
/// documented error. There is no third outcome, and no state is left unexamined.
fn transiciones_exhaustivas<S: StateMachine + std::fmt::Debug>(todos: &[S]) {
    for &from in todos {
        for &to in todos {
            let esperado = from == to || from.allowed_targets().contains(&to);
            match from.transition_to(to) {
                Ok(result) => {
                    assert!(esperado, "{from:?} -> {to:?} debería ser ilegal");
                    assert_eq!(result, to);
                }
                Err(DomainError::InvalidStateTransition {
                    entity,
                    from: f,
                    to: t,
                }) => {
                    assert!(!esperado, "{from:?} -> {to:?} debería ser legal");
                    assert_eq!(entity, S::ENTITY);
                    assert_eq!(f, from.as_key());
                    assert_eq!(t, to.as_key());
                }
                Err(other) => panic!("error inesperado: {other:?}"),
            }
        }
    }
}

#[test]
fn transiciones_legales_e_ilegales_de_factura() {
    transiciones_exhaustivas(&EstadoFactura::ALL);
}

#[test]
fn transiciones_legales_e_ilegales_de_obra() {
    transiciones_exhaustivas(&EstadoObra::ALL);
}

#[test]
fn transiciones_legales_e_ilegales_de_trabajo() {
    transiciones_exhaustivas(&EstadoTrabajo::ALL);
}

#[test]
fn transicion_a_si_mismo_es_ok() {
    // A retried command must not fail just because the first attempt landed.
    for estado in EstadoFactura::ALL {
        assert_eq!(estado.transition_to(estado).unwrap(), estado);
    }
    for estado in EstadoObra::ALL {
        assert_eq!(estado.transition_to(estado).unwrap(), estado);
    }
    for estado in EstadoTrabajo::ALL {
        assert_eq!(estado.transition_to(estado).unwrap(), estado);
    }
}

#[test]
fn los_unicos_terminales_son_los_de_factura() {
    let terminales: Vec<_> = EstadoFactura::ALL
        .into_iter()
        .filter(|e| e.is_terminal())
        .collect();
    assert_eq!(
        terminales,
        vec![EstadoFactura::Pagada, EstadoFactura::Anulada]
    );

    assert!(EstadoObra::ALL.into_iter().all(|e| !e.is_terminal()));
    assert!(EstadoTrabajo::ALL.into_iter().all(|e| !e.is_terminal()));
}

#[test]
fn as_key_es_unico_y_estable() {
    fn unicas<S: StateMachine>(todos: &[S]) {
        let claves: HashSet<_> = todos.iter().map(|s| s.as_key()).collect();
        assert_eq!(claves.len(), todos.len());
    }
    unicas(&EstadoFactura::ALL);
    unicas(&EstadoObra::ALL);
    unicas(&EstadoTrabajo::ALL);

    // Stable: the keys are the last segment of an i18n key and of the wire contract.
    assert_eq!(EstadoFactura::PagadaParcial.as_key(), "PagadaParcial");
    assert_eq!(EstadoTrabajo::EnProceso.as_key(), "EnProceso");
}

#[test]
fn los_estados_automaticos_no_son_destinos_de_usuario() {
    let automaticos = [
        EstadoFactura::Pagada,
        EstadoFactura::PagadaParcial,
        EstadoFactura::Vencida,
    ];
    for estado in EstadoFactura::ALL {
        for automatico in automaticos {
            assert!(
                !estado.allowed_targets().contains(&automatico),
                "{estado:?} no debería poder ir a {automatico:?} por acción del usuario"
            );
        }
    }
}

#[test]
fn un_pago_parcial_deja_la_factura_en_pagada_parcial() {
    let mut f = factura(EstadoFactura::Emitida, "1000", &["400"]);
    recalcular_estado_factura(&mut f, dia(5), 30).unwrap();
    assert_eq!(f.estado, EstadoFactura::PagadaParcial);
    assert_eq!(f.saldo_pendiente().unwrap(), Money::parse("600").unwrap());
}

#[test]
fn un_pago_total_deja_la_factura_pagada() {
    let mut f = factura(EstadoFactura::PagadaParcial, "1000", &["400", "600"]);
    recalcular_estado_factura(&mut f, dia(5), 30).unwrap();
    assert_eq!(f.estado, EstadoFactura::Pagada);
}

#[test]
fn borrar_todos_los_pagos_vuelve_a_emitida_y_no_a_borrador() {
    let mut f = factura(EstadoFactura::Pagada, "1000", &[]);
    recalcular_estado_factura(&mut f, dia(5), 30).unwrap();
    assert_eq!(f.estado, EstadoFactura::Emitida);
}

#[test]
fn el_saldo_manda_sobre_el_vencimiento() {
    // Paid in full after the due date is `Pagada`, not `Vencida`: there is nothing left to chase.
    let mut f = factura(EstadoFactura::Vencida, "1000", &["1000"]);
    recalcular_estado_factura(&mut f, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(), 30).unwrap();
    assert_eq!(f.estado, EstadoFactura::Pagada);
}

#[test]
fn con_saldo_y_fecha_pasada_queda_vencida() {
    let mut f = factura(EstadoFactura::PagadaParcial, "1000", &["400"]);
    recalcular_estado_factura(&mut f, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), 30).unwrap();
    assert_eq!(f.estado, EstadoFactura::Vencida);
}

#[test]
fn el_vencimiento_se_calcula_sin_columna() {
    let mut f = factura(EstadoFactura::Emitida, "1000", &[]);
    f.fecha_vencimiento = None;
    recalcular_estado_factura(&mut f, dia(31), 30).unwrap();
    assert_eq!(f.estado, EstadoFactura::Emitida);

    recalcular_estado_factura(&mut f, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), 30).unwrap();
    assert_eq!(f.estado, EstadoFactura::Vencida);
}

#[test]
fn el_recalculo_es_idempotente() {
    let mut f = factura(EstadoFactura::Emitida, "1000", &["400"]);
    recalcular_estado_factura(&mut f, dia(5), 30).unwrap();
    let primero = f.estado;
    recalcular_estado_factura(&mut f, dia(5), 30).unwrap();
    assert_eq!(f.estado, primero);
}

#[test]
fn el_recalculo_no_toca_borrador_ni_anulada() {
    for estado in [EstadoFactura::Borrador, EstadoFactura::Anulada] {
        let mut f = factura(estado, "1000", &["1000"]);
        recalcular_estado_factura(&mut f, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(), 30)
            .unwrap();
        assert_eq!(f.estado, estado);
    }
}
