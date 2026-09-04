use certaro_application::dtos::certificados::CertificadoDetalle;
use certaro_domain::{Decimal4, Money};

use super::super::canvas::{Align, Canvas};
use super::super::table::{Border, Cell, Row, Table, Width};
use super::super::theme::{self, size};
use crate::reporting::format::{format_money_plain, format_number, format_percent};
use crate::reporting::ReportContext;

const PCT_DECIMALS: u8 = 1;

pub(super) fn tabla(canvas: &mut Canvas, data: &CertificadoDetalle, ctx: &ReportContext) {
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
                Cell::new(format_percent(
                    item.porcentaje_anterior,
                    &ctx.locale,
                    PCT_DECIMALS,
                ))
                .align(Align::Center),
                Cell::new(format_percent(
                    item.porcentaje_actual,
                    &ctx.locale,
                    PCT_DECIMALS,
                ))
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
    let acumulado_total =
        Money::try_sum(data.items.iter().map(|i| i.subtotal_acumulado)).unwrap_or(Money::ZERO);

    let etiqueta = |texto: String| Cell::new(texto).colspan(7).align(Align::Right).bold();

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
pub(super) fn porcentaje_ajuste(total: Money, ajuste: Money) -> Decimal4 {
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
