//! `docs/08-maquinas-de-estado.md` §4.

use crate::enums::EstadoTrabajo;
use crate::state::StateMachine;

impl StateMachine for EstadoTrabajo {
    const ENTITY: &'static str = "Trabajo";

    /// `Presupuestado → Finalizado` is absent deliberately: something that never started cannot be
    /// finished, and a job that was billed without being tracked has to be walked through
    /// `EnProceso` so the record says what happened.
    fn allowed_targets(self) -> &'static [Self] {
        use EstadoTrabajo::*;
        match self {
            Presupuestado => &[EnProceso, Cancelado],
            EnProceso => &[Pausado, Finalizado, Cancelado],
            Pausado => &[EnProceso, Finalizado, Cancelado],
            Finalizado => &[EnProceso],
            Cancelado => &[Presupuestado],
        }
    }

    fn as_key(self) -> &'static str {
        match self {
            Self::Presupuestado => "Presupuestado",
            Self::EnProceso => "EnProceso",
            Self::Pausado => "Pausado",
            Self::Finalizado => "Finalizado",
            Self::Cancelado => "Cancelado",
        }
    }
}

impl EstadoTrabajo {
    /// Starting, resuming and reopening all require a running site (T-T02, T-T05, T-T10).
    /// Closing a job never does: shutting things down is always allowed.
    pub const fn exige_proyecto_activo(destino: Self, origen: Self) -> bool {
        matches!(destino, Self::EnProceso)
            && matches!(
                origen,
                Self::Presupuestado | Self::Pausado | Self::Finalizado
            )
    }

    pub const fn requiere_confirmacion_desde(self, destino: Self) -> bool {
        matches!(
            (self, destino),
            (Self::Finalizado, Self::EnProceso) | (Self::Cancelado, Self::Presupuestado)
        )
    }
}
