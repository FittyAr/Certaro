//! Persistence: connection, SeaORM models, mappers, repositories and the unit of work.

pub mod connection;
pub mod handle;
pub mod mappers;
pub mod models;
pub mod repositories;
pub mod seed;
pub mod unit_of_work;

pub use connection::{open, open_in_memory, open_readonly};
pub use handle::DbHandle;
pub use seed::{seed_demo_data, SeedSummary};
pub use unit_of_work::SeaOrmUnitOfWork;
