use certaro_application::dtos::certificados::CertificadoDetalle;

use super::super::canvas::{Align, Canvas, TextSpec};
use super::super::theme::{self, size};
use crate::reporting::format::format_date;
use crate::reporting::ReportContext;

pub(super) fn encabezado(canvas: &mut Canvas, data: &CertificadoDetalle, ctx: &ReportContext) {
    let left = canvas.left();
    let width = canvas.content_width();
    let izquierda = width * 3.0 / 5.0;
    let derecha = width - izquierda;
    let padding = 5.0;
    let linea = Canvas::line_height(size::BODY_CERTIFICADO);
    let alto = padding * 2.0 + linea * 4.0;
    let top = canvas.cursor();

    canvas.rect(left, top, width, alto, None, Some((theme::BLACK, 1.0)));
    canvas.vline(left + izquierda, top, alto, theme::BLACK, 1.0);

    let proyecto = if data.proyecto_nombre.trim().is_empty() {
        ctx.t("Report.Certificado.ProyectoGeneral")
    } else {
        data.proyecto_nombre.clone()
    };

    let filas = [
        ("Report.Certificado.Proyecto", proyecto),
        ("Report.Certificado.Ref", data.orden_titulo.clone()),
        (
            "Report.Certificado.Contratista",
            ctx.empresa.contratista.clone(),
        ),
        ("Report.Certificado.Cliente", data.cliente_nombre.clone()),
    ];

    let rotulo_ancho = 62.0;
    let mut y = top + padding;
    for (clave, valor) in filas {
        canvas.text_in(
            &TextSpec::new(ctx.t(clave), size::BODY_CERTIFICADO).bold(),
            left + padding,
            rotulo_ancho,
            y,
        );
        canvas.text_in(
            &TextSpec::new(valor, size::BODY_CERTIFICADO),
            left + padding + rotulo_ancho,
            izquierda - 2.0 * padding - rotulo_ancho,
            y,
        );
        y += linea;
    }

    // The right box: the brand, from configuration. No image is drawn even when a logo path is
    // set, so a missing or unreadable file can never break the document; the trade name is shown
    // instead, which is what the legacy version did with its hardcoded string.
    let rx = left + izquierda + padding;
    let rw = derecha - 2.0 * padding;
    let mut ry = top + padding;
    canvas.text_in(
        &TextSpec::new(ctx.empresa.nombre.to_uppercase(), 16.0)
            .bold()
            .align(Align::Center),
        rx,
        rw,
        ry,
    );
    ry += Canvas::line_height(16.0);
    if !ctx.empresa.lema.trim().is_empty() {
        canvas.text_in(
            &TextSpec::new(ctx.empresa.lema.clone(), size::CERT_HEADER).align(Align::Center),
            rx,
            rw,
            ry,
        );
    }
    ry += Canvas::line_height(size::CERT_HEADER) + 5.0;

    for (clave, valor) in [
        (
            "Report.Certificado.Fecha",
            format_date(data.fecha, &ctx.locale),
        ),
        ("Report.Certificado.Numero", data.numero.to_string()),
    ] {
        canvas.text_in(
            &TextSpec::new(ctx.t(clave), size::BODY_CERTIFICADO).bold(),
            rx,
            rw / 2.0,
            ry,
        );
        canvas.text_in(
            &TextSpec::new(valor, size::BODY_CERTIFICADO).align(Align::Right),
            rx + rw / 2.0,
            rw / 2.0,
            ry,
        );
        ry += linea;
    }

    canvas.set_cursor(top + alto.max(ry - top) + 12.0);
}
