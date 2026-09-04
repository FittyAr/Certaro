//! End-to-end exercise of the personnel modules against a real database: employees, the attendance
//! grid with its click cycle, the holiday table and the settlement with its frozen advances.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use certaro_application::config::AppConfig;
use certaro_application::dtos::asistencias::AsistenciaUpsertInput;
use certaro_application::dtos::categorias::CategoriaInput;
use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::empleados::EmpleadoInput;
use certaro_application::dtos::liquidaciones::LiquidacionInput;
use certaro_application::dtos::movimientos::MovimientoInput;
use certaro_application::ports::holidays::HolidayProvider;
use certaro_application::ports::repositories::{SortDir, UnitOfWork};
use certaro_application::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use certaro_application::use_cases::asistencias::AsistenciasService;
use certaro_application::use_cases::categorias::CategoriasService;
use certaro_application::use_cases::empleados::EmpleadosService;
use certaro_application::use_cases::feriados::FeriadosService;
use certaro_application::use_cases::liquidaciones::LiquidacionesService;
use certaro_application::use_cases::movimientos::MovimientosService;
use certaro_application::{AppError, AppResult};
use certaro_domain::clock::FixedClock;
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::entities::{Feriado, OrigenFeriado};
use certaro_domain::ids::UuidV7Generator;
use certaro_domain::{Decimal4, FrecuenciaPago, Moneda, Money, TipoJornada};
use certaro_infrastructure::config::FileSettingsStore;
use certaro_infrastructure::persistence::DbHandle;
use certaro_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use uuid::Uuid;

/// A provider with a fixed calendar, so the tests never touch the network. Returning an error is
/// what proves the degradation path.
pub struct FakeHolidays {
    pub feriados: Vec<(NaiveDate, &'static str)>,
    pub falla: bool,
}

#[async_trait]
impl HolidayProvider for FakeHolidays {
    async fn fetch(&self, anio: i32) -> AppResult<Vec<Feriado>> {
        if self.falla {
            return Err(AppError::ExternalUnavailable { service: "test" });
        }
        let now = ahora();
        Ok(self
            .feriados
            .iter()
            .filter(|(fecha, _)| fecha.format("%Y").to_string() == format!("{anio:04}"))
            .map(|(fecha, nombre)| Feriado {
                fecha: *fecha,
                nombre: (*nombre).to_owned(),
                tipo: Some("inamovible".to_owned()),
                origen: OrigenFeriado::Api,
                created_at: now,
                updated_at: None,
            })
            .collect())
    }
}

pub struct Fixture {
    pub empleados: EmpleadosService,
    pub asistencias: AsistenciasService,
    pub liquidaciones: LiquidacionesService,
    pub feriados: FeriadosService,
    pub movimientos: MovimientosService,
    pub categorias: CategoriasService,
}

pub fn ahora() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 30, 12, 0, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

pub fn query<F>(filtro: F) -> ListQuery<F> {
    ListQuery {
        filtro,
        page: 1,
        page_size: 30,
        sort_by: None,
        sort_dir: SortDir::Asc,
    }
}

pub fn dia(mes: u32, dia: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, mes, dia).unwrap()
}

pub async fn fixture_con(provider: FakeHolidays) -> Fixture {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db)));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(ahora()));
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    let settings: Arc<dyn SettingsStore> = Arc::new(FileSettingsStore::new(
        std::env::temp_dir().join("eo-test-personal.json"),
        AppConfig::default(),
    ));
    let holidays: Arc<dyn HolidayProvider> = Arc::new(provider);

    Fixture {
        empleados: EmpleadosService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        asistencias: AsistenciasService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&ids),
            Arc::clone(&settings),
        ),
        liquidaciones: LiquidacionesService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&ids),
        ),
        feriados: FeriadosService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            holidays,
            Arc::clone(&settings),
        ),
        categorias: CategoriasService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        movimientos: MovimientosService::new(uow, clock, ids, settings),
    }
}

pub async fn fixture() -> Fixture {
    fixture_con(FakeHolidays {
        feriados: vec![(dia(6, 15), "Prueba")],
        falla: false,
    })
    .await
}

pub fn empleado_input(nombre: &str, tarifa: &str) -> EmpleadoInput {
    EmpleadoInput {
        nombre: nombre.to_owned(),
        dni: None,
        cargo: Some("Oficial".to_owned()),
        sueldo_base: Money::ZERO,
        pago_frecuencia: FrecuenciaPago::Quincenal,
        tarifa_diaria: Money::parse(tarifa).unwrap(),
        multiplicador_sabado: Decimal4::parse("1.5").unwrap(),
        multiplicador_domingo: Decimal4::parse("2.0").unwrap(),
        multiplicador_feriado: Decimal4::parse("2.0").unwrap(),
        email: None,
        telefono: None,
        fecha_ingreso: dia(1, 5),
        fecha_egreso: None,
        activo: true,
    }
}

impl Fixture {
    pub async fn empleado(&self, nombre: &str, tarifa: &str) -> Uuid {
        self.empleados
            .create(empleado_input(nombre, tarifa))
            .await
            .unwrap()
            .id
    }

    pub async fn marcar(&self, empleado_id: Uuid, fecha: NaiveDate, tipo: Option<TipoJornada>) {
        self.asistencias
            .upsert(AsistenciaUpsertInput {
                empleado_id,
                fecha,
                tipo_jornada: tipo,
                trabajo_id: None,
                observaciones: None,
            })
            .await
            .unwrap();
    }

    /// An advance, as the movements module records it: the payroll finds it by the seeded type.
    pub async fn adelanto(&self, empleado_id: Uuid, fecha: NaiveDate, monto: &str) -> Uuid {
        let categoria = self
            .categorias
            .create(CategoriaInput {
                nombre: format!("Sueldos {fecha}"),
                descripcion: None,
                color_hex: None,
                icono: None,
                categoria_padre_id: None,
            })
            .await
            .unwrap()
            .id;

        self.movimientos
            .create(MovimientoInput {
                fecha: fecha
                    .and_hms_opt(10, 0, 0)
                    .map(|d| Utc.from_utc_datetime(&d))
                    .unwrap(),
                concepto: "Adelanto".to_owned(),
                monto: Money::parse(monto).unwrap(),
                cantidad: Decimal4::ONE,
                tipo_movimiento_id: tipos_movimiento::ADELANTO,
                moneda: Moneda::Ars,
                cotizacion_aplicada: None,
                tipo_concepto_pago_id: None,
                categoria_id: Some(categoria),
                cliente_id: None,
                trabajo_id: None,
                empleado_id: Some(empleado_id),
                factura_id: None,
            })
            .await
            .unwrap()
            .item
            .id
    }
}

pub fn liquidacion_input(
    empleado_id: Uuid,
    desde: NaiveDate,
    hasta: NaiveDate,
    dias: &str,
) -> LiquidacionInput {
    let dias_trabajados = Decimal4::parse(dias).unwrap();
    let tarifa = Money::parse("10000.0000").unwrap();
    LiquidacionInput {
        empleado_id,
        fecha_inicio: desde,
        fecha_fin: hasta,
        dias_trabajados,
        tarifa_aplicada: tarifa,
        incluir_sabados: false,
        incluir_domingos: false,
        incluir_feriados: false,
        multiplicador_sabado: Decimal4::ONE,
        multiplicador_domingo: Decimal4::ONE,
        multiplicador_feriado: Decimal4::ONE,
        total_bruto: tarifa.checked_mul(dias_trabajados).unwrap(),
        total_adelantos: Money::ZERO,
        observaciones: None,
        adelantos: vec![],
    }
}
