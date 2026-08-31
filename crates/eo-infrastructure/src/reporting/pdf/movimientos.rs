//! PDF of movements. See `docs/12-reportes-y-exportaciones.md` §2.1.

use eo_application::dtos::reportes::{GeneratedReport, ReporteMovimientos};
use eo_application::result::AppResult;

use super::canvas::{Align, Canvas, TextSpec};
use super::table::{Border, Cell, Row, Table, Width};
use super::theme::{self, size};
use crate::reporting::format::{format_datetime, format_money};
use crate::reporting::movimientos::{cell_text, columns, filtros_prosa, row as cells, Layout};
use crate::reporting::{filename, footer_text, ReportContext};

pub fn generate(data: &ReporteMovimientos, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    let mut canvas = Canvas::new(
        &ctx.t("Report.Movimientos.Title"),
        theme::page::A4_WIDTH,
        theme::page::A4_HEIGHT,
        theme::page::MARGIN_MOVIMIENTOS,
    )?;

    encabezado(&mut canvas, data, ctx);

    let cols = columns(Layout::Narrow);
    let mut table = Table::new(
        cols.iter()
            .map(|c| Width::Relative(f32::from(c.width)))
            .collect(),
        size::BODY,
    );
    table.zebra = Some(theme::ZEBRA);
    table.header = vec![Row::new(
        cols.iter()
            .map(|c| Cell::new(ctx.t(c.key)).bold().align(align_of(c.align)))
            .collect(),
    )
    .border_bottom(Border::new(theme::BLACK, 1.0))];

    table.rows = data
        .items
        .iter()
        .map(|item| {
            let values = cells(item, Layout::Narrow);
            Row::new(
                values
                    .iter()
                    .zip(&cols)
                    .enumerate()
                    .map(|(index, (cell, column))| {
                        let mut c = Cell::new(cell_text(cell, ctx)).align(align_of(column.align));
                        // The total is the number the reader is looking for, so it carries weight.
                        if index + 1 == cols.len() {
                            c = c.bold();
                        }
                        c
                    })
                    .collect(),
            )
            .border_bottom(Border::thin())
        })
        .collect();

    if data.items.is_empty() {
        table.rows = vec![Row::new(vec![Cell::new(ctx.t("Report.Movimientos.Vacio"))
            .colspan(cols.len())
            .italic()
            .color(theme::MUTED)
            .align(Align::Center)])];
    } else {
        table.footer = totales(data, ctx, cols.len());
    }

    table.render(&mut canvas);

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
        registros: data.items.len() as u64,
        nombre_sugerido: filename::movimientos(ctx.generado_en, filename::FormatoExport::Pdf),
    })
}

/// Title, filters in prose and record count. The legacy header was a single line that did not say
/// which filters had produced the list, so a printed copy was evidence of nothing.
fn encabezado(canvas: &mut Canvas, data: &ReporteMovimientos, ctx: &ReportContext) {
    let left = canvas.left();
    let width = canvas.content_width();

    let titulo = format!(
        "{} · {}",
        ctx.t("Report.Movimientos.Title"),
        ctx.empresa.nombre
    );
    canvas.text_in(
        &TextSpec::new(titulo, size::TITLE)
            .bold()
            .color(theme::PRIMARY),
        left,
        width,
        canvas.cursor(),
    );
    canvas.advance(Canvas::line_height(size::TITLE) + 2.0);

    canvas.text_in(
        &TextSpec::new(
            filtros_prosa(&data.filtros_descripcion, ctx),
            size::SUBTITLE,
        )
        .color(theme::MUTED),
        left,
        width,
        canvas.cursor(),
    );
    canvas.advance(Canvas::line_height(size::SUBTITLE));

    let registros = ctx.tp(
        "Report.Movimientos.Registros",
        &[("cantidad", &data.resumen.cantidad.to_string())],
    );
    canvas.text_in(
        &TextSpec::new(
            format!(
                "{registros} · {}",
                format_datetime(ctx.generado_en, &ctx.locale)
            ),
            size::SUBTITLE,
        )
        .color(theme::MUTED),
        left,
        width,
        canvas.cursor(),
    );
    canvas.advance(Canvas::line_height(size::SUBTITLE) + 10.0);
}

/// Income, expenses and balance. The legacy PDF totalled nothing, so the reader had to add the
/// column by hand to check it.
fn totales(data: &ReporteMovimientos, ctx: &ReportContext, columnas: usize) -> Vec<Row> {
    let balance_color = if data.resumen.balance.is_negative() {
        theme::NEGATIVE
    } else {
        theme::POSITIVE
    };

    let fila = |clave: &str, valor: String, color: theme::Rgb| {
        Row::new(vec![
            Cell::new(ctx.t(clave))
                .colspan(columnas - 1)
                .align(Align::Right)
                .bold(),
            Cell::new(valor).align(Align::Right).bold().color(color),
        ])
    };

    vec![
        fila(
            "Report.Total.Ingresos",
            format_money(data.resumen.total_ingresos, &ctx.locale),
            theme::TEXT,
        ),
        fila(
            "Report.Total.Gastos",
            format_money(data.resumen.total_gastos, &ctx.locale),
            theme::TEXT,
        ),
        fila(
            "Report.Total.Balance",
            format_money(data.resumen.balance, &ctx.locale),
            balance_color,
        )
        .fill(theme::TOTAL_FILL),
    ]
}

fn align_of(align: crate::reporting::movimientos::Align) -> Align {
    match align {
        crate::reporting::movimientos::Align::Left => Align::Left,
        crate::reporting::movimientos::Align::Right => Align::Right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::tests_support::pdf_text;
    use crate::reporting::tests_support::{contexto, filas, movimiento, reporte};

    #[test]
    fn pdf_movimientos_no_falla_con_cero_filas() {
        let generado = generate(&reporte(vec![]), &contexto()).unwrap();
        assert!(generado.bytes.starts_with(b"%PDF"));
        assert_eq!(generado.registros, 0);
        let texto = pdf_text(&generado.bytes);
        assert!(
            texto.contains("No hay movimientos"),
            "no muestra el mensaje de vacío: {texto}"
        );
    }

    #[test]
    fn pdf_movimientos_muestra_los_valores_y_los_totales() {
        let generado = generate(
            &reporte(vec![movimiento("Cable 2.5", "1500.5", "2")]),
            &contexto(),
        )
        .unwrap();
        let texto = pdf_text(&generado.bytes);
        assert!(texto.contains("Cable 2.5"), "{texto}");
        assert!(
            texto.contains("3.001,00"),
            "falta el total de la fila: {texto}"
        );
        assert!(texto.contains("Total de gastos"), "{texto}");
    }

    #[test]
    fn pdf_movimientos_dice_que_filtros_aplico() {
        let mut data = reporte(vec![movimiento("Cable", "10", "1")]);
        data.filtros_descripcion = vec![eo_application::dtos::reportes::FiltroDescripcion {
            clave: "Report.Filtro.Cliente".to_owned(),
            valor: "Acme".to_owned(),
        }];
        let texto = pdf_text(&generate(&data, &contexto()).unwrap().bytes);
        assert!(texto.contains("Acme"), "{texto}");
    }

    #[test]
    fn pdf_movimientos_pagina_correctamente() {
        let generado = generate(&reporte(filas(500)), &contexto()).unwrap();
        let texto = pdf_text(&generado.bytes);
        // The heading is repeated once per page, so more than one occurrence means it paginated
        // and carried the headings along.
        let veces = texto.matches("Concepto").count();
        assert!(veces > 1, "el encabezado no se repitió: {veces}");
        assert_eq!(generado.registros, 500);
    }

    #[test]
    fn pdf_movimientos_usa_el_nombre_de_empresa_de_configuracion() {
        let texto = pdf_text(&generate(&reporte(vec![]), &contexto()).unwrap().bytes);
        assert!(texto.contains("GENERCON"), "{texto}");
    }
}
