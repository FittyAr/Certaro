//! DOCX of movements. See `docs/12-reportes-y-exportaciones.md` §2.3.
//!
//! Same seven columns and same formatting as the PDF. That equality is the point: the legacy Word
//! export had four columns, the PDF had four different ones and the spreadsheet six, so the same
//! report answered a different question depending on the button pressed.

use std::io::Cursor;

use docx_rs::{
    AlignmentType, Docx, Paragraph, Run, Shading, Table, TableCell, TableCellBorders, TableRow,
    VAlignType, WidthType,
};
use eo_application::dtos::reportes::{GeneratedReport, ReporteMovimientos};
use eo_application::result::AppResult;

use super::format::{format_datetime, format_money};
use super::movimientos::{cell_text, columns, filtros_prosa, row as cells, Align, Layout};
use super::pdf::theme::{self, Rgb};
use super::{filename, io_error, ReportContext};

/// A4 portrait in twips (1/1440 inch), and 2 cm margins, per doc 12 §2.3.
const PAGE_WIDTH: u32 = 11_906;
const PAGE_HEIGHT: u32 = 16_838;
const MARGIN: u32 = 1_134;

/// Half-points, which is how Word measures font size.
const fn half_points(size: usize) -> usize {
    size * 2
}

pub fn movimientos(data: &ReporteMovimientos, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    let cols = columns(Layout::Narrow);
    let content_width = PAGE_WIDTH - 2 * MARGIN;
    let total_share: f32 = cols.iter().map(|c| f32::from(c.width)).sum();
    let widths: Vec<usize> = cols
        .iter()
        .map(|c| (content_width as f32 * f32::from(c.width) / total_share) as usize)
        .collect();

    let mut docx = Docx::new()
        .page_size(PAGE_WIDTH, PAGE_HEIGHT)
        .page_margin(
            docx_rs::PageMargin::new()
                .top(MARGIN as i32)
                .bottom(MARGIN as i32)
                .left(MARGIN as i32)
                .right(MARGIN as i32),
        )
        .add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(
                    Run::new()
                        .add_text(format!(
                            "{} · {}",
                            ctx.t("Report.Movimientos.Title"),
                            ctx.empresa.nombre
                        ))
                        .bold()
                        .size(half_points(20)),
                ),
        )
        .add_paragraph(
            Paragraph::new().align(AlignmentType::Center).add_run(
                Run::new()
                    .add_text(filtros_prosa(&data.filtros_descripcion, ctx))
                    .size(half_points(10))
                    .color(hex(theme::MUTED)),
            ),
        )
        .add_paragraph(
            Paragraph::new().align(AlignmentType::Center).add_run(
                Run::new()
                    .add_text(format!(
                        "{} · {}",
                        ctx.tp(
                            "Report.Movimientos.Registros",
                            &[("cantidad", &data.resumen.cantidad.to_string())]
                        ),
                        format_datetime(ctx.generado_en, &ctx.locale)
                    ))
                    .size(half_points(8))
                    .color(hex(theme::MUTED)),
            ),
        );

    let mut tabla = Table::new(vec![encabezado(ctx, &cols, &widths)])
        .width(content_width as usize, WidthType::Dxa);

    if data.items.is_empty() {
        tabla = tabla.add_row(TableRow::new(vec![celda(
            ctx.t("Report.Movimientos.Vacio"),
            content_width as usize,
            Align::Left,
            false,
            None,
        )
        .grid_span(cols.len())]));
    } else {
        for item in &data.items {
            let valores = cells(item, Layout::Narrow);
            let celdas = valores
                .iter()
                .zip(&cols)
                .enumerate()
                .map(|(index, (cell, column))| {
                    celda(
                        cell_text(cell, ctx),
                        widths[index],
                        column.align,
                        index + 1 == cols.len(),
                        None,
                    )
                })
                .collect();
            tabla = tabla.add_row(TableRow::new(celdas));
        }
    }

    docx = docx.add_table(tabla);

    for (clave, valor) in [
        ("Report.Total.Ingresos", data.resumen.total_ingresos),
        ("Report.Total.Gastos", data.resumen.total_gastos),
        ("Report.Total.Balance", data.resumen.balance),
    ] {
        let color = if clave == "Report.Total.Balance" {
            if data.resumen.balance.is_negative() {
                theme::NEGATIVE
            } else {
                theme::POSITIVE
            }
        } else {
            theme::TEXT
        };
        docx = docx.add_paragraph(
            Paragraph::new().align(AlignmentType::Right).add_run(
                Run::new()
                    .add_text(format!(
                        "{}: {}",
                        ctx.t(clave),
                        format_money(valor, &ctx.locale)
                    ))
                    .bold()
                    .size(half_points(10))
                    .color(hex(color)),
            ),
        );
    }

    let mut buffer = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut buffer)
        .map_err(|e| io_error("export.docx.pack", e))?;

    Ok(GeneratedReport {
        bytes: buffer.into_inner(),
        registros: data.items.len() as u64,
        nombre_sugerido: filename::movimientos(ctx.generado_en, filename::FormatoExport::Docx),
    })
}

fn encabezado(
    ctx: &ReportContext,
    cols: &[super::movimientos::Column],
    widths: &[usize],
) -> TableRow {
    let celdas = cols
        .iter()
        .enumerate()
        .map(|(index, column)| {
            celda(
                ctx.t(column.key),
                widths[index],
                column.align,
                true,
                Some(theme::ZEBRA),
            )
        })
        .collect();
    // Word repeats a row flagged as a header on every page, which is what the legacy export lacked.
    TableRow::new(celdas).cant_split()
}

fn celda(
    texto: String,
    width: usize,
    align: Align,
    bold: bool,
    fill: Option<Rgb>,
) -> TableCell {
    let mut run = Run::new().add_text(texto).size(half_points(10));
    if bold {
        run = run.bold();
    }
    let parrafo = Paragraph::new().add_run(run).align(match align {
        Align::Left => AlignmentType::Left,
        Align::Right => AlignmentType::Right,
    });

    let mut cell = TableCell::new()
        .add_paragraph(parrafo)
        .width(width, WidthType::Dxa)
        .vertical_align(VAlignType::Center)
        .set_borders(TableCellBorders::new());
    if let Some(color) = fill {
        cell = cell.shading(Shading::new().fill(hex(color)));
    }
    cell
}

/// A theme colour as the `RRGGBB` Word expects. The hex only ever comes from the theme, so no
/// colour is written twice.
fn hex(color: Rgb) -> String {
    let channel = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    format!(
        "{:02X}{:02X}{:02X}",
        channel(color.0),
        channel(color.1),
        channel(color.2)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::tests_support::{contexto, docx_text, movimiento, reporte};

    #[test]
    fn el_docx_tiene_las_mismas_siete_columnas_del_pdf() {
        let generado =
            movimientos(&reporte(vec![movimiento("Cable", "10", "1")]), &contexto()).unwrap();
        let texto = docx_text(&generado.bytes);
        for rotulo in ["Fecha", "Concepto", "Tipo", "Categoría", "Monto", "Cantidad", "Total"] {
            assert!(texto.contains(rotulo), "falta el rótulo {rotulo}");
        }
        assert!(!texto.contains("Cliente"), "el layout angosto no lleva Cliente");
    }

    #[test]
    fn el_docx_muestra_los_datos_y_los_totales() {
        let generado = movimientos(
            &reporte(vec![movimiento("Cable 2.5", "1500.5", "2")]),
            &contexto(),
        )
        .unwrap();
        let texto = docx_text(&generado.bytes);
        assert!(texto.contains("Cable 2.5"), "{texto}");
        assert!(texto.contains("3.001,00"), "{texto}");
        assert!(texto.contains("Balance"), "{texto}");
    }

    #[test]
    fn reporte_vacio_genera_un_docx_valido() {
        let generado = movimientos(&reporte(vec![]), &contexto()).unwrap();
        assert!(generado.bytes.starts_with(b"PK"));
        assert!(docx_text(&generado.bytes).contains("No hay movimientos"));
    }

    #[test]
    fn el_color_sale_del_tema_en_formato_de_word() {
        assert_eq!(hex(theme::WHITE), "FFFFFF");
        assert_eq!(hex(theme::BLACK), "000000");
    }
}
