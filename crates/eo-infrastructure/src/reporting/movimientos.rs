//! The columns of the movements report, shared by the five formats. See `docs/12` §2.
//!
//! One definition for every format on purpose. The legacy system had four columns in the PDF, six
//! in the spreadsheet and four different ones in the Word document, so the same report answered
//! three different questions depending on which button you pressed.
//!
//! **Deviation from doc 12 §2.2**: the `Unidad` and `Observaciones` columns are not produced. The
//! movement entity has no such fields (doc 05 §2.13), so the document asks for data that does not
//! exist. The wide layout therefore has 11 columns instead of 13.

use eo_application::dtos::movimientos::MovimientoListItem;
use eo_application::dtos::reportes::FiltroDescripcion;

use super::format::{format_date, format_date_iso, format_money_plain, format_number};
use super::ReportContext;

/// What a cell holds, so each format can render it natively: a spreadsheet needs the number as a
/// number, and a CSV needs it with a point as the decimal mark.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Text(String),
    /// A civil date, kept as such so the spreadsheet writes a real date cell.
    Date(chrono::NaiveDate),
    /// An amount, in units, with its four decimals preserved.
    Money(eo_domain::Money),
    Quantity(eo_domain::Decimal4),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    /// i18n key of the heading.
    pub key: &'static str,
    pub align: Align,
    /// Relative width used by the PDF and the Word document.
    pub width: u8,
    /// Whether the narrow layout (PDF, DOCX) includes it.
    pub narrow: bool,
}

/// Every column, in order. The narrow layouts take the seven flagged ones (doc 12 §2.1).
pub const COLUMNS: &[Column] = &[
    Column {
        key: "Report.Col.Fecha",
        align: Align::Left,
        width: 2,
        narrow: true,
    },
    Column {
        key: "Report.Col.Concepto",
        align: Align::Left,
        width: 5,
        narrow: true,
    },
    Column {
        key: "Report.Col.Tipo",
        align: Align::Left,
        width: 2,
        narrow: true,
    },
    Column {
        key: "Report.Col.Categoria",
        align: Align::Left,
        width: 2,
        narrow: true,
    },
    Column {
        key: "Report.Col.Cliente",
        align: Align::Left,
        width: 3,
        narrow: false,
    },
    Column {
        key: "Report.Col.Obra",
        align: Align::Left,
        width: 3,
        narrow: false,
    },
    Column {
        key: "Report.Col.Trabajo",
        align: Align::Left,
        width: 3,
        narrow: false,
    },
    Column {
        key: "Report.Col.Moneda",
        align: Align::Left,
        width: 1,
        narrow: false,
    },
    Column {
        key: "Report.Col.Monto",
        align: Align::Right,
        width: 2,
        narrow: true,
    },
    Column {
        key: "Report.Col.Cantidad",
        align: Align::Right,
        width: 1,
        narrow: true,
    },
    Column {
        key: "Report.Col.Total",
        align: Align::Right,
        width: 2,
        narrow: true,
    },
];

/// Which columns a layout uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// PDF and DOCX: seven columns that fit an A4 portrait page.
    Narrow,
    /// XLSX and CSV: everything, because a spreadsheet is scrolled, not printed.
    Wide,
}

#[must_use]
pub fn columns(layout: Layout) -> Vec<Column> {
    match layout {
        Layout::Narrow => COLUMNS.iter().copied().filter(|c| c.narrow).collect(),
        Layout::Wide => COLUMNS.to_vec(),
    }
}

/// The cells of one row, in the order of `columns(layout)`.
#[must_use]
pub fn row(item: &MovimientoListItem, layout: Layout) -> Vec<Cell> {
    let fecha = item.fecha.with_timezone(&chrono::Local).date_naive();
    let all = [
        Cell::Date(fecha),
        Cell::Text(item.concepto.clone()),
        Cell::Text(item.tipo_movimiento_nombre.clone()),
        Cell::Text(item.categoria_nombre.clone().unwrap_or_default()),
        Cell::Text(item.cliente_nombre.clone().unwrap_or_default()),
        Cell::Text(item.obra_nombre.clone().unwrap_or_default()),
        Cell::Text(item.trabajo_descripcion.clone().unwrap_or_default()),
        Cell::Text(item.moneda.iso().to_owned()),
        Cell::Money(item.monto),
        Cell::Quantity(item.cantidad),
        Cell::Money(item.total),
    ];

    all.into_iter()
        .zip(COLUMNS)
        .filter(|(_, column)| layout == Layout::Wide || column.narrow)
        .map(|(cell, _)| cell)
        .collect()
}

/// The cell as the human-readable text of the PDF and the Word document.
#[must_use]
pub fn cell_text(cell: &Cell, ctx: &ReportContext) -> String {
    match cell {
        Cell::Text(text) => text.clone(),
        Cell::Date(date) => format_date(*date, &ctx.locale),
        Cell::Money(value) => format_money_plain(*value, &ctx.locale),
        Cell::Quantity(value) => format_number(*value, &ctx.locale, 0),
    }
}

/// The cell as machine-readable text for the CSV: ISO dates and a point as the decimal mark, so
/// no spreadsheet reinterprets them by locale (doc 12 §2.4).
#[must_use]
pub fn cell_csv(cell: &Cell) -> String {
    match cell {
        Cell::Text(text) => text.clone(),
        Cell::Date(date) => format_date_iso(*date),
        // Two fixed decimals with a point: what a spreadsheet parses without guessing a locale.
        Cell::Money(value) => {
            let rounded = value.round_to(2).to_decimal_string();
            rounded
                .split_once('.')
                .map(|(whole, fraction)| format!("{whole}.{}", &fraction[..2]))
                .unwrap_or(rounded)
        }
        Cell::Quantity(value) => value.to_decimal_string(),
    }
}

/// The "filters applied" line, in prose. An exported PDF that does not say what it is showing is
/// not evidence of anything, which is what the legacy header amounted to.
#[must_use]
pub fn filtros_prosa(filtros: &[FiltroDescripcion], ctx: &ReportContext) -> String {
    if filtros.is_empty() {
        return ctx.t("Report.Filtro.Ninguno");
    }
    filtros
        .iter()
        .map(|f| ctx.tp(&f.clave, &[("valor", &f.valor)]))
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::tests_support::{contexto, movimiento};

    #[test]
    fn el_layout_angosto_tiene_las_siete_columnas_del_pdf() {
        assert_eq!(columns(Layout::Narrow).len(), 7);
    }

    #[test]
    fn el_layout_ancho_tiene_las_once_columnas_de_la_planilla() {
        assert_eq!(columns(Layout::Wide).len(), 11);
    }

    #[test]
    fn cada_fila_tiene_tantas_celdas_como_columnas_su_layout() {
        let item = movimiento("Cable 2.5", "1500.5", "2");
        for layout in [Layout::Narrow, Layout::Wide] {
            assert_eq!(row(&item, layout).len(), columns(layout).len());
        }
    }

    #[test]
    fn el_csv_usa_iso_y_punto_decimal() {
        let item = movimiento("Cable", "1500.5", "2");
        let celdas: Vec<String> = row(&item, Layout::Wide).iter().map(cell_csv).collect();
        assert!(celdas[0].contains('-'), "la fecha no es ISO: {}", celdas[0]);
        assert_eq!(celdas[8], "1500.50");
        assert_eq!(celdas[10], "3001.00");
    }

    #[test]
    fn sin_filtros_la_prosa_lo_dice_en_lugar_de_quedar_vacia() {
        let ctx = contexto();
        assert_eq!(filtros_prosa(&[], &ctx), ctx.t("Report.Filtro.Ninguno"));
    }

    #[test]
    fn los_filtros_se_encadenan_con_su_valor() {
        let ctx = contexto();
        let texto = filtros_prosa(
            &[
                FiltroDescripcion {
                    clave: "Report.Filtro.Cliente".to_owned(),
                    valor: "Acme".to_owned(),
                },
                FiltroDescripcion {
                    clave: "Report.Filtro.Tipo".to_owned(),
                    valor: "Gasto".to_owned(),
                },
            ],
            &ctx,
        );
        assert!(texto.contains("Acme") && texto.contains("Gasto"), "{texto}");
        assert!(!texto.contains('{'), "{texto}");
    }
}
