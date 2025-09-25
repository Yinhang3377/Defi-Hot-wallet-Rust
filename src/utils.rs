//! Utility functions for the wallet application

use anyhow::Result;

/// Converts a hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    Ok(bytes)
}

/// Converts bytes to a hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Creates a timestamp string in ISO 8601 format
pub fn format_timestamp(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    timestamp.to_rfc3339()
}