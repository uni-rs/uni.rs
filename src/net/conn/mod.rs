//! Contains various type and trait definition related to connexion management

use sync::Arc;

mod uni;
mod multi;
pub mod filter;

pub use self::uni::UniConn;
pub use self::multi::MultiConn;

/// A connexion, either uni or multi
pub enum Connexion {
    Uni(Arc<UniConn>),
    Multi(Arc<MultiConn>),
}
