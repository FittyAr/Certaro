use certaro_application::AppError;
use certaro_domain::entities::{Liquidacion, LiquidacionAdelanto, ReglasLiquidacion};
use certaro_domain::{time, Decimal4, Money};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::liquidacion::{ActiveModel, Model};
use crate::persistence::models::liquidacion_adelanto::{
    ActiveModel as AdelantoActiveModel, Model as AdelantoModel,
};

pub fn to_domain(model: Model) -> Result<Liquidacion, AppError> {
    Ok(Liquidacion {
        id: mappers::uuid(&model.id)?,
        empleado_id: mappers::uuid(&model.empleado_id)?,
        fecha_inicio: mappers::civil(&model.fecha_inicio)?,
        fecha_fin: mappers::civil(&model.fecha_fin)?,
        dias_trabajados: Decimal4::from_raw(model.dias_trabajados),
        tarifa_aplicada: Money::from_raw(model.tarifa_aplicada),
        reglas: ReglasLiquidacion {
            incluir_sabados: model.incluir_sabados,
            incluir_domingos: model.incluir_domingos,
            incluir_feriados: model.incluir_feriados,
            multiplicador_sabado: Decimal4::from_raw(model.multiplicador_sabado),
            multiplicador_domingo: Decimal4::from_raw(model.multiplicador_domingo),
            multiplicador_feriado: Decimal4::from_raw(model.multiplicador_feriado),
        },
        total_bruto: Money::from_raw(model.total_bruto),
        total_adelantos: Money::from_raw(model.total_adelantos),
        observaciones: model.observaciones,
        pdf_generado_at: mappers::instant_opt(model.pdf_generado_at.as_deref())?,
        adelantos: Vec::new(),
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Liquidacion) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        empleado_id: Set(entity.empleado_id.to_string()),
        fecha_inicio: Set(mappers::civil_to_storage(entity.fecha_inicio)),
        fecha_fin: Set(mappers::civil_to_storage(entity.fecha_fin)),
        dias_trabajados: Set(entity.dias_trabajados.raw()),
        tarifa_aplicada: Set(entity.tarifa_aplicada.raw()),
        incluir_sabados: Set(entity.reglas.incluir_sabados),
        incluir_domingos: Set(entity.reglas.incluir_domingos),
        incluir_feriados: Set(entity.reglas.incluir_feriados),
        multiplicador_sabado: Set(entity.reglas.multiplicador_sabado.raw()),
        multiplicador_domingo: Set(entity.reglas.multiplicador_domingo.raw()),
        multiplicador_feriado: Set(entity.reglas.multiplicador_feriado.raw()),
        total_bruto: Set(entity.total_bruto.raw()),
        total_adelantos: Set(entity.total_adelantos.raw()),
        observaciones: Set(entity.observaciones.clone()),
        pdf_generado_at: Set(entity.pdf_generado_at.map(time::to_storage)),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn adelanto_to_domain(model: AdelantoModel) -> Result<LiquidacionAdelanto, AppError> {
    Ok(LiquidacionAdelanto {
        id: mappers::uuid(&model.id)?,
        liquidacion_id: mappers::uuid(&model.liquidacion_id)?,
        movimiento_id: mappers::uuid(&model.movimiento_id)?,
        monto: Money::from_raw(model.monto),
        fecha: mappers::civil(&model.fecha)?,
        concepto: model.concepto,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn adelanto_to_active(entity: &LiquidacionAdelanto) -> AdelantoActiveModel {
    AdelantoActiveModel {
        id: Set(entity.id.to_string()),
        liquidacion_id: Set(entity.liquidacion_id.to_string()),
        movimiento_id: Set(entity.movimiento_id.to_string()),
        monto: Set(entity.monto.raw()),
        fecha: Set(mappers::civil_to_storage(entity.fecha)),
        concepto: Set(entity.concepto.clone()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
