//! Calculation engine for liquidaciones suggestions and breakdowns.

use std::collections::HashSet;
use chrono::{Datelike, NaiveDate, Weekday};
use certaro_domain::entities::{Empleado, Liquidacion, LiquidacionAdelanto, ReglasLiquidacion};
use certaro_domain::{Decimal4, Money, TipoJornada};
use uuid::Uuid;

use crate::dtos::liquidaciones::{
    LiquidacionAdelantoSugerido, LiquidacionDesglose, LiquidacionSugerencia, OrigenLiquidacion,
};
use crate::error::AppError;
use crate::ports::repositories::AdelantoCandidato;
use crate::result::AppResult;

pub fn construir_sugerencia(
    empleado: &Empleado,
    desde: NaiveDate,
    hasta: NaiveDate,
    dias_manuales: Option<Decimal4>,
    jornadas: &[(NaiveDate, TipoJornada)],
    feriados: &HashSet<NaiveDate>,
    candidatos: &[crate::ports::repositories::AdelantoCandidato],
) -> AppResult<LiquidacionSugerencia> {
    let reglas = reglas_de(empleado);
    let mut desglose = desglose_vacio(&reglas);

    // The comparison is exact rather than `<= 0`: a negative count is a typo, and letting it fall
    // into the calendar branch would quietly pay a full period.
    let (dias, bruto, origen) = match dias_manuales {
        Some(dias) if dias != Decimal4::ZERO => {
            // Manual: no multipliers, no calendar. The user said how many days.
            let bruto = empleado.tarifa_diaria.checked_mul(dias)?;
            (dias, bruto, OrigenLiquidacion::Manual)
        }
        _ if !jornadas.is_empty() => {
            let (dias, bruto) =
                desde_asistencia(empleado, jornadas, feriados, &reglas, &mut desglose)?;
            (dias, bruto, OrigenLiquidacion::Asistencia)
        }
        _ => {
            let (dias, bruto) =
                desde_calendario(empleado, desde, hasta, feriados, &reglas, &mut desglose)?;
            (dias, bruto, OrigenLiquidacion::Calendario)
        }
    };

    // What the multipliers added on top of the plain rate, so the PDF can show it as its own line.
    let sin_recargo = empleado.tarifa_diaria.checked_mul(dias)?;
    desglose.recargos = bruto.checked_sub(sin_recargo).unwrap_or(Money::ZERO);

    let adelantos: Vec<LiquidacionAdelantoSugerido> = candidatos
        .iter()
        .map(|c| LiquidacionAdelantoSugerido {
            movimiento_id: c.movimiento_id,
            fecha: c.fecha,
            concepto: c.concepto.clone(),
            monto: c.monto,
            ya_descontado: c.liquidacion_id.is_some(),
            liquidacion_que_lo_desconto: c.liquidacion_id,
            incluir: c.liquidacion_id.is_none(),
        })
        .collect();

    let total_adelantos = Money::try_sum(
        adelantos
            .iter()
            .filter(|a| !a.ya_descontado)
            .map(|a| a.monto),
    )?;

    Ok(LiquidacionSugerencia {
        empleado_id: empleado.id,
        empleado_nombre: empleado.nombre.clone(),
        desde,
        hasta,
        dias_trabajados: dias,
        tarifa_aplicada: empleado.tarifa_diaria,
        total_bruto: bruto,
        total_adelantos,
        total_neto: bruto.checked_sub(total_adelantos)?,
        origen,
        incluir_sabados: reglas.incluir_sabados,
        incluir_domingos: reglas.incluir_domingos,
        incluir_feriados: reglas.incluir_feriados,
        desglose,
        adelantos,
        feriados_no_disponibles: feriados.is_empty(),
    })
}

/// Branch A. A single attendance record is enough to own the period: mixing it with the calendar
/// would pay the days nobody recorded.
fn desde_asistencia(
    empleado: &Empleado,
    jornadas: &[(NaiveDate, TipoJornada)],
    feriados: &HashSet<NaiveDate>,
    reglas: &ReglasLiquidacion,
    desglose: &mut LiquidacionDesglose,
) -> AppResult<(Decimal4, Money)> {
    let mut dias = Decimal4::ZERO;
    let mut bruto = Money::ZERO;

    for (fecha, tipo) in jornadas {
        match tipo {
            TipoJornada::Completa => {
                desglose.jornadas_completas =
                    desglose.jornadas_completas.checked_add(Decimal4::ONE)?;
            }
            TipoJornada::Media => {
                desglose.jornadas_medias = desglose.jornadas_medias.checked_add(Decimal4::ONE)?;
            }
            TipoJornada::Falta => desglose.faltas += 1,
            TipoJornada::FaltaJustificada => desglose.faltas_justificadas += 1,
            TipoJornada::Feriado => {}
        }

        let factor = tipo.factor();
        if !factor.is_positive() {
            continue;
        }

        // A day recorded as a holiday does not consult the calendar: whoever loaded it already
        // said so.
        let mult = if *tipo == TipoJornada::Feriado {
            desglose.dias_feriado = desglose.dias_feriado.checked_add(factor)?;
            reglas.multiplicador_jornada_feriado()
        } else {
            contar_dia_especial(*fecha, factor, feriados, desglose)?;
            reglas.multiplicador_dia(*fecha, feriados)
        };
        if !mult.is_positive() {
            continue;
        }

        dias = dias.checked_add(factor)?;
        bruto = bruto.checked_add(
            empleado
                .tarifa_diaria
                .checked_mul(factor)?
                .checked_mul(mult)?,
        )?;
    }

    Ok((dias, bruto))
}

/// Branch B, only when there is no attendance at all. Every countable day is exactly one day, no
/// matter what its multiplier does to the money.
fn desde_calendario(
    empleado: &Empleado,
    desde: NaiveDate,
    hasta: NaiveDate,
    feriados: &HashSet<NaiveDate>,
    reglas: &ReglasLiquidacion,
    desglose: &mut LiquidacionDesglose,
) -> AppResult<(Decimal4, Money)> {
    let mut dias = Decimal4::ZERO;
    let mut bruto = Money::ZERO;

    for fecha in crate::use_cases::asistencias::fechas_del_rango(desde, hasta) {
        let mult = reglas.multiplicador_dia(fecha, feriados);
        if !mult.is_positive() {
            continue;
        }
        contar_dia_especial(fecha, Decimal4::ONE, feriados, desglose)?;
        desglose.jornadas_completas = desglose.jornadas_completas.checked_add(Decimal4::ONE)?;
        dias = dias.checked_add(Decimal4::ONE)?;
        bruto = bruto.checked_add(empleado.tarifa_diaria.checked_mul(mult)?)?;
    }

    Ok((dias, bruto))
}

fn contar_dia_especial(
    fecha: NaiveDate,
    factor: Decimal4,
    feriados: &HashSet<NaiveDate>,
    desglose: &mut LiquidacionDesglose,
) -> AppResult<()> {
    if feriados.contains(&fecha) {
        desglose.dias_feriado = desglose.dias_feriado.checked_add(factor)?;
    } else if fecha.weekday() == Weekday::Sun {
        desglose.dias_domingo = desglose.dias_domingo.checked_add(factor)?;
    } else if fecha.weekday() == Weekday::Sat {
        desglose.dias_sabado = desglose.dias_sabado.checked_add(factor)?;
    }
    Ok(())
}

/// The rules of the settlement come from the employee's card: configuration only seeds a new one.
fn reglas_de(empleado: &Empleado) -> ReglasLiquidacion {
    ReglasLiquidacion {
        // A multiplier of zero is how the card says "this day does not count", so inclusion is read
        // from the multiplier rather than from a second flag that could disagree with it.
        incluir_sabados: empleado.multiplicador_sabado.is_positive(),
        incluir_domingos: empleado.multiplicador_domingo.is_positive(),
        incluir_feriados: empleado.multiplicador_feriado.is_positive(),
        multiplicador_sabado: empleado.multiplicador_sabado,
        multiplicador_domingo: empleado.multiplicador_domingo,
        multiplicador_feriado: empleado.multiplicador_feriado,
    }
}

fn desglose_vacio(reglas: &ReglasLiquidacion) -> LiquidacionDesglose {
    LiquidacionDesglose {
        jornadas_completas: Decimal4::ZERO,
        jornadas_medias: Decimal4::ZERO,
        faltas: 0,
        faltas_justificadas: 0,
        dias_sabado: Decimal4::ZERO,
        dias_domingo: Decimal4::ZERO,
        dias_feriado: Decimal4::ZERO,
        multiplicador_sabado: reglas.multiplicador_sabado,
        multiplicador_domingo: reglas.multiplicador_domingo,
        multiplicador_feriado: reglas.multiplicador_feriado,
        recargos: Money::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::repositories::AdelantoCandidato;
    use chrono::{TimeZone, Utc};
    use certaro_domain::entities::Audit;
    use certaro_domain::FrecuenciaPago;

    fn dia(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn empleado(sabado: &str, domingo: &str, feriado: &str) -> Empleado {
        Empleado {
            id: Uuid::from_u128(1),
            nombre: "Juan".into(),
            dni: None,
            cargo: None,
            sueldo_base: Money::ZERO,
            pago_frecuencia: FrecuenciaPago::Mensual,
            tarifa_diaria: Money::from_units(40_000).unwrap(),
            multiplicador_sabado: Decimal4::parse(sabado).unwrap(),
            multiplicador_domingo: Decimal4::parse(domingo).unwrap(),
            multiplicador_feriado: Decimal4::parse(feriado).unwrap(),
            email: None,
            telefono: None,
            fecha_ingreso: dia(2020, 1, 1),
            fecha_egreso: None,
            activo: true,
            audit: Audit::new(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
        }
    }

    fn sugerir(
        empleado: &Empleado,
        desde: NaiveDate,
        hasta: NaiveDate,
        manuales: Option<Decimal4>,
        jornadas: &[(NaiveDate, TipoJornada)],
        feriados: &[NaiveDate],
        candidatos: &[AdelantoCandidato],
    ) -> LiquidacionSugerencia {
        construir_sugerencia(
            empleado,
            desde,
            hasta,
            manuales,
            jornadas,
            &feriados.iter().copied().collect(),
            candidatos,
        )
        .unwrap()
    }

    fn adelanto(monto: i64, liquidacion: Option<Uuid>) -> AdelantoCandidato {
        AdelantoCandidato {
            movimiento_id: Uuid::from_u128(u128::try_from(monto).unwrap_or(1)),
            fecha: dia(2026, 8, 5),
            concepto: "Adelanto".into(),
            monto: Money::from_units(monto).unwrap(),
            liquidacion_id: liquidacion,
        }
    }

    #[test]
    fn sugerencia_liquidacion_manual() {
        // RC-01: 10 × 40 000 = 400 000, menos 260 000 de adelantos = 140 000
        let e = empleado("1", "1", "1");
        let s = sugerir(
            &e,
            dia(2026, 8, 1),
            dia(2026, 8, 15),
            Some(Decimal4::from_units(10).unwrap()),
            &[],
            &[],
            &[
                adelanto(30_000, None),
                adelanto(40_000, None),
                adelanto(50_000, None),
                adelanto(140_000, None),
            ],
        );
        assert_eq!(s.origen, OrigenLiquidacion::Manual);
        assert_eq!(s.total_bruto, Money::from_units(400_000).unwrap());
        assert_eq!(s.total_adelantos, Money::from_units(260_000).unwrap());
        assert_eq!(s.total_neto, Money::from_units(140_000).unwrap());
    }

    #[test]
    fn sugerencia_liquidacion_desde_asistencia() {
        // lun completa 1,0 · mar media 0,5 · mié falta 0 · jue falta justificada 0 · vie feriado
        // 1,0 × 2,0 · sáb completa con multiplicador 0
        // = 2,5 días y 40 000 + 20 000 + 80 000 = 140 000
        let e = empleado("0", "0", "2");
        let s = sugerir(
            &e,
            dia(2026, 8, 24),
            dia(2026, 8, 29),
            None,
            &[
                (dia(2026, 8, 24), TipoJornada::Completa),
                (dia(2026, 8, 25), TipoJornada::Media),
                (dia(2026, 8, 26), TipoJornada::Falta),
                (dia(2026, 8, 27), TipoJornada::FaltaJustificada),
                (dia(2026, 8, 28), TipoJornada::Feriado),
                (dia(2026, 8, 29), TipoJornada::Completa),
            ],
            &[],
            &[],
        );
        assert_eq!(s.origen, OrigenLiquidacion::Asistencia);
        assert_eq!(s.dias_trabajados, Decimal4::parse("2.5").unwrap());
        assert_eq!(s.total_bruto, Money::from_units(140_000).unwrap());
        assert_eq!(s.desglose.faltas, 1);
        assert_eq!(s.desglose.faltas_justificadas, 1);
    }

    #[test]
    fn sugerencia_liquidacion_por_calendario() {
        // 2026-08-24 a 28 son lunes a viernes: 5 días hábiles × 40 000 = 200 000
        let e = empleado("0", "0", "0");
        let s = sugerir(&e, dia(2026, 8, 24), dia(2026, 8, 28), None, &[], &[], &[]);
        assert_eq!(s.origen, OrigenLiquidacion::Calendario);
        assert_eq!(s.dias_trabajados, Decimal4::from_units(5).unwrap());
        assert_eq!(s.total_bruto, Money::from_units(200_000).unwrap());
    }

    #[test]
    fn la_asistencia_gana_sobre_el_calendario() {
        // Un solo registro alcanza para que la rama de calendario no se ejecute.
        let e = empleado("0", "0", "0");
        let s = sugerir(
            &e,
            dia(2026, 8, 24),
            dia(2026, 8, 28),
            None,
            &[(dia(2026, 8, 24), TipoJornada::Completa)],
            &[],
            &[],
        );
        assert_eq!(s.origen, OrigenLiquidacion::Asistencia);
        assert_eq!(s.dias_trabajados, Decimal4::ONE);
        assert_eq!(s.total_bruto, Money::from_units(40_000).unwrap());
    }

    #[test]
    fn sugerencia_prioridad_feriado_sobre_domingo() {
        // 2026-08-30 es domingo y feriado: paga el multiplicador de feriado, 3,0.
        let e = empleado("1.5", "2", "3");
        let s = sugerir(
            &e,
            dia(2026, 8, 30),
            dia(2026, 8, 30),
            None,
            &[],
            &[dia(2026, 8, 30)],
            &[],
        );
        assert_eq!(s.total_bruto, Money::from_units(120_000).unwrap());
        assert_eq!(s.dias_trabajados, Decimal4::ONE);
    }

    #[test]
    fn sugerencia_prioridad_domingo_sobre_sabado() {
        let e = empleado("1.5", "2", "3");
        let sabado = sugerir(&e, dia(2026, 8, 29), dia(2026, 8, 29), None, &[], &[], &[]);
        let domingo = sugerir(&e, dia(2026, 8, 30), dia(2026, 8, 30), None, &[], &[], &[]);
        assert_eq!(sabado.total_bruto, Money::from_units(60_000).unwrap());
        assert_eq!(domingo.total_bruto, Money::from_units(80_000).unwrap());
    }

    #[test]
    fn sugerencia_ignora_dias_excluidos() {
        // Con el multiplicador de sábado en cero, el 29 no cuenta ni como día.
        let e = empleado("0", "0", "0");
        let s = sugerir(&e, dia(2026, 8, 29), dia(2026, 8, 30), None, &[], &[], &[]);
        assert_eq!(s.dias_trabajados, Decimal4::ZERO);
        assert_eq!(s.total_bruto, Money::ZERO);
    }

    #[test]
    fn sugerencia_no_reusa_un_adelanto_ya_liquidado() {
        // INV-05: el que ya fue descontado viaja marcado y no suma.
        let e = empleado("1", "1", "1");
        let s = sugerir(
            &e,
            dia(2026, 8, 1),
            dia(2026, 8, 15),
            Some(Decimal4::from_units(10).unwrap()),
            &[],
            &[],
            &[
                adelanto(50_000, None),
                adelanto(80_000, Some(Uuid::from_u128(9))),
            ],
        );
        assert_eq!(s.total_adelantos, Money::from_units(50_000).unwrap());
        assert!(s.adelantos[1].ya_descontado);
        assert!(!s.adelantos[1].incluir);
    }

    #[test]
    fn sugerencia_media_jornada_cuenta_medio_dia() {
        let e = empleado("0", "0", "0");
        let s = sugerir(
            &e,
            dia(2026, 8, 24),
            dia(2026, 8, 24),
            None,
            &[(dia(2026, 8, 24), TipoJornada::Media)],
            &[],
            &[],
        );
        assert_eq!(s.dias_trabajados, Decimal4::HALF);
        assert_eq!(s.total_bruto, Money::from_units(20_000).unwrap());
    }

    #[test]
    fn la_rama_manual_no_aplica_multiplicadores() {
        // 10 días a 40 000 son 400 000 aunque el período fuese todo feriados.
        let e = empleado("1.5", "2", "3");
        let s = sugerir(
            &e,
            dia(2026, 8, 1),
            dia(2026, 8, 15),
            Some(Decimal4::from_units(10).unwrap()),
            &[],
            &[dia(2026, 8, 3), dia(2026, 8, 4)],
            &[],
        );
        assert_eq!(s.total_bruto, Money::from_units(400_000).unwrap());
        assert_eq!(s.desglose.recargos, Money::ZERO);
    }

    #[test]
    fn el_test_de_referencia_da_el_valor_exacto() {
        // Doc 17 §7.2: quincena, tarifa 40 000, 12 completas, 2 medias y un sábado a 1,5.
        // Bruto = (12 × 40 000) + (2 × 0,5 × 40 000) + (1 × 1,5 × 40 000)
        //       = 480 000 + 40 000 + 60 000 = 580 000
        // Adelantos = 150 000 + 80 000 = 230 000 · Neto = 350 000
        let e = empleado("1.5", "0", "0");
        // Doce hábiles completos: 3 al 7, 10 al 14 y 17 y 18 de agosto de 2026.
        let mut jornadas: Vec<(NaiveDate, TipoJornada)> =
            [3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 17, 18]
                .into_iter()
                .map(|d| (dia(2026, 8, d), TipoJornada::Completa))
                .collect();
        jornadas.push((dia(2026, 8, 19), TipoJornada::Media));
        jornadas.push((dia(2026, 8, 20), TipoJornada::Media));
        jornadas.push((dia(2026, 8, 22), TipoJornada::Completa)); // sábado, ×1,5

        let s = sugerir(
            &e,
            dia(2026, 8, 3),
            dia(2026, 8, 22),
            None,
            &jornadas,
            &[],
            &[adelanto(150_000, None), adelanto(80_000, None)],
        );
        assert_eq!(s.total_bruto, Money::from_units(580_000).unwrap());
        assert_eq!(s.total_adelantos, Money::from_units(230_000).unwrap());
        assert_eq!(s.total_neto, Money::from_units(350_000).unwrap());
        assert_eq!(s.desglose.recargos, Money::from_units(20_000).unwrap());
    }

    #[test]
    fn un_periodo_sin_feriados_se_marca_para_advertir() {
        let e = empleado("0", "0", "2");
        let s = sugerir(&e, dia(2026, 8, 24), dia(2026, 8, 28), None, &[], &[], &[]);
        assert!(s.feriados_no_disponibles);
    }
}

