use certaro_application::dtos::liquidaciones::LiquidacionDetalle;
use super::super::super::canvas::{Align, Canvas, TextSpec};
use super::super::super::theme::{self, size};
use crate::reporting::format::format_money;
use crate::reporting::ReportContext;

const TOTALES_WIDTH: f32 = 200.0;

pub fn totales(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
    canvas.advance(30.0);
    let x = canvas.left() + canvas.content_width() - TOTALES_WIDTH;
    let padding = 10.0;
    let alto = padding * 2.0
        + Canvas::line_height(size::BODY_LIQUIDACION) * 2.0
        + Canvas::line_height(size::TOTAL)
        + 6.0;

    canvas.ensure_space(alto + 20.0);
    let top = canvas.cursor();
    canvas.rect(x, top, TOTALES_WIDTH, alto, Some(theme::TOTAL_FILL), None);

    let inner_x = x + padding;
    let inner_w = TOTALES_WIDTH - 2.0 * padding;
    let mut y = top + padding;

    let fila = |canvas: &Canvas,
                y: f32,
                etiqueta: String,
                valor: String,
                tamano: f32,
                color: theme::Rgb,
                bold: bool| {
        let mut izq = TextSpec::new(etiqueta, tamano);
        let mut der = TextSpec::new(valor, tamano)
            .align(Align::Right)
            .color(color);
        if bold {
            izq = izq.bold();
            der = der.bold();
        }
        canvas.text_in(&izq, inner_x, inner_w / 2.0, y);
        canvas.text_in(&der, inner_x + inner_w / 2.0, inner_w / 2.0, y);
    };

    fila(
        canvas,
        y,
        ctx.t("Report.Liquidacion.Subtotal"),
        format_money(data.total_bruto, &ctx.locale),
        size::BODY_LIQUIDACION,
        theme::TEXT,
        false,
    );
    y += Canvas::line_height(size::BODY_LIQUIDACION);

    fila(
        canvas,
        y,
        ctx.t("Report.Liquidacion.Adelantos"),
        format!("- {}", format_money(data.total_adelantos, &ctx.locale)),
        size::BODY_LIQUIDACION,
        theme::NEGATIVE,
        false,
    );
    y += Canvas::line_height(size::BODY_LIQUIDACION) + 3.0;

    canvas.hline(inner_x, y, inner_w, theme::MUTED, 0.7);
    y += 3.0;

    let color = if data.total_neto.is_negative() {
        theme::NEGATIVE
    } else {
        theme::POSITIVE
    };
    fila(
        canvas,
        y,
        ctx.t("Report.Liquidacion.TotalAPagar"),
        format_money(data.total_neto, &ctx.locale),
        size::TOTAL,
        color,
        true,
    );

    canvas.set_cursor(top + alto);
}

pub fn observaciones(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
    let Some(texto) = data
        .observaciones
        .as_deref()
        .filter(|t| !t.trim().is_empty())
    else {
        return;
    };
    canvas.advance(20.0);
    let left = canvas.left();
    let width = canvas.content_width();
    canvas.text_in(
        &TextSpec::new(ctx.t("Report.Liquidacion.Observaciones"), size::BODY).bold(),
        left,
        width,
        canvas.cursor(),
    );
    canvas.advance(Canvas::line_height(size::BODY));
    canvas.text_in(
        &TextSpec::new(texto.to_owned(), size::BODY),
        left,
        width,
        canvas.cursor(),
    );
    canvas.advance(Canvas::line_height(size::BODY));
}

pub fn firmas(canvas: &mut Canvas, ctx: &ReportContext) {
    if !ctx.report.mostrar_firmas {
        return;
    }
    canvas.advance(60.0);
    canvas.ensure_space(50.0);

    let left = canvas.left();
    let width = canvas.content_width();
    let bloque = (width - 100.0) / 2.0;
    let y = canvas.cursor();

    for (index, clave) in [
        "Report.Liquidacion.FirmaRevision",
        "Report.Liquidacion.FirmaAdministracion",
    ]
    .iter()
    .enumerate()
    {
        let x = left + index as f32 * (bloque + 100.0);
        canvas.hline(x, y, bloque, theme::MUTED, 0.7);
        canvas.text_in(
            &TextSpec::new(ctx.t(clave), size::FOOTER)
                .align(Align::Center)
                .color(theme::MUTED),
            x,
            bloque,
            y + 6.0,
        );
    }
    canvas.advance(Canvas::line_height(size::FOOTER) + 6.0);
}
