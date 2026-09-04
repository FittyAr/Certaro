//! PDF of a progress certificate. See `docs/12-reportes-y-exportaciones.md` §4.

use certaro_application::dtos::certificados::CertificadoDetalle;
use certaro_application::dtos::reportes::GeneratedReport;
use certaro_application::result::AppResult;

use super::canvas::{Align, Canvas, TextSpec};
use super::theme::{self, size};
use crate::reporting::{filename, ReportContext};

mod header;
mod table;

#[cfg(test)]
mod tests;

pub fn generate(data: &CertificadoDetalle, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    // Landscape: it is what lets the nine columns fit.
    let mut canvas = Canvas::new(
        &ctx.t("Report.Certificado.Title"),
        theme::page::A4_HEIGHT,
        theme::page::A4_WIDTH,
        theme::page::MARGIN_CERTIFICADO,
    )?;

    header::encabezado(&mut canvas, data, ctx);
    table::tabla(&mut canvas, data, ctx);

    let pie = |actual: usize, total: usize| {
        Some(
            TextSpec::new(
                ctx.tp(
                    "Report.Certificado.Footer",
                    &[
                        ("empresa", &ctx.empresa.nombre),
                        ("actual", &actual.to_string()),
                        ("total", &total.to_string()),
                    ],
                ),
                size::FOOTER,
            )
            .color(theme::MUTED)
            .align(Align::Center),
        )
    };
    let bytes = canvas.finish(pie)?;

    Ok(GeneratedReport {
        bytes,
        registros: data.items.len() as u64,
        nombre_sugerido: filename::certificado(&data.proyecto_nombre, data.numero, data.fecha),
    })
}
