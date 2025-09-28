//! # DeFi Hot Wallet Library
//!
//! This is the main library crate for the DeFi Hot Wallet application. It encapsulates
//! all the core logic, including wallet management, cryptographic operations,
//! blockchain interactions, and security features.

// Re-export modules to make them accessible from the outside.
pub mod api;
pub mod application;
pub mod audit;
pub mod blockchain;
pub mod cli;
pub mod config;
pub mod core;
pub mod crypto;
pub mod i18n;
pub mod monitoring;
pub mod mvp;
pub mod network;
pub mod ops;
pub mod security;
pub mod storage;
pub mod tools;
pub mod utils;

use crate::core::config::WalletConfig;
use anyhow::Result;

/// Initializes the wallet library with a default configuration.
/// This is a placeholder for any top-level library setup.
pub fn init_wallet_lib() -> Result<()> {
    // In a real scenario, this might initialize logging, load a default config,
    // or perform other global setup tasks.
    Ok(())
}

/// Initializes the wallet library with a specific configuration.
/// This is a placeholder to simulate initialization with different settings.
pub fn init_wallet_lib_with_config(config: WalletConfig) -> Result<()> {
    // A real implementation would use the config to set up various components.
    // For this test, we'll check for a specific "invalid" condition.
    if config.storage.database_url == "invalid-path" {
        return Err(anyhow::anyhow!("Invalid database path in config"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::WalletConfig;

    #[test]
    fn test_lib_initialization() {
        // Happy path: Initialize the library.
        let result = init_wallet_lib();
        assert!(result.is_ok());
    }

    #[test]
    fn test_lib_invalid_config() {
        // Error path: Invalid configuration.
        let mut config = WalletConfig::default();
        // Simulate an invalid configuration that would cause an error.
        config.storage.database_url = "invalid-path".to_string();
        let result = init_wallet_lib_with_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_lib_edge_case_empty_config() {
        // Edge case: Default (empty) configuration.
        let config = WalletConfig::default();
        let result = init_wallet_lib_with_config(config);
        assert!(result.is_ok());
    }
}
