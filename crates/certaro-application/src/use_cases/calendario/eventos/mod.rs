use uuid::Uuid;
use certaro_domain::RowVersion;

use crate::dtos::calendario::{ActualizarEventoInput, CalendarioEventoDto, CrearEventoInput};
use crate::result::AppResult;

use super::CalendarioService;

mod mutation;
mod query;

impl CalendarioService {
    pub async fn list_eventos(
        &self,
        desde_iso: &str,
        hasta_iso: &str,
    ) -> AppResult<Vec<CalendarioEventoDto>> {
        query::list_eventos_impl(self.uow.as_ref(), self.id_gen.as_ref(), desde_iso, hasta_iso).await
    }

    pub async fn create_evento(&self, input: CrearEventoInput) -> AppResult<CalendarioEventoDto> {
        mutation::create_evento_impl(
            self.uow.as_ref(),
            self.clock.as_ref(),
            self.id_gen.as_ref(),
            input,
        )
        .await
    }

    pub async fn update_evento(
        &self,
        id: Uuid,
        input: ActualizarEventoInput,
    ) -> AppResult<CalendarioEventoDto> {
        mutation::update_evento_impl(
            self.uow.as_ref(),
            self.clock.as_ref(),
            id,
            input,
        )
        .await
    }

    pub async fn mover_evento(
        &self,
        id: Uuid,
        nuevo_inicio: &str,
        nuevo_fin: &str,
        row_version: RowVersion,
    ) -> AppResult<()> {
        mutation::mover_evento_impl(
            self.uow.as_ref(),
            self.clock.as_ref(),
            id,
            nuevo_inicio,
            nuevo_fin,
            row_version,
        )
        .await
    }

    pub async fn delete_evento(&self, id: Uuid, row_version: RowVersion) -> AppResult<()> {
        mutation::delete_evento_impl(self.uow.as_ref(), id, row_version).await
    }
}
