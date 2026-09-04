use super::AppConfig;
use crate::error::{AppError, FieldError};
use crate::paging::PageRequest;

pub(super) fn validate_config(config: &AppConfig) -> Result<(), AppError> {
    let mut errors = Vec::new();
    if !PageRequest::ALLOWED_SIZES.contains(&config.application.last_page_size) {
        errors.push(FieldError::new(
            "application.lastPageSize",
            "Validation.Config.ValorNoPermitido",
        ));
    }
    if !matches!(config.locale.language.as_str(), "es" | "en") {
        errors.push(FieldError::new(
            "locale.language",
            "Validation.Config.ValorNoPermitido",
        ));
    }
    if config.locale.zona_horaria.parse::<chrono_tz::Tz>().is_err() {
        errors.push(FieldError::new(
            "locale.zonaHoraria",
            "Validation.Config.ValorNoPermitido",
        ));
    }
    if config.locale.decimales_moneda > 4 {
        errors.push(
            FieldError::new(
                "locale.decimalesMoneda",
                "Validation.Config.ValorNoPermitido",
            )
            .with_param("max", 4),
        );
    }
    if config.external_apis.timeout_seconds == 0 {
        errors.push(FieldError::new(
            "externalApis.timeoutSeconds",
            "Validation.Config.ValorNoPermitido",
        ));
    }
    if config.attachments.max_size_mb == 0
        || config.attachments.max_size_mb > config.attachments.max_total_mb
    {
        errors.push(FieldError::new(
            "attachments.maxSizeMb",
            "Validation.Config.ValorNoPermitido",
        ));
    }
    if config.database.max_connections == 0 {
        errors.push(FieldError::new(
            "database.maxConnections",
            "Validation.Config.ValorNoPermitido",
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errors))
    }
}
