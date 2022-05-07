// #![deny(missing_docs)] // TODO: enable and fill docs

pub mod db_manager;
pub mod error;
pub mod processor;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
