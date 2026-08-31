//! CSV of movements. See `docs/12-reportes-y-exportaciones.md` §2.4.
//!
//! Three decisions worth stating, all of them fixes to the legacy export:
//!
//! - **BOM**. Without it Excel on Windows opens the file in the local code page and every accent
//!   breaks. It is the single most reported defect of any CSV exported in Spanish.
//! - **CRLF, always**. The legacy export used the platform's line ending, so the same data
//!   produced different files depending on the machine that ran it.
//! - **ISO dates**. `dd/MM/yyyy` is read as month/day by a spreadsheet in another locale, which
//!   silently yields the wrong date rather than an error.

use eo_application::dtos::reportes::{GeneratedReport, ReporteMovimientos};
use eo_application::result::AppResult;

use super::movimientos::{cell_csv, columns, row, Layout};
use super::{filename, io_error, ReportContext};

pub const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

pub fn movimientos(data: &ReporteMovimientos, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    let cols = columns(Layout::Wide);

    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(Vec::new());

    writer
        .write_record(cols.iter().map(|c| ctx.t(c.key)))
        .map_err(|e| io_error("export.csv.header", e))?;

    for item in &data.items {
        let cells = row(item, Layout::Wide);
        writer
            .write_record(cells.iter().map(cell_csv))
            .map_err(|e| io_error("export.csv.row", e))?;
    }

    let body = writer
        .into_inner()
        .map_err(|e| io_error("export.csv.flush", e))?;

    let mut bytes = Vec::with_capacity(BOM.len() + body.len());
    bytes.extend_from_slice(BOM);
    bytes.extend_from_slice(&body);

    Ok(GeneratedReport {
        bytes,
        registros: data.items.len() as u64,
        nombre_sugerido: filename::movimientos(ctx.generado_en, filename::FormatoExport::Csv),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::tests_support::{contexto, movimiento, reporte};

    fn texto(data: &ReporteMovimientos) -> String {
        let generado = movimientos(data, &contexto()).unwrap();
        String::from_utf8(generado.bytes[BOM.len()..].to_vec()).unwrap()
    }

    #[test]
    fn csv_tiene_bom_y_crlf() {
        let generado =
            movimientos(&reporte(vec![movimiento("Cable", "10", "1")]), &contexto()).unwrap();
        assert_eq!(&generado.bytes[..3], BOM);
        let cuerpo = String::from_utf8(generado.bytes[3..].to_vec()).unwrap();
        assert!(cuerpo.contains("\r\n"), "no hay CRLF");
        assert!(!cuerpo.contains("\n\n"));
        for linea in cuerpo.split("\r\n").filter(|l| !l.is_empty()) {
            assert!(!linea.contains('\n'), "línea con LF suelto: {linea}");
        }
    }

    #[test]
    fn csv_escapa_comillas_y_comas() {
        let item = movimiento("Cable \"2,5\" mm, rojo", "10", "1");
        let cuerpo = texto(&reporte(vec![item]));
        assert!(
            cuerpo.contains("\"Cable \"\"2,5\"\" mm, rojo\""),
            "no se entrecomilló: {cuerpo}"
        );

        // Round trip: what a spreadsheet reads back has to be the original concept.
        let mut lector = csv::ReaderBuilder::new().from_reader(cuerpo.as_bytes());
        let fila = lector.records().next().unwrap().unwrap();
        assert_eq!(&fila[1], "Cable \"2,5\" mm, rojo");
    }

    #[test]
    fn csv_fechas_iso() {
        let cuerpo = texto(&reporte(vec![movimiento("Cable", "10", "1")]));
        let mut lector = csv::ReaderBuilder::new().from_reader(cuerpo.as_bytes());
        let fila = lector.records().next().unwrap().unwrap();
        assert_eq!(&fila[0], "2026-08-29");
    }

    #[test]
    fn csv_usa_punto_decimal_sin_separador_de_miles() {
        let cuerpo = texto(&reporte(vec![movimiento("Cable", "1234567.89", "1")]));
        assert!(cuerpo.contains("1234567.89"), "{cuerpo}");
    }

    #[test]
    fn csv_tiene_las_once_columnas_con_los_rotulos_traducidos() {
        let cuerpo = texto(&reporte(vec![]));
        let primera = cuerpo.lines().next().unwrap();
        assert_eq!(primera.split(',').count(), 11);
        assert!(primera.starts_with("Fecha,Concepto"), "{primera}");
    }

    #[test]
    fn reporte_vacio_genera_un_archivo_valido_con_solo_encabezado() {
        let generado = movimientos(&reporte(vec![]), &contexto()).unwrap();
        assert_eq!(generado.registros, 0);
        let cuerpo = String::from_utf8(generado.bytes[3..].to_vec()).unwrap();
        assert_eq!(cuerpo.trim_end_matches("\r\n").lines().count(), 1);
    }

    #[test]
    fn el_nombre_sugerido_lleva_la_extension_del_formato() {
        let generado = movimientos(&reporte(vec![]), &contexto()).unwrap();
        assert!(generado.nombre_sugerido.ends_with(".csv"));
    }
}
