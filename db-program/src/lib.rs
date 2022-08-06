//! Solcery DB program
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]

pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
