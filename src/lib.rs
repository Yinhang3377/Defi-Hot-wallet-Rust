//! Crate library entry: re-export modules for integration tests and external use.

pub mod config;
pub mod tools;
pub mod security;

// Convenient re-exports
pub use tools::error::WalletError;
