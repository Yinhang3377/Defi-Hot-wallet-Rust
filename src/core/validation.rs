use anyhow::Result;
use regex::Regex;

/// Validates an Ethereum address.
pub fn validate_ethereum_address(address: &str) -> Result<()> {
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(anyhow::anyhow!("Invalid Ethereum address format"));
    }
    let hex_regex = Regex::new(r"^0x[0-9a-fA-F]{40}$").unwrap();
    if !hex_regex.is_match(address) {
        return Err(anyhow::anyhow!("Invalid Ethereum address characters"));
    }
    Ok(())
}

/// Validates a Solana address (base58 encoded).
pub fn validate_solana_address(address: &str) -> Result<()> {
    if address.len() < 32 || address.len() > 44 {
        return Err(anyhow::anyhow!("Invalid Solana address length"));
    }
    // Check if it's valid base58
    match bs58::decode(address).into_vec() {
        Ok(decoded) => {
            if decoded.len() != 32 {
                return Err(anyhow::anyhow!("Invalid Solana address decoded length"));
            }
        }
        Err(_) => return Err(anyhow::anyhow!("Invalid Solana address base58 encoding")),
    }
    Ok(())
}

/// Validates an address based on network.
pub fn validate_address(address: &str, network: &str) -> Result<()> {
    match network {
        "eth" | "sepolia" | "polygon" | "bsc" => validate_ethereum_address(address),
        "solana" | "solana-devnet" => validate_solana_address(address),
        _ => Err(anyhow::anyhow!("Unsupported network for address validation: {}", network)),
    }
}

/// Validates an amount string (positive number).
pub fn validate_amount(amount: &str) -> Result<f64> {
    let amount: f64 = amount.parse().map_err(|_| anyhow::anyhow!("Invalid amount format"))?;
    if amount <= 0.0 {
        return Err(anyhow::anyhow!("Amount must be positive"));
    }
    Ok(amount)
}

/// Validates a token symbol.
pub fn validate_token(token: &str) -> Result<()> {
    if token.is_empty() || token.len() > 10 {
        return Err(anyhow::anyhow!("Invalid token symbol"));
    }
    let token_regex = Regex::new(r"^[A-Z]{2,10}$").unwrap();
    if !token_regex.is_match(token) {
        return Err(anyhow::anyhow!("Token symbol must be uppercase letters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ethereum_address_valid() {
        assert!(validate_ethereum_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").is_ok());
    }

    #[test]
    fn test_validate_ethereum_address_invalid_length() {
        assert!(validate_ethereum_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44").is_err());
    }

    #[test]
    fn test_validate_ethereum_address_invalid_chars() {
        assert!(validate_ethereum_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44g").is_err());
    }

    #[test]
    fn test_validate_solana_address_valid() {
        assert!(validate_solana_address("11111111111111111111111111111112").is_ok());
    }

    #[test]
    fn test_validate_solana_address_invalid() {
        assert!(validate_solana_address("invalid").is_err());
    }

    #[test]
    fn test_validate_address_eth() {
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e", "eth").is_ok());
    }

    #[test]
    fn test_validate_address_solana() {
        assert!(validate_address("11111111111111111111111111111112", "solana").is_ok());
    }

    #[test]
    fn test_validate_amount_valid() {
        assert_eq!(validate_amount("10.5").unwrap(), 10.5);
    }

    #[test]
    fn test_validate_amount_invalid() {
        assert!(validate_amount("-10").is_err());
    }

    #[test]
    fn test_validate_token_valid() {
        assert!(validate_token("USDC").is_ok());
    }

    #[test]
    fn test_validate_token_invalid() {
        assert!(validate_token("usdc").is_err());
    }
}
