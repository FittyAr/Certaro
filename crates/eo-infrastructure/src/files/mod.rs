//! Attachments on the local filesystem, and handing a file to the operating system.
//!
//! See `docs/13-servicios-externos-y-archivos.md` §1.

pub mod mime;
pub mod name;
pub mod opener;
pub mod store;

pub use opener::SystemOpener;
pub use store::FsAttachmentStore;
