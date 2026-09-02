use certaro_application::AppError;
use certaro_domain::entities::{AuthExterno, Permiso, Rol, RolPermiso, Sesion, Usuario, UsuarioRol};
use certaro_domain::time;
use sea_orm::Set;

use super::{audit, instant, instant_opt, uuid};
use crate::persistence::models::{
    auth_externo, permiso, rol, rol_permiso, sesion, usuario, usuario_rol,
};

pub fn usuario_to_domain(m: usuario::Model) -> Result<Usuario, AppError> {
    Ok(Usuario {
        id: uuid(&m.id)?,
        email: m.email,
        nombre_completo: m.nombre_completo,
        password_hash: m.password_hash,
        activo: m.activo,
        requiere_2fa: m.requiere_2fa,
        totp_secret: m.totp_secret,
        ultimo_login: instant_opt(m.ultimo_login.as_deref())?,
        intentos_fallidos: m.intentos_fallidos as u32,
        bloqueado_hasta: instant_opt(m.bloqueado_hasta.as_deref())?,
        audit: audit(
            &m.created_at,
            m.updated_at.as_deref(),
            &m.row_version,
            m.is_deleted,
            m.deleted_at.as_deref(),
        )?,
    })
}

pub fn usuario_to_active(e: &Usuario) -> usuario::ActiveModel {
    usuario::ActiveModel {
        id: Set(e.id.to_string()),
        email: Set(e.email.clone()),
        nombre_completo: Set(e.nombre_completo.clone()),
        password_hash: Set(e.password_hash.clone()),
        activo: Set(e.activo),
        requiere_2fa: Set(e.requiere_2fa),
        totp_secret: Set(e.totp_secret.clone()),
        ultimo_login: Set(e.ultimo_login.map(time::to_storage)),
        intentos_fallidos: Set(e.intentos_fallidos as i32),
        bloqueado_hasta: Set(e.bloqueado_hasta.map(time::to_storage)),
        created_at: Set(time::to_storage(e.audit.created_at)),
        updated_at: Set(e.audit.updated_at.map(time::to_storage)),
        row_version: Set(e.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(e.audit.is_deleted),
        deleted_at: Set(e.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn rol_to_domain(m: rol::Model) -> Result<Rol, AppError> {
    Ok(Rol {
        id: uuid(&m.id)?,
        nombre: m.nombre,
        descripcion: m.descripcion,
        es_sistema: m.es_sistema,
        prioridad: m.prioridad,
        audit: audit(
            &m.created_at,
            m.updated_at.as_deref(),
            &m.row_version,
            m.is_deleted,
            m.deleted_at.as_deref(),
        )?,
    })
}

pub fn rol_to_active(e: &Rol) -> rol::ActiveModel {
    rol::ActiveModel {
        id: Set(e.id.to_string()),
        nombre: Set(e.nombre.clone()),
        descripcion: Set(e.descripcion.clone()),
        es_sistema: Set(e.es_sistema),
        prioridad: Set(e.prioridad),
        created_at: Set(time::to_storage(e.audit.created_at)),
        updated_at: Set(e.audit.updated_at.map(time::to_storage)),
        row_version: Set(e.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(e.audit.is_deleted),
        deleted_at: Set(e.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn permiso_to_domain(m: permiso::Model) -> Result<Permiso, AppError> {
    Ok(Permiso {
        id: uuid(&m.id)?,
        modulo: m.modulo,
        accion: m.accion,
        recurso: m.recurso,
        clave: m.clave,
    })
}

pub fn permiso_to_active(e: &Permiso) -> permiso::ActiveModel {
    permiso::ActiveModel {
        id: Set(e.id.to_string()),
        modulo: Set(e.modulo.clone()),
        accion: Set(e.accion.clone()),
        recurso: Set(e.recurso.clone()),
        clave: Set(e.clave.clone()),
    }
}

pub fn usuario_rol_to_domain(m: usuario_rol::Model) -> Result<UsuarioRol, AppError> {
    Ok(UsuarioRol {
        id: uuid(&m.id)?,
        usuario_id: uuid(&m.usuario_id)?,
        rol_id: uuid(&m.rol_id)?,
        audit: audit(
            &m.created_at,
            m.updated_at.as_deref(),
            &m.row_version,
            m.is_deleted,
            m.deleted_at.as_deref(),
        )?,
    })
}

pub fn usuario_rol_to_active(e: &UsuarioRol) -> usuario_rol::ActiveModel {
    usuario_rol::ActiveModel {
        id: Set(e.id.to_string()),
        usuario_id: Set(e.usuario_id.to_string()),
        rol_id: Set(e.rol_id.to_string()),
        created_at: Set(time::to_storage(e.audit.created_at)),
        updated_at: Set(e.audit.updated_at.map(time::to_storage)),
        row_version: Set(e.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(e.audit.is_deleted),
        deleted_at: Set(e.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn rol_permiso_to_domain(m: rol_permiso::Model) -> Result<RolPermiso, AppError> {
    Ok(RolPermiso {
        id: uuid(&m.id)?,
        rol_id: uuid(&m.rol_id)?,
        permiso_id: uuid(&m.permiso_id)?,
    })
}

pub fn rol_permiso_to_active(e: &RolPermiso) -> rol_permiso::ActiveModel {
    rol_permiso::ActiveModel {
        id: Set(e.id.to_string()),
        rol_id: Set(e.rol_id.to_string()),
        permiso_id: Set(e.permiso_id.to_string()),
    }
}

pub fn sesion_to_domain(m: sesion::Model) -> Result<Sesion, AppError> {
    Ok(Sesion {
        id: uuid(&m.id)?,
        usuario_id: uuid(&m.usuario_id)?,
        token_hash: m.token_hash,
        expira_en: instant(&m.expira_en)?,
        ip: m.ip,
        user_agent: m.user_agent,
        created_at: instant(&m.created_at)?,
    })
}

pub fn sesion_to_active(e: &Sesion) -> sesion::ActiveModel {
    sesion::ActiveModel {
        id: Set(e.id.to_string()),
        usuario_id: Set(e.usuario_id.to_string()),
        token_hash: Set(e.token_hash.clone()),
        expira_en: Set(time::to_storage(e.expira_en)),
        ip: Set(e.ip.clone()),
        user_agent: Set(e.user_agent.clone()),
        created_at: Set(time::to_storage(e.created_at)),
    }
}

pub fn auth_externo_to_domain(m: auth_externo::Model) -> Result<AuthExterno, AppError> {
    Ok(AuthExterno {
        id: uuid(&m.id)?,
        usuario_id: uuid(&m.usuario_id)?,
        proveedor: m.proveedor,
        proveedor_user_id: m.proveedor_user_id,
        email: m.email,
        vinculado_en: instant(&m.vinculado_en)?,
    })
}

pub fn auth_externo_to_active(e: &AuthExterno) -> auth_externo::ActiveModel {
    auth_externo::ActiveModel {
        id: Set(e.id.to_string()),
        usuario_id: Set(e.usuario_id.to_string()),
        proveedor: Set(e.proveedor.clone()),
        proveedor_user_id: Set(e.proveedor_user_id.clone()),
        email: Set(e.email.clone()),
        vinculado_en: Set(time::to_storage(e.vinculado_en)),
    }
}
