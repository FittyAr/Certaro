use super::*;
use super::table::porcentaje_ajuste;
use certaro_domain::{Decimal4, Money};
use crate::reporting::tests_support::{certificado, contexto, pdf_text};

#[test]
fn pdf_certificado_tiene_las_nueve_columnas() {
    let texto = pdf_text(
        &generate(&certificado(2, "8", "0"), &contexto())
            .unwrap()
            .bytes,
    );
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
    let bytes = generate(&certificado(1, "0", "0"), &contexto())
        .unwrap()
        .bytes;
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
    let texto = pdf_text(
        &generate(&certificado(1, "0", "0"), &contexto())
            .unwrap()
            .bytes,
    );
    assert!(!texto.contains("AJUSTE UOCRA"), "{texto}");
    assert!(!texto.contains("OTROS DESCUENTOS"), "{texto}");
    assert!(texto.contains("TOTAL A FACTURAR"), "{texto}");
}

#[test]
fn pdf_usa_el_contratista_de_configuracion() {
    let texto = pdf_text(
        &generate(&certificado(1, "0", "0"), &contexto())
            .unwrap()
            .bytes,
    );
    assert!(texto.contains("Pablo Báez"), "{texto}");
    assert!(
        !texto.contains("PABLO BAEZ"),
        "quedó el literal legacy: {texto}"
    );
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
    let texto = pdf_text(
        &generate(&certificado(60, "0", "0"), &contexto())
            .unwrap()
            .bytes,
    );
    let veces = texto.matches("ACUMULADO").count();
    assert!(veces > 1, "el encabezado no se repitió: {veces}");
}

#[test]
fn el_porcentaje_del_ajuste_se_deriva_de_los_totales() {
    let total = Money::parse("1000000").unwrap();
    let ajuste = Money::parse("80000").unwrap();
    assert_eq!(
        porcentaje_ajuste(total, ajuste),
        Decimal4::parse("8").unwrap()
    );
    assert_eq!(porcentaje_ajuste(Money::ZERO, ajuste), Decimal4::ZERO);
}
