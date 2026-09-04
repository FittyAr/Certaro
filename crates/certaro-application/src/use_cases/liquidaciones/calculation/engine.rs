//! Calculation engine for liquidaciones suggestions and breakdowns.

use std::collections::HashSet;
use chrono::{Datelike, NaiveDate, Weekday};
use certaro_domain::entities::{Empleado, ReglasLiquidacion};
use certaro_domain::{Decimal4, Money, TipoJornada};

use crate::dtos::liquidaciones::{
    LiquidacionAdelantoSugerido, LiquidacionDesglose, LiquidacionSugerencia, OrigenLiquidacion,
};
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
