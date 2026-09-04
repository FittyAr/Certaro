//! Closed enumerations of the domain. See `docs/05-dominio-entidades.md` §3.
//!
//! Each one persists as the integer of the document and is transported to the frontend as its
//! name, so a value read from the database keeps its meaning while the contract stays legible.

pub mod estados;
pub mod moneda;
pub mod personal;

pub use estados::{EstadoFactura, EstadoProyecto, EstadoTrabajo};
pub use moneda::{MedioPago, Moneda};
pub use personal::{FrecuenciaPago, TipoJornada};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decimal4::Decimal4;

    #[test]
    fn la_moneda_por_defecto_es_el_peso() {
        assert_eq!(Moneda::default(), Moneda::Ars);
    }

    #[test]
    fn ida_y_vuelta_por_el_entero_persistido() {
        for moneda in [Moneda::Ars, Moneda::Usd] {
            assert_eq!(Moneda::from_i32(moneda.as_i32()).unwrap(), moneda);
        }
    }

    #[test]
    fn un_valor_desconocido_no_se_adivina() {
        assert!(Moneda::from_i32(7).is_err());
    }

    #[test]
    fn solo_el_dolar_exige_cotizacion() {
        assert!(Moneda::Usd.requiere_cotizacion());
        assert!(!Moneda::Ars.requiere_cotizacion());
    }

    #[test]
    fn los_estados_van_y_vuelven_por_su_entero() {
        for estado in EstadoFactura::ALL {
            assert_eq!(EstadoFactura::from_i32(estado.as_i32()).unwrap(), estado);
        }
        for estado in EstadoProyecto::ALL {
            assert_eq!(EstadoProyecto::from_i32(estado.as_i32()).unwrap(), estado);
        }
        for estado in EstadoTrabajo::ALL {
            assert_eq!(EstadoTrabajo::from_i32(estado.as_i32()).unwrap(), estado);
        }
    }

    #[test]
    fn pagada_parcial_conserva_el_cinco() {
        assert_eq!(EstadoFactura::PagadaParcial.as_i32(), 5);
        assert_eq!(EstadoFactura::Pagada.as_i32(), 2);
    }

    #[test]
    fn solo_las_facturas_vivas_con_saldo_admiten_pagos() {
        assert!(EstadoFactura::Emitida.admite_pagos());
        assert!(EstadoFactura::PagadaParcial.admite_pagos());
        assert!(EstadoFactura::Vencida.admite_pagos());
        assert!(!EstadoFactura::Borrador.admite_pagos());
        assert!(!EstadoFactura::Pagada.admite_pagos());
        assert!(!EstadoFactura::Anulada.admite_pagos());
    }

    #[test]
    fn el_borrador_y_la_anulada_no_son_deuda() {
        assert!(!EstadoFactura::Borrador.cuenta_como_deuda());
        assert!(!EstadoFactura::Anulada.cuenta_como_deuda());
        assert!(EstadoFactura::Emitida.cuenta_como_deuda());
        assert!(EstadoFactura::PagadaParcial.cuenta_como_deuda());
    }

    #[test]
    fn un_trabajo_cerrado_no_esta_abierto() {
        assert!(EstadoTrabajo::Presupuestado.esta_abierto());
        assert!(EstadoTrabajo::EnProceso.esta_abierto());
        assert!(EstadoTrabajo::Pausado.esta_abierto());
        assert!(!EstadoTrabajo::Finalizado.esta_abierto());
        assert!(!EstadoTrabajo::Cancelado.esta_abierto());
    }

    #[test]
    fn la_jornada_y_la_frecuencia_van_y_vuelven_por_su_entero() {
        for tipo in TipoJornada::ALL {
            assert_eq!(TipoJornada::from_i32(tipo.as_i32()).unwrap(), tipo);
        }
        for frecuencia in FrecuenciaPago::ALL {
            assert_eq!(
                FrecuenciaPago::from_i32(frecuencia.as_i32()).unwrap(),
                frecuencia
            );
        }
    }

    #[test]
    fn una_ausencia_no_se_paga_aunque_este_justificada() {
        assert_eq!(TipoJornada::Falta.factor(), Decimal4::ZERO);
        assert_eq!(TipoJornada::FaltaJustificada.factor(), Decimal4::ZERO);
    }

    #[test]
    fn la_media_jornada_vale_medio_dia() {
        assert_eq!(TipoJornada::Media.factor(), Decimal4::HALF);
        assert_eq!(TipoJornada::Completa.factor(), Decimal4::ONE);
        assert_eq!(TipoJornada::Feriado.factor(), Decimal4::ONE);
    }

    #[test]
    fn el_ciclo_de_click_vuelve_al_vacio() {
        let mut actual = None;
        let mut recorrido = Vec::new();
        for _ in 0..6 {
            actual = TipoJornada::siguiente(actual);
            recorrido.push(actual);
        }
        assert_eq!(
            recorrido,
            vec![
                Some(TipoJornada::Completa),
                Some(TipoJornada::Media),
                Some(TipoJornada::Falta),
                Some(TipoJornada::FaltaJustificada),
                Some(TipoJornada::Feriado),
                None,
            ]
        );
    }

    #[test]
    fn la_semana_laboral_tiene_seis_dias() {
        assert_eq!(
            FrecuenciaPago::Semanal.dias_por_periodo(),
            Decimal4::from_units(6).unwrap()
        );
        assert_eq!(
            FrecuenciaPago::Mensual.dias_por_periodo(),
            Decimal4::from_units(30).unwrap()
        );
        assert_eq!(
            FrecuenciaPago::Quincenal.dias_por_periodo(),
            Decimal4::from_units(15).unwrap()
        );
        assert_eq!(FrecuenciaPago::Diario.dias_por_periodo(), Decimal4::ONE);
    }
}
