//! Use cases for Authentication, RBAC, Sessions, and 2FA.

use std::sync::Arc;

use crate::ports::auth::{PasswordHasher, TokenPort, TotpPort};
use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;
use crate::ports::settings::SettingsStore;

mod rbac;
mod session;
mod usuarios;

pub struct AuthService {
    pub(crate) uow: Arc<dyn UnitOfWork>,
    pub(crate) clock: Arc<dyn ClockPort>,
    pub(crate) ids: Arc<dyn IdGeneratorPort>,
    #[allow(dead_code)]
    pub(crate) settings: Arc<dyn SettingsStore>,
    pub(crate) hasher: Arc<dyn PasswordHasher>,
    pub(crate) tokens: Arc<dyn TokenPort>,
    pub(crate) totp: Arc<dyn TotpPort>,
}

impl AuthService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
        settings: Arc<dyn SettingsStore>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenPort>,
        totp: Arc<dyn TotpPort>,
    ) -> Self {
        Self {
            uow,
            clock,
            ids,
            settings,
            hasher,
            tokens,
            totp,
        }
    }
}
