//! XLSX of movements. See `docs/12-reportes-y-exportaciones.md` §2.2.
//!
//! Every monetary column declares its number format. The legacy export wrote bare numbers and let
//! Excel decide how many decimals to show, so the same file looked different on two machines and
//! neither matched the screen.

use std::collections::BTreeMap;

use eo_application::dtos::reportes::{GeneratedReport, ReporteMovimientos};
use eo_application::result::AppResult;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};

use super::movimientos::{columns, filtros_prosa, row, Cell, Layout};
use super::{filename, io_error, ReportContext};

/// Where the data starts. Row 1 is the title, 2 the filters, 3 blank and 4 the headings.
const HEADER_ROW: u32 = 3;
const FIRST_DATA_ROW: u32 = 4;

const MONEY_FORMAT: &str = "#,##0.00";
const QUANTITY_FORMAT: &str = "#,##0.####";
const DATE_FORMAT: &str = "dd/mm/yyyy";

const MIN_WIDTH: f64 = 10.0;
const MAX_WIDTH: f64 = 60.0;

pub fn movimientos(data: &ReporteMovimientos, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    let cols = columns(Layout::Wide);
    let mut workbook = Workbook::new();

    {
        let sheet = workbook.add_worksheet();
        sheet
            .set_name(ctx.t("Report.Sheet.Movimientos"))
            .map_err(|e| io_error("export.xlsx.sheet", e))?;
        write_movimientos(sheet, data, ctx, &cols)?;
    }
    {
        let sheet = workbook.add_worksheet();
        sheet
            .set_name(ctx.t("Report.Sheet.Resumen"))
            .map_err(|e| io_error("export.xlsx.resumen", e))?;
        write_resumen(sheet, data, ctx)?;
    }

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| io_error("export.xlsx.save", e))?;

    Ok(GeneratedReport {
        bytes,
        registros: data.items.len() as u64,
        nombre_sugerido: filename::movimientos(ctx.generado_en, filename::FormatoExport::Xlsx),
    })
}

fn write_movimientos(
    sheet: &mut Worksheet,
    data: &ReporteMovimientos,
    ctx: &ReportContext,
    cols: &[super::movimientos::Column],
) -> AppResult<()> {
    let titulo = Format::new().set_bold().set_font_size(14);
    let subtitulo = Format::new()
        .set_font_size(9)
        .set_font_color(Color::Gray)
        .set_italic();
    let encabezado = Format::new()
        .set_bold()
        .set_border_bottom(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    let money = Format::new().set_num_format(MONEY_FORMAT);
    let money_bold = Format::new().set_num_format(MONEY_FORMAT).set_bold();
    let quantity = Format::new().set_num_format(QUANTITY_FORMAT);
    let date = Format::new().set_num_format(DATE_FORMAT);

    let last_col = (cols.len() - 1) as u16;

    if last_col > 0 {
        sheet
            .merge_range(
                0,
                0,
                0,
                last_col,
                &format!("{} · {}", ctx.t("Report.Movimientos.Title"), ctx.empresa.nombre),
                &titulo,
            )
            .map_err(|e| io_error("export.xlsx.title", e))?;
        sheet
            .merge_range(
                1,
                0,
                1,
                last_col,
                &filtros_prosa(&data.filtros_descripcion, ctx),
                &subtitulo,
            )
            .map_err(|e| io_error("export.xlsx.filters", e))?;
    }

    // Tracks the widest text of each column, so the widths follow the content and not a guess.
    let mut widths: BTreeMap<u16, f64> = BTreeMap::new();

    for (index, column) in cols.iter().enumerate() {
        let index = index as u16;
        let text = ctx.t(column.key);
        widths.insert(index, text.chars().count() as f64 + 2.0);
        sheet
            .write_string_with_format(HEADER_ROW, index, &text, &encabezado)
            .map_err(|e| io_error("export.xlsx.header", e))?;
    }

    for (offset, item) in data.items.iter().enumerate() {
        let excel_row = FIRST_DATA_ROW + offset as u32;
        let is_last_column = |i: usize| i + 1 == cols.len();

        for (index, cell) in row(item, Layout::Wide).into_iter().enumerate() {
            let col = index as u16;
            let width_hint = match &cell {
                Cell::Text(text) => text.chars().count() as f64 + 2.0,
                Cell::Date(_) => 12.0,
                Cell::Money(_) | Cell::Quantity(_) => 14.0,
            };
            let entry = widths.entry(col).or_insert(MIN_WIDTH);
            *entry = entry.max(width_hint);

            match cell {
                Cell::Text(text) => sheet
                    .write_string(excel_row, col, &text)
                    .map(|_| ())
                    .map_err(|e| io_error("export.xlsx.text", e)),
                Cell::Date(value) => sheet
                    .write_date_with_format(excel_row, col, &value, &date)
                    .map(|_| ())
                    .map_err(|e| io_error("export.xlsx.date", e)),
                Cell::Money(value) => {
                    // The spreadsheet gets a real number so the user can go on calculating with
                    // it; the four stored decimals are kept and only the display is rounded.
                    let formato = if is_last_column(index) {
                        &money_bold
                    } else {
                        &money
                    };
                    sheet
                        .write_number_with_format(excel_row, col, as_f64(value.raw()), formato)
                        .map(|_| ())
                        .map_err(|e| io_error("export.xlsx.money", e))
                }
                Cell::Quantity(value) => sheet
                    .write_number_with_format(excel_row, col, as_f64(value.raw()), &quantity)
                    .map(|_| ())
                    .map_err(|e| io_error("export.xlsx.quantity", e)),
            }?;
        }
    }

    // Totals with SUBTOTAL(109) rather than SUM: that way the total answers the autofilter, which
    // is the whole reason a spreadsheet is asked for instead of a PDF.
    if !data.items.is_empty() {
        let total_row = FIRST_DATA_ROW + data.items.len() as u32;
        for (index, column) in cols.iter().enumerate() {
            if column.key != "Report.Col.Monto" && column.key != "Report.Col.Total" {
                continue;
            }
            let col = index as u16;
            let letra = column_letter(col);
            let formula = format!(
                "=SUBTOTAL(109,{letra}{}:{letra}{})",
                FIRST_DATA_ROW + 1,
                total_row
            );
            sheet
                .write_formula_with_format(
                    total_row,
                    col,
                    formula.as_str(),
                    &money_bold,
                )
                .map_err(|e| io_error("export.xlsx.total", e))?;
        }
    }

    for (col, width) in widths {
        sheet
            .set_column_width(col, width.clamp(MIN_WIDTH, MAX_WIDTH))
            .map_err(|e| io_error("export.xlsx.width", e))?;
    }

    sheet
        .set_freeze_panes(FIRST_DATA_ROW, 0)
        .map_err(|e| io_error("export.xlsx.freeze", e))?;

    let last_row = FIRST_DATA_ROW + data.items.len().max(1) as u32 - 1;
    sheet
        .autofilter(HEADER_ROW, 0, last_row, last_col)
        .map_err(|e| io_error("export.xlsx.autofilter", e))?;

    Ok(())
}

fn write_resumen(
    sheet: &mut Worksheet,
    data: &ReporteMovimientos,
    ctx: &ReportContext,
) -> AppResult<()> {
    let etiqueta = Format::new().set_bold();
    let money = Format::new().set_num_format(MONEY_FORMAT);

    let filas = [
        ("Report.Total.Ingresos", data.resumen.total_ingresos),
        ("Report.Total.Gastos", data.resumen.total_gastos),
        ("Report.Total.Balance", data.resumen.balance),
    ];

    for (index, (clave, valor)) in filas.iter().enumerate() {
        let fila = index as u32;
        sheet
            .write_string_with_format(fila, 0, ctx.t(clave), &etiqueta)
            .map_err(|e| io_error("export.xlsx.resumen.label", e))?;
        sheet
            .write_number_with_format(fila, 1, as_f64(valor.raw()), &money)
            .map_err(|e| io_error("export.xlsx.resumen.value", e))?;
    }

    sheet
        .write_string_with_format(3, 0, ctx.t("Report.Movimientos.Registros"), &etiqueta)
        .map_err(|e| io_error("export.xlsx.resumen.count.label", e))?;
    sheet
        .write_number(3, 1, data.resumen.cantidad as f64)
        .map_err(|e| io_error("export.xlsx.resumen.count", e))?;

    sheet
        .set_column_width(0, 28.0)
        .map_err(|e| io_error("export.xlsx.resumen.width", e))?;
    sheet
        .set_column_width(1, 18.0)
        .map_err(|e| io_error("export.xlsx.resumen.width", e))?;
    Ok(())
}

/// A scaled integer as the decimal number the spreadsheet holds. `f64` is exact for any amount up
/// to 2^53 / 10 000, which is far past anything this system records, and a spreadsheet has no
/// other numeric type to offer.
fn as_f64(raw: i64) -> f64 {
    raw as f64 / 10_000.0
}

/// `A`, `B`, … `AA`. Only used to build the SUBTOTAL range.
fn column_letter(mut index: u16) -> String {
    let mut letters = Vec::new();
    loop {
        letters.push(b'A' + (index % 26) as u8);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    letters.reverse();
    String::from_utf8_lossy(&letters).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::tests_support::{contexto, filas, movimiento, reporte};

    #[test]
    fn las_letras_de_columna_siguen_la_convencion_de_excel() {
        assert_eq!(column_letter(0), "A");
        assert_eq!(column_letter(10), "K");
        assert_eq!(column_letter(25), "Z");
        assert_eq!(column_letter(26), "AA");
    }

    #[test]
    fn el_escalado_a_decimal_no_pierde_centavos() {
        assert_eq!(as_f64(15_005_000), 1500.5);
        assert_eq!(as_f64(-2_407_500), -240.75);
    }

    #[test]
    fn el_libro_se_genera_con_dos_hojas() {
        let generado =
            movimientos(&reporte(vec![movimiento("Cable", "10", "2")]), &contexto()).unwrap();
        assert!(generado.bytes.starts_with(b"PK"), "no es un zip");
        assert_eq!(generado.registros, 1);
        assert!(generado.nombre_sugerido.ends_with(".xlsx"));
    }

    #[test]
    fn reporte_vacio_genera_un_libro_valido() {
        let generado = movimientos(&reporte(vec![]), &contexto()).unwrap();
        assert!(generado.bytes.starts_with(b"PK"));
        assert_eq!(generado.registros, 0);
    }

    #[test]
    fn xlsx_congela_en_a5_y_tiene_autofiltro() {
        let generado =
            movimientos(&reporte(vec![movimiento("Cable", "10", "2")]), &contexto()).unwrap();
        let hoja = crate::reporting::tests_support::zip_entry(
            &generado.bytes,
            "xl/worksheets/sheet1.xml",
        );
        assert!(
            hoja.contains("ySplit=\"4\"") && hoja.contains("topLeftCell=\"A5\""),
            "no congeló en A5: {hoja}"
        );
        assert!(hoja.contains("<autoFilter"), "no hay autofiltro");
    }

    #[test]
    fn xlsx_formatos_numericos_por_columna() {
        let generado =
            movimientos(&reporte(vec![movimiento("Cable", "10", "2")]), &contexto()).unwrap();
        let estilos =
            crate::reporting::tests_support::zip_entry(&generado.bytes, "xl/styles.xml");
        for formato in [MONEY_FORMAT, QUANTITY_FORMAT, DATE_FORMAT] {
            let escapado = formato.replace('#', "#");
            assert!(
                estilos.contains(&escapado),
                "falta el formato {formato}: {estilos}"
            );
        }
    }

    #[test]
    fn xlsx_totaliza_con_subtotal_para_que_responda_al_autofiltro() {
        let generado =
            movimientos(&reporte(vec![movimiento("Cable", "10", "2")]), &contexto()).unwrap();
        let hoja = crate::reporting::tests_support::zip_entry(
            &generado.bytes,
            "xl/worksheets/sheet1.xml",
        );
        assert!(hoja.contains("SUBTOTAL(109"), "no usa SUBTOTAL: {hoja}");
    }

    #[test]
    fn cinco_mil_filas_se_generan_en_menos_de_diez_segundos() {
        let inicio = std::time::Instant::now();
        let generado = movimientos(&reporte(filas(5_000)), &contexto()).unwrap();
        let transcurrido = inicio.elapsed();
        assert_eq!(generado.registros, 5_000);
        assert!(
            transcurrido < std::time::Duration::from_secs(10),
            "tardó {transcurrido:?}"
        );
    }
}
