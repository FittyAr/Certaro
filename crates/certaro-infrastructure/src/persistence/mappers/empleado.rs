use certaro_application::AppError;
use certaro_domain::entities::Empleado;
use certaro_domain::{time, Decimal4, FrecuenciaPago, Money};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::empleado::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<Empleado, AppError> {
    Ok(Empleado {
        id: mappers::uuid(&model.id)?,
        nombre: model.nombre,
        dni: model.dni,
        cargo: model.cargo,
        sueldo_base: Money::from_raw(model.sueldo_base),
        pago_frecuencia: FrecuenciaPago::from_i32(model.pago_frecuencia).map_err(AppError::from)?,
        tarifa_diaria: Money::from_raw(model.tarifa_diaria),
        multiplicador_sabado: Decimal4::from_raw(model.multiplicador_sabado),
        multiplicador_domingo: Decimal4::from_raw(model.multiplicador_domingo),
        multiplicador_feriado: Decimal4::from_raw(model.multiplicador_feriado),
        email: model.email,
        telefono: model.telefono,
        fecha_ingreso: mappers::civil(&model.fecha_ingreso)?,
        fecha_egreso: mappers::civil_opt(model.fecha_egreso.as_deref())?,
        activo: model.activo,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Empleado) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        nombre: Set(entity.nombre.clone()),
        dni: Set(entity.dni.clone()),
        cargo: Set(entity.cargo.clone()),
        sueldo_base: Set(entity.sueldo_base.raw()),
        pago_frecuencia: Set(entity.pago_frecuencia.as_i32()),
        tarifa_diaria: Set(entity.tarifa_diaria.raw()),
        multiplicador_sabado: Set(entity.multiplicador_sabado.raw()),
        multiplicador_domingo: Set(entity.multiplicador_domingo.raw()),
        multiplicador_feriado: Set(entity.multiplicador_feriado.raw()),
        email: Set(entity.email.clone()),
        telefono: Set(entity.telefono.clone()),
        fecha_ingreso: Set(mappers::civil_to_storage(entity.fecha_ingreso)),
        fecha_egreso: Set(entity.fecha_egreso.map(mappers::civil_to_storage)),
        activo: Set(entity.activo),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
