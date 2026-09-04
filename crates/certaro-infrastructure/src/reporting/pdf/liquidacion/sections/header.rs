use certaro_application::dtos::liquidaciones::LiquidacionDetalle;
use super::super::super::canvas::{Align, Canvas, TextSpec};
use super::super::super::theme::{self, size};
use crate::reporting::format::{format_date, format_datetime};
use crate::reporting::ReportContext;

pub fn encabezado(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
    let left = canvas.left();
    let width = canvas.content_width();
    let half = width / 2.0;
    let top = canvas.cursor();

    canvas.text_in(
        &TextSpec::new(ctx.t("Report.Liquidacion.Title"), size::TITLE_LIQUIDACION)
            .bold()
            .color(theme::PRIMARY),
        left,
        half,
        top,
    );
    let mut y = top + Canvas::line_height(size::TITLE_LIQUIDACION) + 2.0;

    canvas.text_in(
        &TextSpec::new(
            format!(
                "{} {}",
                ctx.t("Report.Liquidacion.Empleado"),
                data.empleado_nombre
            ),
            size::EMPLEADO,
        )
        .bold(),
        left,
        half,
        y,
    );
    y += Canvas::line_height(size::EMPLEADO);

    canvas.text_in(
        &TextSpec::new(
            ctx.tp(
                "Report.Liquidacion.Periodo",
                &[
                    ("desde", &format_date(data.fecha_inicio, &ctx.locale)),
                    ("hasta", &format_date(data.fecha_fin, &ctx.locale)),
                ],
            ),
            size::BODY,
        ),
        left,
        half,
        y,
    );
    y += Canvas::line_height(size::BODY);

    // Document and position: the legacy receipt named the employee and nothing else, which is not
    // enough to tell two people with the same name apart.
    let identidad: Vec<String> = [data.empleado_dni.clone(), data.empleado_cargo.clone()]
        .into_iter()
        .flatten()
        .collect();
    if !identidad.is_empty() {
        canvas.text_in(
            &TextSpec::new(identidad.join(" · "), 9.0).color(theme::MUTED),
            left,
            half,
            y,
        );
        y += Canvas::line_height(9.0);
    }

    // Right column: the company, from configuration.
    let right_x = left + half;
    let mut ry = top;
    canvas.text_in(
        &TextSpec::new(ctx.empresa.nombre.to_uppercase(), 12.0)
            .bold()
            .align(Align::Right),
        right_x,
        half,
        ry,
    );
    ry += Canvas::line_height(12.0);

    let lema = if ctx.empresa.lema.trim().is_empty() {
        ctx.t("Report.DefaultLema")
    } else {
        ctx.empresa.lema.clone()
    };
    canvas.text_in(
        &TextSpec::new(lema, size::BODY).italic().align(Align::Right),
        right_x,
        half,
        ry,
    );
    ry += Canvas::line_height(size::BODY);

    canvas.text_in(
        &TextSpec::new(format_datetime(ctx.generado_en, &ctx.locale), size::FOOTER)
            .color(theme::MUTED)
            .align(Align::Right),
        right_x,
        half,
        ry,
    );
    ry += Canvas::line_height(size::FOOTER);

    canvas.set_cursor(y.max(ry) + 16.0);
}

pub fn seccion(canvas: &mut Canvas, titulo: String, color: theme::Rgb) {
    canvas.ensure_space(60.0);
    let left = canvas.left();
    let width = canvas.content_width();
    let y = canvas.cursor();
    canvas.text_in(
        &TextSpec::new(titulo, size::SECTION).bold().color(color),
        left,
        width,
        y,
    );
    let bajo = y + Canvas::line_height(size::SECTION);
    canvas.hline(left, bajo, width, color, 0.7);
    canvas.set_cursor(bajo + 10.0);
}
