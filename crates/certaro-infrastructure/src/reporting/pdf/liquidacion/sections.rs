use certaro_application::dtos::liquidaciones::LiquidacionDetalle;
use certaro_domain::Money;
use super::super::canvas::{Align, Canvas, TextSpec};
use super::super::table::{Border, Cell, Row, Table, Width};
use super::super::theme::{self, size};
use crate::reporting::format::{format_date, format_datetime, format_money, format_number};
use crate::reporting::ReportContext;

/// Width of the totals box, per doc 12 §3.4.
const TOTALES_WIDTH: f32 = 200.0;

pub(super) fn encabezado(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
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

pub(super) fn seccion(canvas: &mut Canvas, titulo: String, color: theme::Rgb) {
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

pub(super) fn resumen(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
    seccion(
        canvas,
        ctx.t("Report.Liquidacion.SectionResumen"),
        theme::TEXT,
    );

    let mut table = Table::new(
        vec![
            Width::Relative(3.0),
            Width::Relative(1.0),
            Width::Relative(2.0),
            Width::Relative(2.0),
        ],
        size::BODY_LIQUIDACION,
    );
    table.header = vec![Row::new(vec![
        Cell::new(ctx.t("Report.Col.Concepto")).bold(),
        Cell::new(ctx.t("Report.Col.Dias"))
            .bold()
            .align(Align::Right),
        Cell::new(ctx.t("Report.Col.Tarifa"))
            .bold()
            .align(Align::Right),
        Cell::new(ctx.t("Report.Col.Subtotal"))
            .bold()
            .align(Align::Right),
    ])
    .border_bottom(Border::new(theme::BLACK, 1.0))];

    let base = base_bruto(data);
    table.rows.push(
        Row::new(vec![
            Cell::new(ctx.t("Report.Liquidacion.DiasTrabajados")),
            Cell::new(format_number(data.dias_trabajados, &ctx.locale, 1)).align(Align::Right),
            Cell::new(format_money(data.tarifa_aplicada, &ctx.locale)).align(Align::Right),
            Cell::new(format_money(base, &ctx.locale))
                .align(Align::Right)
                .bold(),
        ])
        .border_bottom(Border::thin()),
    );

    let recargos = data.total_bruto.checked_sub(base).unwrap_or(Money::ZERO);
    if !recargos.is_zero() {
        table.rows.push(
            Row::new(vec![
                Cell::new(ctx.t("Report.Liquidacion.Recargos")),
                Cell::new(multiplicadores(data, ctx)).align(Align::Right),
                Cell::empty(),
                Cell::new(format_money(recargos, &ctx.locale)).align(Align::Right),
            ])
            .border_bottom(Border::thin()),
        );
    }

    table.render(canvas);
    canvas.advance(20.0);
}

/// The multipliers actually applied, so the premium line can be checked against the rules.
pub(super) fn multiplicadores(data: &LiquidacionDetalle, ctx: &ReportContext) -> String {
    let mut partes = Vec::new();
    if data.incluir_sabados {
        partes.push(format_number(data.multiplicador_sabado, &ctx.locale, 1));
    }
    if data.incluir_domingos {
        partes.push(format_number(data.multiplicador_domingo, &ctx.locale, 1));
    }
    if data.incluir_feriados {
        partes.push(format_number(data.multiplicador_feriado, &ctx.locale, 1));
    }
    partes.join(" / ")
}

/// Days × rate, which is what the gross would be with no premiums.
pub(super) fn base_bruto(data: &LiquidacionDetalle) -> Money {
    data.tarifa_aplicada
        .checked_mul(data.dias_trabajados)
        .unwrap_or(data.total_bruto)
}

pub(super) fn adelantos(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
    seccion(
        canvas,
        ctx.t("Report.Liquidacion.SectionAdelantos"),
        theme::NEGATIVE,
    );

    let mut table = Table::new(
        vec![
            Width::Relative(2.0),
            Width::Relative(6.0),
            Width::Relative(2.0),
        ],
        size::BODY_LIQUIDACION,
    );
    table.header = vec![Row::new(vec![
        Cell::new(ctx.t("Report.Col.Fecha")).bold(),
        Cell::new(ctx.t("Report.Col.Concepto")).bold(),
        Cell::new(ctx.t("Report.Col.Monto"))
            .bold()
            .align(Align::Right),
    ])
    .border_bottom(Border::new(theme::BLACK, 1.0))];

    if data.adelantos.is_empty() {
        table.rows.push(Row::new(vec![Cell::new(
            ctx.t("Report.Liquidacion.SinAdelantos"),
        )
        .colspan(3)
        .italic()
        .color(theme::MUTED)
        .align(Align::Center)]));
    } else {
        // Every advance on its own line with its own date. Grouping or rounding them is precisely
        // what the client objected to.
        for adelanto in &data.adelantos {
            table.rows.push(
                Row::new(vec![
                    Cell::new(format_date(adelanto.fecha, &ctx.locale)),
                    Cell::new(adelanto.concepto.clone()),
                    Cell::new(format_money(adelanto.monto, &ctx.locale)).align(Align::Right),
                ])
                .border_bottom(Border::thin()),
            );
        }
        table.footer = vec![Row::new(vec![
            Cell::new(ctx.t("Report.Liquidacion.TotalAdelantos"))
                .colspan(2)
                .align(Align::Right)
                .bold(),
            Cell::new(format_money(data.total_adelantos, &ctx.locale))
                .align(Align::Right)
                .bold()
                .color(theme::NEGATIVE),
        ])];
    }

    table.render(canvas);
}

pub(super) fn totales(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
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

    // A negative net is a real case: the employee drew more than they earned. The legacy receipt
    // painted the total green regardless, colouring a shortfall as good news.
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

pub(super) fn observaciones(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
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

pub(super) fn firmas(canvas: &mut Canvas, ctx: &ReportContext) {
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

