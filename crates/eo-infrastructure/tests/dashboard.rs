//! End-to-end exercise of the dashboard and the commercial analysis against a real database:
//! the KPIs and their comparison, profitability with its indirect imputation, the account
//! statement and the ageing buckets.

use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use eo_application::config::AppConfig;
use eo_application::dtos::categorias::CategoriaInput;
use eo_application::dtos::clientes::ClienteInput;
use eo_application::dtos::comercial::{AntiguedadDeudaQuery, CuentaCorrienteQuery};
use eo_application::dtos::dashboard::PeriodoDashboard;
use eo_application::dtos::facturas::{FacturaInput, PagoFacturaInput};
use eo_application::dtos::movimientos::MovimientoInput;
use eo_application::dtos::obras::ObraInput;
use eo_application::dtos::trabajos::TrabajoInput;
use eo_application::ports::repositories::UnitOfWork;
use eo_application::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use eo_application::use_cases::categorias::CategoriasService;
use eo_application::use_cases::clientes::ClientesService;
use eo_application::use_cases::comercial::ComercialService;
use eo_application::use_cases::dashboard::DashboardService;
use eo_application::use_cases::facturas::FacturasService;
use eo_application::use_cases::movimientos::MovimientosService;
use eo_application::use_cases::obras::ObrasService;
use eo_application::use_cases::trabajos::TrabajosService;
use eo_domain::clock::FixedClock;
use eo_domain::constants::tipos_movimiento;
use eo_domain::ids::UuidV7Generator;
use eo_domain::{Decimal4, EstadoFactura, Moneda, Money};
use eo_infrastructure::config::FileSettingsStore;
use eo_infrastructure::persistence::DbHandle;
use eo_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use pretty_assertions::assert_eq;
use uuid::Uuid;

/// Frozen "now". Every window and every ageing bucket in this file is counted from here.
fn ahora() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 29, 12, 0, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

fn hoy() -> NaiveDate {
    ahora().date_naive()
}

/// A date `dias` before "now", which is how the tests place an invoice in a bucket.
fn hace(dias: i64) -> NaiveDate {
    hoy() - chrono::Duration::days(dias)
}

fn instante(dias: i64) -> DateTime<Utc> {
    ahora() - chrono::Duration::days(dias)
}

struct Fixture {
    dashboard: DashboardService,
    comercial: ComercialService,
    movimientos: MovimientosService,
    categorias: CategoriasService,
    clientes: ClientesService,
    obras: ObrasService,
    trabajos: TrabajosService,
    facturas: FacturasService,
}

async fn fixture() -> Fixture {
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
        obras: ObrasService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
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
    async fn categoria(&self, nombre: &str) -> Uuid {
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

    async fn cliente(&self, nombre: &str) -> Uuid {
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

    async fn obra(&self, numero: i32, nombre: &str, cliente_id: Uuid) -> Uuid {
        self.obras
            .create(ObraInput {
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

    async fn trabajo(&self, obra_id: Uuid, descripcion: &str) -> Uuid {
        self.trabajos
            .create(TrabajoInput {
                obra_id,
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
    async fn movimiento(
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
    async fn factura(&self, numero: &str, cliente_id: Uuid, total: &str, dias_atras: i64) -> Uuid {
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
    async fn factura_vencida(
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

    async fn pagar(&self, factura_id: Uuid, monto: &str) {
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

#[tokio::test]
async fn los_kpis_agregan_solo_la_ventana_del_periodo() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;

    // Inside the monthly window.
    f.movimiento(
        "Cobro",
        "1000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        None,
    )
    .await;
    f.movimiento(
        "Compra",
        "300.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        None,
    )
    .await;
    // Inside the previous window, not the current one.
    f.movimiento(
        "Cobro viejo",
        "500.0000",
        "1.0000",
        true,
        45,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.total_ingresos.to_decimal_string(), "1000.0000");
    assert_eq!(stats.total_gastos.to_decimal_string(), "300.0000");
    assert_eq!(stats.balance.to_decimal_string(), "700.0000");
    assert_eq!(stats.cantidad_movimientos, 2);
    // The older income is the basis of the comparison, not part of the total.
    assert_eq!(stats.anterior_ingresos.to_decimal_string(), "500.0000");
    assert_eq!(
        stats.variacion_ingresos,
        Some(Decimal4::parse("100.0").unwrap())
    );
}

#[tokio::test]
async fn el_periodo_total_no_publica_comparacion() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Cobro",
        "1000.0000",
        "1.0000",
        true,
        400,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Total).await.unwrap();

    // A movement more than a year old is still counted by `Total`.
    assert_eq!(stats.total_ingresos.to_decimal_string(), "1000.0000");
    assert_eq!(stats.variacion_ingresos, None);
    assert_eq!(stats.variacion_gastos, None);
}

#[tokio::test]
async fn el_total_del_periodo_usa_el_producto_y_no_el_monto_unitario() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Cable",
        "1500.5000",
        "2.0000",
        false,
        3,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();
    assert_eq!(stats.total_gastos.to_decimal_string(), "3001.0000");
}

#[tokio::test]
async fn la_serie_mensual_trae_los_doce_meses() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Cobro",
        "800.0000",
        "1.0000",
        true,
        3,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.serie_mensual.len(), 12);
    assert_eq!(stats.serie_mensual[0].mes, 1);
    let agosto = &stats.serie_mensual[7];
    assert_eq!(agosto.ingresos.to_decimal_string(), "800.0000");
    // An empty month is zero, not a gap the chart has to invent.
    assert_eq!(stats.serie_mensual[0].ingresos, Money::ZERO);
}

#[tokio::test]
async fn el_top_de_clientes_agrupa_por_id_y_solo_cuenta_ingresos() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let uno = f.cliente("Cliente Uno").await;
    let dos = f.cliente("Cliente Dos").await;

    f.movimiento(
        "A",
        "1000.0000",
        "1.0000",
        true,
        3,
        Some(categoria),
        Some(uno),
        None,
    )
    .await;
    f.movimiento(
        "B",
        "400.0000",
        "1.0000",
        true,
        4,
        Some(categoria),
        Some(dos),
        None,
    )
    .await;
    // An expense charged to the customer must not inflate their billing.
    f.movimiento(
        "C",
        "9000.0000",
        "1.0000",
        false,
        4,
        Some(categoria),
        Some(dos),
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.top_clientes.len(), 2);
    assert_eq!(stats.top_clientes[0].id, Some(uno));
    assert_eq!(stats.top_clientes[0].total.to_decimal_string(), "1000.0000");
    assert_eq!(stats.top_clientes[1].total.to_decimal_string(), "400.0000");
    assert_eq!(stats.clientes_activos, 2);
}

#[tokio::test]
async fn los_gastos_por_categoria_ordenan_de_mayor_a_menor() {
    let f = fixture().await;
    let materiales = f.categoria("Materiales").await;
    let combustible = f.categoria("Combustible").await;

    f.movimiento(
        "A",
        "100.0000",
        "1.0000",
        false,
        2,
        Some(materiales),
        None,
        None,
    )
    .await;
    f.movimiento(
        "B",
        "700.0000",
        "1.0000",
        false,
        2,
        Some(combustible),
        None,
        None,
    )
    .await;
    f.movimiento(
        "C",
        "999.0000",
        "1.0000",
        true,
        2,
        Some(materiales),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.gastos_por_categoria.len(), 2);
    assert_eq!(stats.gastos_por_categoria[0].nombre, "Combustible");
    assert_eq!(
        stats.gastos_por_categoria[0].total.to_decimal_string(),
        "700.0000"
    );
}

#[tokio::test]
async fn la_rentabilidad_por_obra_imputa_a_traves_del_trabajo() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    let obra = f.obra(1, "Edificio Norte", cliente).await;
    let trabajo = f.trabajo(obra, "Tablero").await;

    f.movimiento(
        "Cobro",
        "3000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        Some(trabajo),
    )
    .await;
    f.movimiento(
        "Gasto",
        "2000.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        Some(trabajo),
    )
    .await;
    // Without a job the movement is imputed to no site at all.
    f.movimiento(
        "Suelto",
        "5000.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        None,
    )
    .await;

    let ranking = f.comercial.rentabilidad_obras(None).await.unwrap();

    assert_eq!(ranking.len(), 1);
    let fila = &ranking[0];
    assert_eq!(fila.ingresos.to_decimal_string(), "3000.0000");
    assert_eq!(fila.gastos.to_decimal_string(), "2000.0000");
    assert_eq!(fila.rentabilidad.to_decimal_string(), "1000.0000");
    assert_eq!(fila.margen_porcentaje, Decimal4::parse("33.33").unwrap());
}

#[tokio::test]
async fn una_obra_sin_ingresos_da_margen_cero_y_no_divide_por_cero() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    let obra = f.obra(1, "Solo gastos", cliente).await;
    let trabajo = f.trabajo(obra, "Zanjeo").await;

    f.movimiento(
        "Gasto",
        "1500.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        Some(trabajo),
    )
    .await;

    let ranking = f.comercial.rentabilidad_obras(None).await.unwrap();

    assert_eq!(ranking[0].rentabilidad.to_decimal_string(), "-1500.0000");
    assert_eq!(ranking[0].margen_porcentaje, Decimal4::ZERO);
}

#[tokio::test]
async fn la_rentabilidad_por_trabajo_se_puede_filtrar_por_obra() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    let obra_a = f.obra(1, "Obra A", cliente).await;
    let obra_b = f.obra(2, "Obra B", cliente).await;
    let trabajo_a = f.trabajo(obra_a, "Tablero A").await;
    let trabajo_b = f.trabajo(obra_b, "Tablero B").await;

    f.movimiento(
        "Cobro A",
        "1000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        Some(trabajo_a),
    )
    .await;
    f.movimiento(
        "Cobro B",
        "2000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        Some(trabajo_b),
    )
    .await;

    let todos = f.comercial.rentabilidad_trabajos(None, None).await.unwrap();
    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].nombre, "Tablero B");
    assert_eq!(todos[0].contexto, "Obra B");

    let solo_a = f
        .comercial
        .rentabilidad_trabajos(Some(obra_a), None)
        .await
        .unwrap();
    assert_eq!(solo_a.len(), 1);
    assert_eq!(solo_a[0].id, trabajo_a);
}

#[tokio::test]
async fn la_cuenta_corriente_deriva_el_saldo_y_la_mora() {
    let f = fixture().await;
    let cliente = f.cliente("Deudor").await;
    let factura = f.factura_vencida("0001", cliente, "1000.0000", 45).await;
    f.pagar(factura, "400.0000").await;

    let cuenta = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: cliente,
            incluir_pagadas: false,
        })
        .await
        .unwrap();

    assert_eq!(cuenta.cliente_nombre, "Deudor");
    assert_eq!(cuenta.facturas.len(), 1);
    assert_eq!(cuenta.saldo.to_decimal_string(), "600.0000");
    assert_eq!(cuenta.total_facturado.to_decimal_string(), "1000.0000");
    assert_eq!(cuenta.total_pagado.to_decimal_string(), "400.0000");
    assert_eq!(cuenta.facturas[0].dias_mora, 45);
}

#[tokio::test]
async fn una_factura_saldada_sale_de_la_cuenta_corriente() {
    let f = fixture().await;
    let cliente = f.cliente("Al día").await;
    let factura = f.factura("0002", cliente, "500.0000", 10).await;
    f.pagar(factura, "500.0000").await;

    let cuenta = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: cliente,
            incluir_pagadas: false,
        })
        .await
        .unwrap();
    assert!(cuenta.facturas.is_empty());
    assert_eq!(cuenta.saldo, Money::ZERO);

    // Asked for explicitly, it comes back with no arrears.
    let con_pagadas = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: cliente,
            incluir_pagadas: true,
        })
        .await
        .unwrap();
    assert_eq!(con_pagadas.facturas.len(), 1);
    assert_eq!(con_pagadas.facturas[0].dias_mora, 0);
}

#[tokio::test]
async fn un_cliente_inexistente_da_una_cuenta_vacia_y_no_un_error() {
    let f = fixture().await;

    let cuenta = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: Uuid::nil(),
            incluir_pagadas: false,
        })
        .await
        .unwrap();

    assert_eq!(cuenta.cliente_id, Uuid::nil());
    assert_eq!(cuenta.cliente_nombre, "");
    assert!(cuenta.facturas.is_empty());
}

#[tokio::test]
async fn los_bordes_de_los_buckets_caen_en_la_columna_documentada() {
    let f = fixture().await;
    let cliente = f.cliente("Deudor").await;

    // One invoice per boundary, each of 100, so the column it lands in is unmistakable.
    for (i, dias) in [30_i64, 31, 60, 61, 90, 91].into_iter().enumerate() {
        f.factura_vencida(&format!("B{i}"), cliente, "100.0000", dias)
            .await;
    }

    let aging = f
        .comercial
        .antiguedad_deuda(AntiguedadDeudaQuery {
            fecha_corte: None,
            cliente_id: None,
        })
        .await
        .unwrap();

    assert_eq!(aging.bucket0a30.to_decimal_string(), "100.0000");
    assert_eq!(aging.bucket31a60.to_decimal_string(), "200.0000");
    assert_eq!(aging.bucket61a90.to_decimal_string(), "200.0000");
    assert_eq!(aging.bucket_mas90.to_decimal_string(), "100.0000");
    assert_eq!(aging.total.to_decimal_string(), "600.0000");

    // The invariant the report lives by.
    let suma = Money::try_sum([
        aging.bucket0a30,
        aging.bucket31a60,
        aging.bucket61a90,
        aging.bucket_mas90,
    ])
    .unwrap();
    assert_eq!(suma, aging.total);
    assert_eq!(aging.limites, vec![30, 60, 90]);
}

#[tokio::test]
async fn la_antiguedad_desglosa_por_cliente_y_respeta_la_fecha_de_corte() {
    let f = fixture().await;
    let uno = f.cliente("Uno").await;
    let dos = f.cliente("Dos").await;
    f.factura_vencida("0001", uno, "1000.0000", 20).await;
    f.factura_vencida("0002", dos, "300.0000", 20).await;

    let aging = f
        .comercial
        .antiguedad_deuda(AntiguedadDeudaQuery {
            fecha_corte: None,
            cliente_id: None,
        })
        .await
        .unwrap();
    assert_eq!(aging.detalle.len(), 2);
    // Sorted by how much each one owes.
    assert_eq!(aging.detalle[0].cliente_id, uno);
    assert_eq!(aging.detalle[0].bucket0a30.to_decimal_string(), "1000.0000");

    // Moving the cut-off date forward ages the debt into the next bucket.
    let corrido = f
        .comercial
        .antiguedad_deuda(AntiguedadDeudaQuery {
            fecha_corte: Some(hoy() + chrono::Duration::days(15)),
            cliente_id: Some(uno),
        })
        .await
        .unwrap();
    assert_eq!(corrido.bucket0a30, Money::ZERO);
    assert_eq!(corrido.bucket31a60.to_decimal_string(), "1000.0000");
    assert_eq!(corrido.detalle.len(), 1);
}

#[tokio::test]
async fn las_facturas_vencidas_exigen_saldo_pendiente() {
    let f = fixture().await;
    let cliente = f.cliente("Cliente").await;
    // Older than the thirty-day threshold, so it counts.
    let vieja = f.factura("0001", cliente, "1000.0000", 60).await;
    // Recent: not overdue yet.
    f.factura("0002", cliente, "500.0000", 5).await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();
    assert_eq!(stats.facturas_vencidas, 1);

    // Once collected it stops being a debt even though its date did not change.
    f.pagar(vieja, "1000.0000").await;
    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();
    assert_eq!(stats.facturas_vencidas, 0);
}

#[tokio::test]
async fn las_alertas_llevan_su_destino_con_el_filtro_aplicado() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    f.factura("0001", cliente, "1000.0000", 60).await;
    // A negative balance raises the error-level alert.
    f.movimiento(
        "Gasto",
        "5000.0000",
        "1.0000",
        false,
        3,
        Some(categoria),
        None,
        None,
    )
    .await;

    let alertas = f
        .dashboard
        .alertas(PeriodoDashboard::Mensual)
        .await
        .unwrap();

    let vencidas = alertas
        .iter()
        .find(|a| a.clave == "Dashboard.Alerta.FacturasVencidas")
        .unwrap();
    assert_eq!(vencidas.cantidad, 1);
    assert_eq!(vencidas.destino, "/facturas?estado=vencida");

    let balance = alertas
        .iter()
        .find(|a| a.clave == "Dashboard.Alerta.BalanceNegativo")
        .unwrap();
    assert_eq!(
        balance.monto.map(|m| m.to_decimal_string()),
        Some("-5000.0000".to_owned())
    );
}

#[tokio::test]
async fn el_estado_del_sistema_informa_la_base() {
    let f = fixture().await;
    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert!(stats.estado_sistema.base_saludable);
    assert_eq!(stats.estado_sistema.estado, "Dashboard.Estado.Saludable");
    assert!(stats.estado_sistema.migraciones > 0);
    assert!(stats.estado_sistema.tamano_bytes > 0);
}

#[tokio::test]
async fn los_ultimos_movimientos_vienen_del_mas_nuevo_al_mas_viejo() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Viejo",
        "100.0000",
        "1.0000",
        false,
        20,
        Some(categoria),
        None,
        None,
    )
    .await;
    f.movimiento(
        "Nuevo",
        "200.0000",
        "1.0000",
        false,
        1,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.ultimos_movimientos.len(), 2);
    assert_eq!(stats.ultimos_movimientos[0].concepto, "Nuevo");
}
