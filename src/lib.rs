//! Crate library entry: re-export modules for integration tests and external use.

pub mod config;
pub mod security;
pub mod tools;

// Convenient re-exports
pub use security::encryption::EncryptionService;
pub use tools::error::WalletError;
