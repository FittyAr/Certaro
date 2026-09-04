use chrono::{DateTime, Months, NaiveDate, TimeZone, Utc};
use tracing::warn;

use crate::dtos::dashboard::PeriodoDashboard;

/// Where `Total` starts counting from. The validation floor for any date is the year 2000, so this
/// is safely before every record and, unlike `DateTime::MIN_UTC`, it still formats as a normal
/// timestamp for the comparison the query does.
pub const COMIENZO_DE_LOS_TIEMPOS: i32 = 1900;

/// The current window and the one it is compared against. `Total` has no previous window, and its
/// comparison is reported as absent rather than as zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ventanas {
    pub desde: DateTime<Utc>,
    pub hasta: DateTime<Utc>,
    pub anterior: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

/// Rolling windows, not calendar months: "monthly" means the last thirty-odd days, which is
/// what the legacy `AddMonths(-1)` did and what the user reads on the card.
///
/// The previous window is the current one shifted back by its own length rather than another
/// `AddMonths`. Stepping back two calendar months would compare 31 days against 30 and report
/// a three-percent drop that is only the calendar; doc 17 §3.4 requires the two windows to
/// span the same number of days.
pub fn calcular_ventanas(periodo: PeriodoDashboard, ahora: DateTime<Utc>) -> Ventanas {
    let atras = |meses: u32| {
        ahora
            .checked_sub_months(Months::new(meses))
            .unwrap_or(ahora)
    };
    let con_anterior = |desde: DateTime<Utc>| Ventanas {
        desde,
        hasta: ahora,
        anterior: Some((desde - (ahora - desde), desde)),
    };

    match periodo {
        PeriodoDashboard::Mensual => con_anterior(atras(1)),
        PeriodoDashboard::Anual => con_anterior(atras(12)),
        PeriodoDashboard::Total => Ventanas {
            desde: Utc
                .with_ymd_and_hms(COMIENZO_DE_LOS_TIEMPOS, 1, 1, 0, 0, 0)
                .single()
                .unwrap_or(ahora),
            hasta: ahora,
            anterior: None,
        },
    }
}

/// Invoices issued on or before this date and still unpaid count as overdue.
pub fn umbral_vencimiento(hoy: NaiveDate, dias: u32) -> NaiveDate {
    hoy.checked_sub_days(chrono::Days::new(u64::from(dias)))
        .unwrap_or_else(|| {
            warn!(dias, "el umbral de vencimiento se sale del calendario");
            hoy
        })
}
