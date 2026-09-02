//! Use cases of `reportes`. See `docs/12-reportes-y-exportaciones.md` and `docs/09` §3.12.
//!
//! Two rules the legacy system broke and this enforces:
//!
//! 1. The export covers **the whole active filter, unpaginated**. Exporting from the movements
//!    screen used to send only the visible page, while the report centre sent everything unfiltered
//!    — neither is what the user meant.
//! 2. The destination is the user's. The settlement PDF used to be written straight to the desktop
//!    without asking.

use std::sync::Arc;

use tracing::info;

use crate::dtos::movimientos::{MovimientoFiltroDto, MovimientoListItem, MovimientoResumenDto};
use crate::dtos::reportes::{
    ExportResult, FiltroDescripcion, FormatoExport, GeneratedReport, ReporteMovimientos,
    ReporteRequest,
};
use crate::error::{AppError, FieldError};
use crate::paging::PageRequest;
use crate::ports::repositories::{SortDir, UnitOfWork};
use crate::ports::{FileWriterPort, ReportPort};
use crate::result::AppResult;
use crate::use_cases::shared::finish_read;

pub struct ReportesService {
    uow: Arc<dyn UnitOfWork>,
    reports: Arc<dyn ReportPort>,
    writer: Arc<dyn FileWriterPort>,
}

impl ReportesService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        reports: Arc<dyn ReportPort>,
        writer: Arc<dyn FileWriterPort>,
    ) -> Self {
        Self {
            uow,
            reports,
            writer,
        }
    }

    /// Generates the report and writes it to `destino`, which the frontend obtained from the
    /// system dialog.
    pub async fn generar(
        &self,
        request: ReporteRequest,
        formato: FormatoExport,
        destino: String,
    ) -> AppResult<ExportResult> {
        if !request.formatos().contains(&formato) {
            return Err(AppError::Validation(vec![FieldError::new(
                "formato",
                "Validation.Export.FormatoNoSoportado",
            )
            .with_param("reporte", request.nombre())]));
        }

        let generado = self.generate(&request, formato).await?;
        let bytes = self
            .writer
            .write(std::path::Path::new(&destino), &generado.bytes, formato)?;

        info!(
            reporte = request.nombre(),
            formato = formato.extension(),
            registros = generado.registros,
            bytes,
            "reporte generado"
        );

        Ok(ExportResult {
            ruta: destino,
            bytes,
            registros: generado.registros,
        })
    }

    /// The document in memory, without touching the disk. Split out so a caller that only needs
    /// the bytes — a preview, a test — does not have to invent a path.
    pub async fn generate(
        &self,
        request: &ReporteRequest,
        formato: FormatoExport,
    ) -> AppResult<GeneratedReport> {
        match request {
            ReporteRequest::Movimientos { filtro } => {
                let data = self.datos_movimientos(filtro.clone()).await?;
                self.reports.movimientos(&data, formato)
            }
            ReporteRequest::Liquidacion { id } => {
                let tx = self.uow.begin().await?;
                let cargado =
                    crate::use_cases::liquidaciones::load_detalle(tx.liquidaciones(), *id).await;
                let detalle = finish_read(tx, cargado).await?;
                self.reports.liquidacion(&detalle)
            }
            ReporteRequest::Certificado { id } => {
                let tx = self.uow.begin().await?;
                let cargado = crate::use_cases::certificados::load_detalle(&*tx, *id).await;
                let detalle = finish_read(tx, cargado).await?;
                self.reports.certificado(&detalle)
            }
        }
    }

    /// The name to prefill the save dialog with.
    pub fn nombre_sugerido(
        &self,
        reporte: &str,
        formato: FormatoExport,
        detalle: Option<&str>,
    ) -> AppResult<String> {
        self.reports.nombre_sugerido(reporte, formato, detalle)
    }

    /// Every row of the filter plus the summary, in one transaction so the totals describe exactly
    /// the rows that were exported.
    async fn datos_movimientos(
        &self,
        filtro: MovimientoFiltroDto,
    ) -> AppResult<ReporteMovimientos> {
        let dominio = filtro.clone().into();
        let tx = self.uow.begin().await?;
        let cargado = async {
            let repo = tx.movimientos();
            // Size zero is «no paging»: the export is the filter, not the page on screen.
            let page = repo
                .search(&dominio, PageRequest::new(1, 0), None, SortDir::Asc)
                .await?;
            let resumen = repo.resumen(&dominio).await?;
            let items = page
                .items
                .into_iter()
                .map(MovimientoListItem::try_from)
                .collect::<AppResult<Vec<_>>>()?;
            Ok::<_, AppError>((items, MovimientoResumenDto::from(resumen)))
        }
        .await;
        let (items, resumen) = finish_read(tx, cargado).await?;

        Ok(ReporteMovimientos {
            filtros_descripcion: describir(&filtro, &items),
            filtro,
            items,
            resumen,
        })
    }
}

/// The filters in prose, as label keys with their values.
///
/// The names of the filtered entities are taken from the exported rows rather than looked up: a
/// row that matched a customer filter carries that customer's name, and when nothing matched there
/// is nothing to name anyway. That keeps the export at two queries.
fn describir(filtro: &MovimientoFiltroDto, items: &[MovimientoListItem]) -> Vec<FiltroDescripcion> {
    let mut out = Vec::new();
    let mut push = |clave: &str, valor: String| {
        out.push(FiltroDescripcion {
            clave: clave.to_owned(),
            valor,
        });
    };

    if let Some(texto) = filtro.concepto.as_deref().filter(|t| !t.trim().is_empty()) {
        push("Report.Filtro.Texto", texto.trim().to_owned());
    }
    if let Some(desde) = filtro.fecha_desde {
        push("Report.Filtro.Desde", desde.to_string());
    }
    if let Some(hasta) = filtro.fecha_hasta {
        push("Report.Filtro.Hasta", hasta.to_string());
    }
    if filtro.tipo_movimiento_id.is_some() {
        if let Some(nombre) = items.first().map(|i| i.tipo_movimiento_nombre.clone()) {
            push("Report.Filtro.Tipo", nombre);
        }
    }
    if filtro.categoria_id.is_some() {
        if let Some(nombre) = items.first().and_then(|i| i.categoria_nombre.clone()) {
            push("Report.Filtro.Categoria", nombre);
        }
    }
    if filtro.cliente_id.is_some() {
        if let Some(nombre) = items.first().and_then(|i| i.cliente_nombre.clone()) {
            push("Report.Filtro.Cliente", nombre);
        }
    }
    if filtro.trabajo_id.is_some() {
        if let Some(nombre) = items.first().and_then(|i| i.trabajo_descripcion.clone()) {
            push("Report.Filtro.Trabajo", nombre);
        }
    }
    if let Some(moneda) = filtro.moneda {
        push("Report.Filtro.Moneda", moneda.iso().to_owned());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use certaro_domain::{Decimal4, Moneda, Money};
    use uuid::Uuid;

    fn item() -> MovimientoListItem {
        MovimientoListItem {
            id: Uuid::from_u128(1),
            fecha: chrono::Utc::now(),
            concepto: "Cable".to_owned(),
            monto: Money::ZERO,
            cantidad: Decimal4::ONE,
            total: Money::ZERO,
            moneda: Moneda::Ars,
            cotizacion_aplicada: None,
            tipo_movimiento_id: Uuid::from_u128(2),
            tipo_movimiento_nombre: "Gasto".to_owned(),
            es_ingreso: false,
            categoria_id: Some(Uuid::from_u128(3)),
            categoria_nombre: Some("Materiales".to_owned()),
            categoria_color: None,
            cliente_id: Some(Uuid::from_u128(4)),
            cliente_nombre: Some("Acme".to_owned()),
            trabajo_id: Some(Uuid::from_u128(5)),
            trabajo_descripcion: Some("Tablero".to_owned()),
            proyecto_nombre: Some("Edificio Sur".to_owned()),
            empleado_id: None,
            factura_id: None,
            tipo_concepto_pago_id: None,
            bloqueado_por_liquidacion: false,
            row_version: "0000000000000001".to_owned(),
        }
    }

    #[test]
    fn sin_filtros_no_hay_nada_que_describir() {
        assert!(describir(&MovimientoFiltroDto::default(), &[item()]).is_empty());
    }

    #[test]
    fn cada_filtro_aporta_su_linea_con_el_nombre_de_la_fila() {
        let filtro = MovimientoFiltroDto {
            concepto: Some("  cable ".to_owned()),
            cliente_id: Some(Uuid::from_u128(4)),
            categoria_id: Some(Uuid::from_u128(3)),
            fecha_desde: NaiveDate::from_ymd_opt(2026, 8, 1),
            moneda: Some(Moneda::Ars),
            ..MovimientoFiltroDto::default()
        };
        let descripcion = describir(&filtro, &[item()]);
        let claves: Vec<&str> = descripcion.iter().map(|d| d.clave.as_str()).collect();
        assert_eq!(
            claves,
            vec![
                "Report.Filtro.Texto",
                "Report.Filtro.Desde",
                "Report.Filtro.Categoria",
                "Report.Filtro.Cliente",
                "Report.Filtro.Moneda"
            ]
        );
        assert_eq!(descripcion[0].valor, "cable");
        assert_eq!(descripcion[3].valor, "Acme");
    }

    #[test]
    fn un_filtro_que_no_devolvio_filas_no_inventa_un_nombre() {
        let filtro = MovimientoFiltroDto {
            cliente_id: Some(Uuid::from_u128(4)),
            ..MovimientoFiltroDto::default()
        };
        assert!(describir(&filtro, &[]).is_empty());
    }

    #[test]
    fn un_concepto_en_blanco_no_cuenta_como_filtro() {
        let filtro = MovimientoFiltroDto {
            concepto: Some("   ".to_owned()),
            ..MovimientoFiltroDto::default()
        };
        assert!(describir(&filtro, &[item()]).is_empty());
    }
}
