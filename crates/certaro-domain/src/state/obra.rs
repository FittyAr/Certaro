//! `docs/08-maquinas-de-estado.md` §3.

use crate::enums::EstadoObra;
use crate::state::StateMachine;

impl StateMachine for EstadoObra {
    const ENTITY: &'static str = "Obra";

    /// No state is terminal: a finished site gets reopened often enough that forbidding it would
    /// only teach the user to create a duplicate. What is forbidden is going sideways —
    /// `Finalizada → Cancelada` and the like have to pass through `Activa`, which forces the
    /// person to state that the site is running again before changing how it ended.
    fn allowed_targets(self) -> &'static [Self] {
        use EstadoObra::*;
        match self {
            Activa => &[Pausada, Finalizada, Cancelada],
            Pausada => &[Activa, Finalizada, Cancelada],
            Finalizada => &[Activa],
            Cancelada => &[Activa],
        }
    }

    fn as_key(self) -> &'static str {
        match self {
            Self::Activa => "Activa",
            Self::Pausada => "Pausada",
            Self::Finalizada => "Finalizada",
            Self::Cancelada => "Cancelada",
        }
    }
}

impl EstadoObra {
    /// Reopening something already closed is easy to do by accident from a grid, so the interface
    /// asks first.
    pub const fn requiere_confirmacion_desde(self, destino: Self) -> bool {
        matches!(self, Self::Finalizada | Self::Cancelada) && matches!(destino, Self::Activa)
    }
}
