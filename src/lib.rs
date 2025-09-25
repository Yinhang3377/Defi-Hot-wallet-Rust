<<<<<<< HEAD
// src/lib.rs
pub mod blockchain;
pub mod core;
pub mod storage;
pub mod api;
pub mod crypto;
pub mod config;
pub mod utils;
pub mod i18n;
pub mod monitoring;
pub mod mvp;
=======
//! Crate library entry: re-export modules for integration tests and external use.

pub mod config;
pub mod security;
pub mod tools;

// Convenient re-exports
pub use security::encryption::WalletSecurity;
pub use tools::error::WalletError;
>>>>>>> be35db3d094cb6edd3c63585f33fdcb299a57158
