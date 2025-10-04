// src/security/mod.rs
//! Security-related functionality for the wallet
//!
//! This module contains security features such as anti-debugging,
//! zeroization utilities, and other protective measures.

pub mod access_control;
pub mod compliance;
pub mod encryption;
pub mod memory_protection;
pub mod shamir;

// Add the new anti-debug module
pub mod anti_debug;

// Re-export commonly used security functions for convenience
pub use anti_debug::is_debugger_present;
