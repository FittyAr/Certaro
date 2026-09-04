//! End-to-end exercise of the dashboard and the commercial analysis against a real database:
//! the KPIs and their comparison, profitability with its indirect imputation, the account
//! statement and the ageing buckets.

use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use certaro_application::config::AppConfig;
use certaro_application::dtos::categorias::CategoriaInput;
use certaro_application::dtos::clientes::ClienteInput;
use certaro_application::dtos::facturas::{FacturaInput, PagoFacturaInput};
use certaro_application::dtos::movimientos::MovimientoInput;
use certaro_application::dtos::proyectos::ProyectoInput;
use certaro_application::dtos::trabajos::TrabajoInput;
use certaro_application::ports::repositories::UnitOfWork;
use certaro_application::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use certaro_application::use_cases::categorias::CategoriasService;
use certaro_application::use_cases::clientes::ClientesService;
use certaro_application::use_cases::comercial::ComercialService;
use certaro_application::use_cases::dashboard::DashboardService;
use certaro_application::use_cases::facturas::FacturasService;
use certaro_application::use_cases::movimientos::MovimientosService;
use certaro_application::use_cases::proyectos::ProyectosService;
use certaro_application::use_cases::trabajos::TrabajosService;
use certaro_domain::clock::FixedClock;
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::ids::UuidV7Generator;
use certaro_domain::{Decimal4, EstadoFactura, Moneda, Money};
use certaro_infrastructure::config::FileSettingsStore;
use certaro_infrastructure::persistence::DbHandle;
use certaro_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use uuid::Uuid;

/// Frozen "now". Every window and every ageing bucket in this file is counted from here.
pub fn ahora() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 29, 12, 0, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

pub fn hoy() -> NaiveDate {
    ahora().date_naive()
}

/// A date `dias` before "now", which is how the tests place an invoice in a bucket.
pub fn hace(dias: i64) -> NaiveDate {
    hoy() - chrono::Duration::days(dias)
}

pub fn instante(dias: i64) -> DateTime<Utc> {
    ahora() - chrono::Duration::days(dias)
}

pub struct Fixture {
    pub dashboard: DashboardService,
    pub comercial: ComercialService,
    pub movimientos: MovimientosService,
    pub categorias: CategoriasService,
    pub clientes: ClientesService,
    pub proyectos: ProyectosService,
    pub trabajos: TrabajosService,
    pub facturas: FacturasService,
}

pub async fn fixture() -> Fixture {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db)));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(ahora()));
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    let settings: Arc<dyn SettingsStore> = Arc::new(FileSettingsStore::new(
        std::env::temp_dir().join("eo-test-dashboard.json"),
        AppConfig::default(),
    ));

    Fixture {
        dashboard: DashboardService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&settings),
        ),
        comercial: ComercialService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&settings),
        ),
        categorias: CategoriasService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        clientes: ClientesService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        proyectos: ProyectosService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        trabajos: TrabajosService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        facturas: FacturasService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&ids),
            Arc::clone(&settings),
        ),
        movimientos: MovimientosService::new(uow, clock, ids, settings),
    }
}

impl Fixture {
    pub async fn categoria(&self, nombre: &str) -> Uuid {
        self.categorias
            .create(CategoriaInput {
                nombre: nombre.to_owned(),
                descripcion: None,
                color_hex: None,
                icono: None,
                categoria_padre_id: None,
            })
            .await
            .unwrap()
            .id
    }

    pub async fn cliente(&self, nombre: &str) -> Uuid {
        self.clientes
            .create(ClienteInput {
                nombre: nombre.to_owned(),
                cuit: None,
                direccion: None,
                telefono: None,
                email: None,
                condicion_iva: None,
                contactos: Vec::new(),
            })
            .await
            .unwrap()
            .id
    }

    pub async fn proyecto(&self, numero: i32, nombre: &str, cliente_id: Uuid) -> Uuid {
        self.proyectos
            .create(ProyectoInput {
                numero,
                nombre: nombre.to_owned(),
                direccion: None,
                localidad: None,
                cliente_id,
            })
            .await
            .unwrap()
            .id
    }

    pub async fn trabajo(&self, proyecto_id: Uuid, descripcion: &str) -> Uuid {
        self.trabajos
            .create(TrabajoInput {
                proyecto_id,
                descripcion: descripcion.to_owned(),
                fecha_inicio: hace(90),
                fecha_fin: None,
                presupuesto: Money::ZERO,
            })
            .await
            .unwrap()
            .id
    }

    /// One movement. `dias` places it that many days before "now", which is what puts it inside or
    /// outside a window.
    #[allow(clippy::too_many_arguments)]
    pub async fn movimiento(
        &self,
        concepto: &str,
        monto: &str,
        cantidad: &str,
        es_ingreso: bool,
        dias: i64,
        categoria_id: Option<Uuid>,
        cliente_id: Option<Uuid>,
        trabajo_id: Option<Uuid>,
    ) -> Uuid {
        self.movimientos
            .create(MovimientoInput {
                fecha: instante(dias),
                concepto: concepto.to_owned(),
                monto: Money::parse(monto).unwrap(),
                cantidad: Decimal4::parse(cantidad).unwrap(),
                tipo_movimiento_id: if es_ingreso {
                    tipos_movimiento::INGRESO
                } else {
                    tipos_movimiento::GASTO
                },
                moneda: Moneda::Ars,
                cotizacion_aplicada: None,
                tipo_concepto_pago_id: None,
                categoria_id,
                cliente_id,
                trabajo_id,
                empleado_id: None,
                factura_id: None,
            })
            .await
            .unwrap()
            .item
            .id
    }

    /// An issued invoice. `dias_atras` is how old it is, which is what its bucket depends on.
    pub async fn factura(&self, numero: &str, cliente_id: Uuid, total: &str, dias_atras: i64) -> Uuid {
        let total = Money::parse(total).unwrap();
        let creada = self
            .facturas
            .create(FacturaInput {
                numero: numero.to_owned(),
                fecha: hace(dias_atras),
                fecha_vencimiento: None,
                cliente_id,
                subtotal: total,
                iva: Money::ZERO,
                total,
                observaciones: None,
            })
            .await
            .unwrap();

        // A draft is not a receivable: the statement and the ageing only see issued invoices.
        self.facturas
            .transition(creada.id, EstadoFactura::Emitida, &creada.audit.row_version)
            .await
            .unwrap()
            .id
    }

    /// An issued invoice whose due date is `dias_mora` days back, so the arrears the statement
    /// derives are exactly that number and not the number minus the default term.
    pub async fn factura_vencida(
        &self,
        numero: &str,
        cliente_id: Uuid,
        total: &str,
        dias_mora: i64,
    ) -> Uuid {
        let total = Money::parse(total).unwrap();
        let creada = self
            .facturas
            .create(FacturaInput {
                numero: numero.to_owned(),
                fecha: hace(dias_mora),
                fecha_vencimiento: Some(hace(dias_mora)),
                cliente_id,
                subtotal: total,
                iva: Money::ZERO,
                total,
                observaciones: None,
            })
            .await
            .unwrap();

        self.facturas
            .transition(creada.id, EstadoFactura::Emitida, &creada.audit.row_version)
            .await
            .unwrap()
            .id
    }

    pub async fn pagar(&self, factura_id: Uuid, monto: &str) {
        self.facturas
            .crear_pago(PagoFacturaInput {
                factura_id,
                fecha: hoy(),
                monto: Money::parse(monto).unwrap(),
                medio_pago: "Efectivo".to_owned(),
            })
            .await
            .unwrap();
    }
}
