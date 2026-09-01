//! The result alias used across the application layer.
//!
//! The legacy `Result<T>` class with `IsSuccess`, `Error` and already-translated messages is gone.
//! See `docs/04-dinero-fechas-y-tipos.md` §8.

use crate::error::AppError;

pub type AppResult<T> = Result<T, AppError>;
