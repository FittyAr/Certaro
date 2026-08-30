//! PDF of a progress certificate. See `docs/12-reportes-y-exportaciones.md` §4.
//!
//! This reproduces a paper form the client already uses, so the nine columns and the order of the
//! closing rows are not open to interpretation. What does change from the legacy version is that
//! the contractor, the trade name, the tagline and the logo come from configuration: they used to
//! be the literals `"PABLO BAEZ"`, `"GENERCON"` and `"ENERGIA CONTROLADA"` in the code, so a
//! second user of the software could not print a certificate without a rebuild.

use eo_application::dtos::certificados::CertificadoDetalle;
use eo_application::dtos::reportes::GeneratedReport;
use eo_application::result::AppResult;
use eo_domain::{Decimal4, Money};

use super::canvas::{Align, Canvas, TextSpec};
use super::table::{Border, Cell, Row, Table, Width};
use super::theme::{self, size};
use crate::reporting::format::{format_date, format_money_plain, format_number, format_percent};
use crate::reporting::{filename, ReportContext};

/// Decimals of the progress percentages, per doc 12 §4.4.
const PCT_DECIMALS: u8 = 1;

pub fn generate(data: &CertificadoDetalle, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    // Landscape: it is what lets the nine columns fit.
    let mut canvas = Canvas::new(
        &ctx.t("Report.Certificado.Title"),
        theme::page::A4_HEIGHT,
        theme::page::A4_WIDTH,
        theme::page::MARGIN_CERTIFICADO,
    )?;

    encabezado(&mut canvas, data, ctx);
    tabla(&mut canvas, data, ctx);

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
        nombre_sugerido: filename::certificado(&data.obra_nombre, data.numero, data.fecha),
    })
}

fn encabezado(canvas: &mut Canvas, data: &CertificadoDetalle, ctx: &ReportContext) {
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

    let obra = if data.obra_nombre.trim().is_empty() {
        ctx.t("Report.Certificado.ObraGeneral")
    } else {
        data.obra_nombre.clone()
    };

    let filas = [
        ("Report.Certificado.Obra", obra),
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

fn tabla(canvas: &mut Canvas, data: &CertificadoDetalle, ctx: &ReportContext) {
    let mut table = Table::new(
        vec![
            Width::Relative(3.0),
            Width::Fixed(30.0),
            Width::Fixed(40.0),
            Width::Fixed(70.0),
            Width::Fixed(50.0),
            Width::Fixed(50.0),
            Width::Fixed(50.0),
            Width::Fixed(80.0),
            Width::Fixed(80.0),
        ],
        size::BODY_CERTIFICADO,
    );
    table.padding_v = 2.0;
    table.padding_h = 3.0;

    let grid = Border::hairline();
    let header_cell = |clave: &str| {
        Cell::new(ctx.t(clave))
            .align(Align::Center)
            .bold()
            .color(theme::WHITE)
            .size(size::CERT_HEADER)
            .fill(theme::REPORT_HEADER)
    };
    let sub_cell = |clave: &str| {
        Cell::new(ctx.t(clave))
            .align(Align::Center)
            .size(size::CERT_SUBHEADER)
            .fill(theme::REPORT_SUBHEADER)
    };

    table.header = vec![
        Row::new(vec![
            header_cell("Report.Certificado.ItemDescripcion").rowspan(2),
            header_cell("Report.Certificado.Computos").colspan(2),
            header_cell("Report.Certificado.PU").rowspan(2),
            header_cell("Report.Certificado.Avance").colspan(3),
            header_cell("Report.Certificado.Importe").colspan(2),
        ])
        .grid(grid),
        Row::new(vec![
            sub_cell("Report.Certificado.Und"),
            sub_cell("Report.Certificado.Cant"),
            sub_cell("Report.Certificado.Ant"),
            sub_cell("Report.Certificado.Act"),
            sub_cell("Report.Certificado.Acu"),
            sub_cell("Report.Certificado.Actual"),
            sub_cell("Report.Certificado.Acumulado"),
        ])
        .grid(grid),
    ];

    for item in &data.items {
        table.rows.push(
            Row::new(vec![
                Cell::new(item.descripcion.clone()),
                Cell::new(item.unidad.clone()).align(Align::Center),
                // Zero decimals unless the quantity actually has some: the legacy `N0` format
                // printed `2` for a quantity of `2,5` and hid half a unit.
                Cell::new(format_number(item.cantidad, &ctx.locale, 0)).align(Align::Center),
                Cell::new(format_money_plain(item.precio_unitario, &ctx.locale))
                    .align(Align::Right),
                Cell::new(format_percent(item.porcentaje_anterior, &ctx.locale, PCT_DECIMALS))
                    .align(Align::Center),
                Cell::new(format_percent(item.porcentaje_actual, &ctx.locale, PCT_DECIMALS))
                    .align(Align::Center)
                    .bold(),
                Cell::new(format_percent(
                    item.porcentaje_acumulado,
                    &ctx.locale,
                    PCT_DECIMALS,
                ))
                .align(Align::Center),
                Cell::new(format_money_plain(item.subtotal_actual, &ctx.locale))
                    .align(Align::Right),
                Cell::new(format_money_plain(item.subtotal_acumulado, &ctx.locale))
                    .align(Align::Right),
            ])
            .grid(grid),
        );
    }

    table.footer = pie_de_tabla(data, ctx, grid);
    table.render(canvas);
}

/// The closing rows, in the order doc 12 §4.5 fixes: subtotal, UOCRA adjustment computed on that
/// subtotal, then other deductions, then the total. Swapping the last two gives a different number.
///
/// Column 9 stays empty on all of them on purpose: the historical cumulative total is not adjusted
/// nor discounted, only the certificate in progress is.
fn pie_de_tabla(data: &CertificadoDetalle, ctx: &ReportContext, grid: Border) -> Vec<Row> {
    let acumulado_total = Money::try_sum(data.items.iter().map(|i| i.subtotal_acumulado))
        .unwrap_or(Money::ZERO);

    let etiqueta = |texto: String| {
        Cell::new(texto)
            .colspan(7)
            .align(Align::Right)
            .bold()
    };

    let mut filas = vec![Row::new(vec![
        etiqueta(ctx.t("Report.Certificado.SubTotal")),
        Cell::new(format_money_plain(data.total_certificado, &ctx.locale))
            .align(Align::Right)
            .bold(),
        Cell::new(format_money_plain(acumulado_total, &ctx.locale))
            .align(Align::Right)
            .bold(),
    ])
    .grid(grid)];

    if !data.ajuste_uocra.is_zero() {
        // The certificate freezes the adjustment as an amount, not as the percentage it came from
        // (doc 05 §2.5), so the percentage the label prints is derived back from the two totals.
        let porcentaje = porcentaje_ajuste(data.total_certificado, data.ajuste_uocra);
        filas.push(
            Row::new(vec![
                Cell::new(ctx.tp(
                    "Report.Certificado.AjusteUocra",
                    &[("porcentaje", &format_number(porcentaje, &ctx.locale, 0))],
                ))
                .colspan(7)
                .align(Align::Right)
                .italic(),
                Cell::new(format_money_plain(data.ajuste_uocra, &ctx.locale)).align(Align::Right),
                Cell::empty(),
            ])
            .grid(grid),
        );
    }

    if !data.otros_descuentos.is_zero() {
        filas.push(
            Row::new(vec![
                Cell::new(ctx.t("Report.Certificado.OtrosDescuentos"))
                    .colspan(7)
                    .align(Align::Right)
                    .italic(),
                Cell::new(format!(
                    "- {}",
                    format_money_plain(data.otros_descuentos, &ctx.locale)
                ))
                .align(Align::Right),
                Cell::empty(),
            ])
            .grid(grid),
        );
    }

    filas.push(
        Row::new(vec![
            Cell::new(ctx.t("Report.Certificado.TotalAFacturar"))
                .colspan(7)
                .align(Align::Right)
                .bold()
                .size(size::CERT_TOTAL)
                .fill(theme::TOTAL_A_FACTURAR_FILL),
            Cell::new(format_money_plain(data.total_neto, &ctx.locale))
                .align(Align::Right)
                .bold()
                .size(size::CERT_TOTAL)
                .fill(theme::TOTAL_A_FACTURAR_FILL),
            Cell::empty().fill(theme::TOTAL_A_FACTURAR_FILL),
        ])
        .grid(grid),
    );

    filas
}

/// `ajuste / total × 100`, for the label. Zero when there is no base to divide by.
fn porcentaje_ajuste(total: Money, ajuste: Money) -> Decimal4 {
    if total.is_zero() {
        return Decimal4::ZERO;
    }
    let proporcion = Decimal4::from_raw(ajuste.raw())
        .checked_div(Decimal4::from_raw(total.raw()))
        .unwrap_or(Decimal4::ZERO);
    proporcion
        .checked_mul(Decimal4::from_raw(100 * 10_000))
        .unwrap_or(Decimal4::ZERO)
        .round_to(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::tests_support::{certificado, contexto, pdf_text};

    #[test]
    fn pdf_certificado_tiene_las_nueve_columnas() {
        let texto = pdf_text(&generate(&certificado(2, "8", "0"), &contexto()).unwrap().bytes);
        for clave in [
            "ÍTEM / DESCRIPCIÓN",
            "UND",
            "CANT",
            "ANT",
            "ACT",
            "ACU",
            "ACTUAL",
            "ACUMULADO",
        ] {
            assert!(texto.contains(clave), "falta el rótulo {clave}: {texto}");
        }
    }

    #[test]
    fn pdf_certificado_es_landscape() {
        let bytes = generate(&certificado(1, "0", "0"), &contexto()).unwrap().bytes;
        // The page box is written in the PDF; landscape means the width exceeds the height.
        let (ancho, alto) = crate::reporting::tests_support::pdf_page_size(&bytes);
        assert!(ancho > alto, "no es landscape: {ancho} x {alto}");
    }

    #[test]
    fn pdf_certificado_orden_de_operaciones() {
        // 1 000 000 certified, an 8 % UOCRA adjustment and 50 000 of other deductions.
        // 1 000 000 − 80 000 − 50 000 = 870 000. Doing it the other way round gives 872 000.
        let data = certificado(1, "8", "50000");
        assert_eq!(data.total_neto, Money::parse("870000").unwrap());
        let texto = pdf_text(&generate(&data, &contexto()).unwrap().bytes);
        assert!(texto.contains("870.000,00"), "{texto}");
        assert!(texto.contains("80.000,00"), "falta el ajuste: {texto}");
    }

    #[test]
    fn pdf_certificado_ajuste_negativo_se_muestra() {
        // A negative adjustment is a rebate, and the legacy version only printed it when positive.
        let data = certificado(1, "-5", "0");
        assert!(data.ajuste_uocra.is_negative());
        let texto = pdf_text(&generate(&data, &contexto()).unwrap().bytes);
        assert!(texto.contains("AJUSTE UOCRA"), "{texto}");
        assert!(texto.contains("-50.000,00"), "{texto}");
    }

    #[test]
    fn pdf_certificado_sin_ajuste_ni_descuentos_no_muestra_esas_filas() {
        let texto = pdf_text(&generate(&certificado(1, "0", "0"), &contexto()).unwrap().bytes);
        assert!(!texto.contains("AJUSTE UOCRA"), "{texto}");
        assert!(!texto.contains("OTROS DESCUENTOS"), "{texto}");
        assert!(texto.contains("TOTAL A FACTURAR"), "{texto}");
    }

    #[test]
    fn pdf_usa_el_contratista_de_configuracion() {
        let texto = pdf_text(&generate(&certificado(1, "0", "0"), &contexto()).unwrap().bytes);
        assert!(texto.contains("Pablo Báez"), "{texto}");
        assert!(!texto.contains("PABLO BAEZ"), "quedó el literal legacy: {texto}");
    }

    #[test]
    fn pdf_sin_logo_configurado_no_falla() {
        let mut config = crate::reporting::tests_support::config();
        config.business.logo_path = None;
        let ctx = crate::reporting::tests_support::contexto_con(&config);
        let generado = generate(&certificado(1, "0", "0"), &ctx).unwrap();
        assert!(generado.bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn pdf_certificado_repite_encabezado() {
        let texto = pdf_text(&generate(&certificado(60, "0", "0"), &contexto()).unwrap().bytes);
        let veces = texto.matches("ACUMULADO").count();
        assert!(veces > 1, "el encabezado no se repitió: {veces}");
    }

    #[test]
    fn el_porcentaje_del_ajuste_se_deriva_de_los_totales() {
        let total = Money::parse("1000000").unwrap();
        let ajuste = Money::parse("80000").unwrap();
        assert_eq!(porcentaje_ajuste(total, ajuste), Decimal4::parse("8").unwrap());
        assert_eq!(porcentaje_ajuste(Money::ZERO, ajuste), Decimal4::ZERO);
    }
}
