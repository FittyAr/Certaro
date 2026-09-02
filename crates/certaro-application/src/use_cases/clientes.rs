//! Use cases of `clientes`. See `docs/09-modulos-funcionales.md` §3.3.
//!
//! The customer and its contacts are one aggregate: they are validated, written and deleted in a
//! single transaction. The legacy system had the contacts table and the validator but only ever
//! edited the single `email` column, so half of RC-13 existed and never ran.

use std::sync::Arc;

use certaro_domain::entities::{Audit, Cliente, ClienteContacto};
use tracing::info;
use uuid::Uuid;

use crate::dtos::clientes::{
    ClienteContactoInput, ClienteDetalle, ClienteFiltroDto, ClienteInput, ClienteListItem,
};
use crate::dtos::common::{ListQuery, LookupItem};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{ClienteRepository, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;

const ENTITY: &str = "Cliente";

const SORTABLE: [&str; 4] = ["nombre", "cuit", "deuda", "createdAt"];

pub struct ClientesService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl ClientesService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<ClienteFiltroDto>,
    ) -> AppResult<PagedResult<ClienteListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .clientes()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(ClienteListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<ClienteDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.clientes(), id).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .clientes()
            .lookup(texto.as_deref(), limite.unwrap_or(50))
            .await;
        let clientes = finish_read(tx, result).await?;
        Ok(clientes
            .into_iter()
            .map(|c| {
                let item = LookupItem::new(c.id, c.nombre);
                match c.cuit {
                    Some(cuit) => item.with_meta("cuit", cuit),
                    None => item,
                }
            })
            .collect())
    }

    pub async fn create(&self, input: ClienteInput) -> AppResult<ClienteDetalle> {
        validation::clientes::validate(&input)?;

        let now = self.clock.now_utc();
        let id = self.ids.new_id();
        let contactos: Vec<_> = input
            .contactos
            .iter()
            .map(|c| self.build_contacto(c, id, now))
            .collect();
        let mut cliente = build_cliente(id, &input, now);
        cliente.contactos = contactos;

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.clientes();
            repo.insert(&cliente).await?;
            for contacto in &cliente.contactos {
                repo.insert_contacto(contacto).await?;
            }
            Ok(ClienteDetalle::build(&cliente, 0, 0))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, nombre = %detalle.nombre, contactos = detalle.contactos.len(), "cliente creado");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: ClienteInput,
        row_version: &str,
    ) -> AppResult<ClienteDetalle> {
        validation::clientes::validate(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.clientes();
            let existente = repo
                .find_con_contactos(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let mut cliente = build_cliente(id, &input, now);
            cliente.audit = existente.audit.clone();
            cliente.audit.touch(now);
            repo.update(&cliente, esperado).await?;

            cliente.contactos = self
                .sync_contactos(repo, id, &input.contactos, &existente.contactos, now)
                .await?;

            let proyectos = repo.count_proyectos(id).await?;
            let facturas = repo.count_facturas(id).await?;
            Ok(ClienteDetalle::build(&cliente, proyectos, facturas))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "cliente actualizado");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.clientes();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            // The foreign keys are `RESTRICT`, so both would fail at the database anyway; asking
            // first turns a constraint violation into a message that names the obstacle.
            for (count, code, key) in [
                (
                    repo.count_proyectos(id).await?,
                    "CLIENTE_CON_PROYECTOS",
                    "Conflict.Cliente.ConProyectos",
                ),
                (
                    repo.count_facturas(id).await?,
                    "CLIENTE_CON_FACTURAS",
                    "Conflict.Cliente.ConFacturas",
                ),
                (
                    repo.count_movimientos(id).await?,
                    "CLIENTE_CON_MOVIMIENTOS",
                    "Conflict.Cliente.ConMovimientos",
                ),
            ] {
                if count > 0 {
                    return Err(AppError::DependencyInUse {
                        code,
                        message_key: key,
                        params: [("count".to_owned(), count.to_string())].into(),
                    });
                }
            }

            // The contacts go with the customer: keeping them alive would leave rows the interface
            // can no longer reach and the mail action would still find.
            repo.soft_delete_contactos_excepto(id, &[], now).await?;
            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "cliente eliminado");
        Ok(())
    }

    fn build_contacto(
        &self,
        input: &ClienteContactoInput,
        cliente_id: Uuid,
        now: chrono::DateTime<chrono::Utc>,
    ) -> ClienteContacto {
        ClienteContacto {
            id: input.id.unwrap_or_else(|| self.ids.new_id()),
            cliente_id,
            etiqueta: input.etiqueta.trim().to_owned(),
            email: input.email.trim().to_lowercase(),
            nombre: normalise(input.nombre.clone()),
            telefono: normalise(input.telefono.clone()),
            es_principal: input.es_principal,
            audit: Audit::new(now),
        }
    }

    /// Applies the contact grid: rows with an id are updated, rows without one are inserted, and
    /// anything no longer listed is logically deleted.
    async fn sync_contactos(
        &self,
        repo: &dyn ClienteRepository,
        cliente_id: Uuid,
        entrantes: &[ClienteContactoInput],
        existentes: &[ClienteContacto],
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<Vec<ClienteContacto>> {
        let mut resultado = Vec::with_capacity(entrantes.len());

        for input in entrantes {
            let previo = input
                .id
                .and_then(|id| existentes.iter().find(|c| c.id == id));

            let mut contacto = self.build_contacto(input, cliente_id, now);
            match previo {
                Some(previo) => {
                    contacto.id = previo.id;
                    contacto.audit = previo.audit.clone();
                    contacto.audit.touch(now);
                    repo.update_contacto(&contacto).await?;
                }
                None => {
                    // An id that no longer exists is a stale form, not an update target: inserting
                    // under a fresh id keeps the row rather than losing the user's typing.
                    contacto.id = self.ids.new_id();
                    repo.insert_contacto(&contacto).await?;
                }
            }
            resultado.push(contacto);
        }

        let conservar: Vec<Uuid> = resultado.iter().map(|c| c.id).collect();
        repo.soft_delete_contactos_excepto(cliente_id, &conservar, now)
            .await?;

        Ok(resultado)
    }
}

fn build_cliente(id: Uuid, input: &ClienteInput, now: chrono::DateTime<chrono::Utc>) -> Cliente {
    Cliente {
        id,
        nombre: input.nombre.trim().to_owned(),
        cuit: normalise(input.cuit.clone()),
        direccion: normalise(input.direccion.clone()),
        telefono: normalise(input.telefono.clone()),
        email: normalise(input.email.clone()).map(|e| e.to_lowercase()),
        condicion_iva: normalise(input.condicion_iva.clone()),
        contactos: Vec::new(),
        audit: Audit::new(now),
    }
}

async fn load_detalle(repo: &dyn ClienteRepository, id: Uuid) -> AppResult<ClienteDetalle> {
    let cliente = repo
        .find_con_contactos(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let proyectos = repo.count_proyectos(id).await?;
    let facturas = repo.count_facturas(id).await?;
    Ok(ClienteDetalle::build(&cliente, proyectos, facturas))
}
