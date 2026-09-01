//! Fixtures shared by the report tests. Compiled only for tests.

use std::sync::Arc;

use chrono::{TimeZone, Utc};
use certaro_application::config::AppConfig;
use certaro_application::dtos::movimientos::{MovimientoListItem, MovimientoResumenDto};
use certaro_application::dtos::reportes::ReporteMovimientos;
use certaro_domain::{Decimal4, Moneda, Money};
use uuid::Uuid;

use super::ReportContext;
use crate::i18n::JsonTranslator;

/// A fixed instant, so every generated document is byte-comparable across runs.
#[must_use]
pub fn instante() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 29, 15, 30, 12).unwrap()
}

#[must_use]
pub fn config() -> AppConfig {
    let mut config = AppConfig::default();
    config.business.nombre_comercial = "GENERCON".to_owned();
    config.business.lema = "Energía controlada".to_owned();
    config.business.contratista = "Pablo Báez".to_owned();
    config
}

#[must_use]
pub fn contexto() -> ReportContext {
    contexto_con(&config())
}

#[must_use]
pub fn contexto_con(config: &AppConfig) -> ReportContext {
    ReportContext::new(
        config,
        Arc::new(JsonTranslator::new(&config.locale.language)),
        instante(),
    )
}

#[must_use]
pub fn movimiento(concepto: &str, monto: &str, cantidad: &str) -> MovimientoListItem {
    let monto = Money::parse(monto).unwrap();
    let cantidad = Decimal4::parse(cantidad).unwrap();
    MovimientoListItem {
        id: Uuid::from_u128(7),
        fecha: instante(),
        concepto: concepto.to_owned(),
        monto,
        cantidad,
        total: monto.checked_mul(cantidad).unwrap(),
        moneda: Moneda::Ars,
        cotizacion_aplicada: None,
        tipo_movimiento_id: Uuid::from_u128(1),
        tipo_movimiento_nombre: "Gasto".to_owned(),
        es_ingreso: false,
        categoria_id: Some(Uuid::from_u128(2)),
        categoria_nombre: Some("Materiales".to_owned()),
        categoria_color: Some("#123456".to_owned()),
        cliente_id: Some(Uuid::from_u128(3)),
        cliente_nombre: Some("Acme S.A.".to_owned()),
        trabajo_id: Some(Uuid::from_u128(4)),
        trabajo_descripcion: Some("Tablero principal".to_owned()),
        obra_nombre: Some("Edificio Sur".to_owned()),
        empleado_id: None,
        factura_id: None,
        tipo_concepto_pago_id: None,
        bloqueado_por_liquidacion: false,
        row_version: "0000000000000001".to_owned(),
    }
}

/// A report with `cantidad` rows and a summary consistent with them.
#[must_use]
pub fn reporte(items: Vec<MovimientoListItem>) -> ReporteMovimientos {
    let mut ingresos = Money::ZERO;
    let mut gastos = Money::ZERO;
    for item in &items {
        if item.es_ingreso {
            ingresos = ingresos.checked_add(item.total).unwrap();
        } else {
            gastos = gastos.checked_add(item.total).unwrap();
        }
    }
    let cantidad = items.len() as u64;
    ReporteMovimientos {
        items,
        resumen: MovimientoResumenDto {
            total_ingresos: ingresos,
            total_gastos: gastos,
            balance: ingresos.checked_sub(gastos).unwrap(),
            cantidad,
        },
        filtro: Default::default(),
        filtros_descripcion: Vec::new(),
    }
}

/// A settlement with `adelantos` dated advances, so the receipt can be checked line by line.
#[must_use]
pub fn liquidacion(
    adelantos: usize,
    bruto: &str,
    monto_adelanto: &str,
) -> certaro_application::dtos::liquidaciones::LiquidacionDetalle {
    use certaro_application::dtos::liquidaciones::{LiquidacionAdelantoDto, LiquidacionDetalle};

    let bruto = Money::parse(bruto).unwrap();
    let unitario = Money::parse(monto_adelanto).unwrap();
    let inicio = chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();

    let lista: Vec<LiquidacionAdelantoDto> = (0..adelantos)
        .map(|i| LiquidacionAdelantoDto {
            id: Uuid::from_u128(100 + i as u128),
            movimiento_id: Uuid::from_u128(200 + i as u128),
            fecha: inicio + chrono::Duration::days(i as i64 * 3),
            concepto: format!("Adelanto {}", i + 1),
            monto: unitario,
        })
        .collect();

    let total_adelantos = Money::try_sum(lista.iter().map(|a| a.monto)).unwrap();

    LiquidacionDetalle {
        id: Uuid::from_u128(50),
        empleado_id: Uuid::from_u128(51),
        empleado_nombre: "Juan Pérez".to_owned(),
        empleado_cargo: Some("Oficial electricista".to_owned()),
        empleado_dni: Some("20.123.456".to_owned()),
        fecha_inicio: inicio,
        fecha_fin: chrono::NaiveDate::from_ymd_opt(2026, 8, 31).unwrap(),
        dias_trabajados: Decimal4::parse("24").unwrap(),
        tarifa_aplicada: bruto
            .checked_div(Decimal4::parse("24").unwrap())
            .unwrap_or(Money::ZERO),
        incluir_sabados: true,
        incluir_domingos: false,
        incluir_feriados: true,
        multiplicador_sabado: Decimal4::parse("1.5").unwrap(),
        multiplicador_domingo: Decimal4::parse("2").unwrap(),
        multiplicador_feriado: Decimal4::parse("2").unwrap(),
        total_bruto: bruto,
        total_adelantos,
        total_neto: bruto.checked_sub(total_adelantos).unwrap(),
        observaciones: Some("Quincena completa".to_owned()),
        pdf_generado_at: None,
        admite_cambio_de_importes: true,
        adelantos: lista,
        audit: auditoria(),
    }
}

/// A certificate of `items` lines totalling 1 000 000, with the given UOCRA percentage and other
/// deductions, and its net computed in the order doc 06 §5.4 fixes.
#[must_use]
pub fn certificado(
    items: usize,
    ajuste_pct: &str,
    otros: &str,
) -> certaro_application::dtos::certificados::CertificadoDetalle {
    use certaro_application::dtos::certificados::{CertificadoDetalle, CertificadoItemDto};

    let total = Money::parse("1000000").unwrap();
    let por_item = total
        .checked_div(Decimal4::from_units(items.max(1) as i64).unwrap())
        .unwrap();

    let lista: Vec<CertificadoItemDto> = (0..items)
        .map(|i| CertificadoItemDto {
            id: Uuid::from_u128(300 + i as u128),
            orden_trabajo_item_id: Uuid::from_u128(400 + i as u128),
            descripcion: format!("Bandeja portacables {}", i + 1),
            unidad: "ml".to_owned(),
            cantidad: Decimal4::parse("2.5").unwrap(),
            precio_unitario: Money::parse("40000").unwrap(),
            porcentaje_anterior: Decimal4::parse("20").unwrap(),
            porcentaje_actual: Decimal4::parse("30").unwrap(),
            porcentaje_acumulado: Decimal4::parse("50").unwrap(),
            subtotal_actual: por_item,
            subtotal_acumulado: por_item,
        })
        .collect();

    let ajuste = total.percent(Decimal4::parse(ajuste_pct).unwrap()).unwrap();
    let otros = Money::parse(otros).unwrap();

    CertificadoDetalle {
        id: Uuid::from_u128(60),
        numero: 3,
        fecha: chrono::NaiveDate::from_ymd_opt(2026, 8, 29).unwrap(),
        observaciones: None,
        orden_trabajo_id: Uuid::from_u128(61),
        orden_titulo: "Instalación eléctrica planta baja".to_owned(),
        trabajo_id: Uuid::from_u128(62),
        trabajo_descripcion: "Tablero principal".to_owned(),
        obra_id: Uuid::from_u128(63),
        obra_numero: 12,
        obra_nombre: "Edificio Sur".to_owned(),
        cliente_id: Uuid::from_u128(64),
        cliente_nombre: "Acme S.A.".to_owned(),
        total_certificado: total,
        ajuste_uocra: ajuste,
        otros_descuentos: otros,
        total_neto: total
            .checked_sub(ajuste)
            .unwrap()
            .checked_sub(otros)
            .unwrap(),
        items: lista,
        es_ultimo: true,
        audit: auditoria(),
    }
}

fn auditoria() -> certaro_application::dtos::AuditDto {
    certaro_application::dtos::AuditDto {
        created_at: instante(),
        updated_at: None,
        row_version: "0000000000000001".to_owned(),
        is_deleted: false,
        deleted_at: None,
    }
}

/// The visible text of a DOCX, read out of `word/document.xml`. Comparing paragraphs is what
/// doc 17 §4.4 asks for; the XML around them changes with the library.
#[must_use]
pub fn docx_text(bytes: &[u8]) -> String {
    let xml = zip_entry(bytes, "word/document.xml");
    let mut out = String::new();
    let mut resto = xml.as_str();
    while let Some(inicio) = resto.find("<w:t") {
        resto = &resto[inicio..];
        let Some(abre) = resto.find('>') else { break };
        let Some(cierra) = resto.find("</w:t>") else {
            break;
        };
        out.push_str(&resto[abre + 1..cierra]);
        out.push('\n');
        resto = &resto[cierra + 6..];
    }
    out.replace("&#233;", "é")
        .replace("&#237;", "í")
        .replace("&#243;", "ó")
        .replace("&#225;", "á")
        .replace("&#250;", "ú")
        .replace("&#241;", "ñ")
        .replace("&#183;", "·")
}

/// One entry of a zip container, as text. Used for both DOCX and XLSX.
#[must_use]
pub fn zip_entry(bytes: &[u8], nombre: &str) -> String {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec()))
        .expect("el archivo no es un zip válido");
    let mut entry = archive
        .by_name(nombre)
        .unwrap_or_else(|_| panic!("el zip no tiene {nombre}"));
    let mut texto = String::new();
    entry
        .read_to_string(&mut texto)
        .expect("la entrada no es texto");
    texto
}

/// Width and height of the first page, in points, read from the PDF itself.
#[must_use]
pub fn pdf_page_size(bytes: &[u8]) -> (f32, f32) {
    let texto = String::from_utf8_lossy(bytes);
    let inicio = texto.find("/MediaBox").expect("el PDF no declara MediaBox");
    let resto = &texto[inicio..];
    let abre = resto.find('[').expect("MediaBox sin corchete");
    let cierra = resto.find(']').expect("MediaBox sin cierre");
    let numeros: Vec<f32> = resto[abre + 1..cierra]
        .split_whitespace()
        .filter_map(|n| n.parse().ok())
        .collect();
    (numeros[2] - numeros[0], numeros[3] - numeros[1])
}

/// The text of a generated PDF. Layout is verified by what the document says, not by its bytes:
/// a PDF library changes its output between versions without changing a single visible value
/// (doc 17 §4.4).
#[must_use]
pub fn pdf_text(bytes: &[u8]) -> String {
    pdf_extract::extract_text_from_mem(bytes).unwrap_or_default()
}

#[must_use]
pub fn filas(cantidad: usize) -> Vec<MovimientoListItem> {
    (0..cantidad)
        .map(|i| movimiento(&format!("Ítem {i}"), "1000", "1"))
        .collect()
}
