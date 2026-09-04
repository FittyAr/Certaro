use certaro_application::dtos::liquidaciones::LiquidacionDetalle;
use certaro_domain::Money;
use super::super::super::canvas::{Align, Canvas};
use super::super::super::table::{Border, Cell, Row, Table, Width};
use super::super::super::theme::{self, size};
use crate::reporting::format::{format_date, format_money, format_number};
use crate::reporting::ReportContext;
use super::header::seccion;

pub fn resumen(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
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

pub fn multiplicadores(data: &LiquidacionDetalle, ctx: &ReportContext) -> String {
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

pub fn base_bruto(data: &LiquidacionDetalle) -> Money {
    data.tarifa_aplicada
        .checked_mul(data.dias_trabajados)
        .unwrap_or(data.total_bruto)
}

pub fn adelantos(canvas: &mut Canvas, data: &LiquidacionDetalle, ctx: &ReportContext) {
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
