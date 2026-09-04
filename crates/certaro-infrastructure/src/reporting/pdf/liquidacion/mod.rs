//! PDF of a payroll settlement. See `docs/12-reportes-y-exportaciones.md` §3.
//!
//! This is the document the client asked for by name (RC-02): every advance listed with its own
//! date, so the employee can check the number instead of trusting it. The legacy version showed a
//! single gross figure with no breakdown and no dates.
//!
//! **Deviations from doc 12 §3**, both because the frozen settlement does not store the data the
//! document assumes:
//!
//! - §3.2 asks for one premium row per day type with its count of days. A settlement freezes the
//!   multipliers but not how many Saturdays, Sundays or holidays the period had (doc 05 §2.20), so
//!   the premiums are one row with their total, derived as gross minus days × rate.
//! - §3.3 asks for a «type» column with the payment concept of each advance. The frozen advance
//!   keeps the date, the concept and the amount, not the concept type.

use certaro_application::dtos::liquidaciones::LiquidacionDetalle;
use certaro_application::dtos::reportes::GeneratedReport;
use certaro_application::result::AppResult;

use super::canvas::{Align, Canvas, TextSpec};
use super::theme::{self, size};
use crate::reporting::{filename, footer_text, ReportContext};

mod sections;

use sections::*;

pub fn generate(data: &LiquidacionDetalle, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    let mut canvas = Canvas::new(
        &ctx.t("Report.Liquidacion.Title"),
        theme::page::A4_WIDTH,
        theme::page::A4_HEIGHT,
        theme::page::MARGIN_LIQUIDACION,
    )?;

    encabezado(&mut canvas, data, ctx);
    resumen(&mut canvas, data, ctx);
    adelantos(&mut canvas, data, ctx);
    totales(&mut canvas, data, ctx);
    observaciones(&mut canvas, data, ctx);
    firmas(&mut canvas, ctx);

    let pie = |actual: usize, total: usize| {
        Some(
            TextSpec::new(footer_text(ctx, actual, total), size::FOOTER)
                .color(theme::MUTED)
                .align(Align::Center),
        )
    };
    let bytes = canvas.finish(pie)?;

    Ok(GeneratedReport {
        bytes,
        registros: data.adelantos.len() as u64,
        nombre_sugerido: filename::liquidacion(&data.empleado_nombre, data.fecha_fin),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use certaro_domain::Money;
    use crate::reporting::format::{format_date, format_money};
    use crate::reporting::tests_support::{contexto, liquidacion, pdf_text};

    #[test]
    fn pdf_liquidacion_lista_cada_adelanto_con_su_fecha() {
        let data = liquidacion(5, "300000", "0");
        let generado = generate(&data, &contexto()).unwrap();
        let texto = pdf_text(&generado.bytes);
        assert_eq!(generado.registros, 5);
        for adelanto in &data.adelantos {
            let fecha = format_date(adelanto.fecha, &contexto().locale);
            assert!(texto.contains(&fecha), "falta la fecha {fecha}: {texto}");
            assert!(
                texto.contains(&adelanto.concepto),
                "falta el concepto {}: {texto}",
                adelanto.concepto
            );
        }
    }

    #[test]
    fn pdf_liquidacion_sin_adelantos() {
        let texto = pdf_text(
            &generate(&liquidacion(0, "300000", "0"), &contexto())
                .unwrap()
                .bytes,
        );
        assert!(texto.contains("No se registraron adelantos"), "{texto}");
    }

    #[test]
    fn pdf_liquidacion_total_neto_coincide_con_el_dominio() {
        let data = liquidacion(2, "300000", "0");
        let texto = pdf_text(&generate(&data, &contexto()).unwrap().bytes);
        let neto = format_money(data.total_neto, &contexto().locale);
        assert!(
            texto.contains(&neto),
            "el neto {neto} no está en el PDF: {texto}"
        );
    }

    #[test]
    fn pdf_liquidacion_neto_negativo_se_genera_y_muestra_el_signo() {
        // The employee drew more than they earned: the total is negative and must read as such.
        let data = liquidacion(1, "10000", "50000");
        assert!(data.total_neto.is_negative());
        let texto = pdf_text(&generate(&data, &contexto()).unwrap().bytes);
        assert!(texto.contains('-'), "{texto}");
    }

    #[test]
    fn pdf_liquidacion_desglosa_los_recargos_cuando_hay() {
        // Gross above days × rate means premiums were applied.
        let mut data = liquidacion(0, "300000", "0");
        data.total_bruto = Money::parse("360000").unwrap();
        data.total_neto = data.total_bruto;
        let texto = pdf_text(&generate(&data, &contexto()).unwrap().bytes);
        assert!(texto.contains("Recargos"), "{texto}");
    }

    #[test]
    fn pdf_liquidacion_usa_el_lema_de_configuracion_y_no_un_literal() {
        let texto = pdf_text(
            &generate(&liquidacion(0, "1000", "0"), &contexto())
                .unwrap()
                .bytes,
        );
        assert!(texto.contains("Energía controlada"), "{texto}");
        assert!(
            !texto.contains("Cuentas Claras"),
            "quedó el literal legacy: {texto}"
        );
    }

    #[test]
    fn el_nombre_sugerido_lleva_empleado_y_fecha() {
        let generado = generate(&liquidacion(0, "1000", "0"), &contexto()).unwrap();
        assert!(generado.nombre_sugerido.starts_with("Liquidacion_"));
        assert!(generado.nombre_sugerido.ends_with(".pdf"));
    }
}

