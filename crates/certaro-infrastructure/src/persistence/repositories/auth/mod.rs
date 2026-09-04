//! Repositories for Auth, Users, Roles, Permissions, Sessions, and External SSO.

mod auth_externo;
mod permiso;
mod rol;
mod sesion;
mod usuario;

pub use auth_externo::SeaOrmAuthExternoRepository;
pub use permiso::SeaOrmPermisoRepository;
pub use rol::SeaOrmRolRepository;
pub use sesion::SeaOrmSesionRepository;
pub use usuario::SeaOrmUsuarioRepository;
