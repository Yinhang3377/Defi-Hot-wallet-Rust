use anyhow::Result;

/// 验证钱包名称是否有效
pub fn validate_wallet_name(name: &str) -> Result<bool> {
    if name.is_empty() || name.len() > 64 {
        return Ok(false);
    }

    // 只允许字母、数字、连字符和下划线
    let valid = name.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_'
    });

    Ok(valid)
}

/// 验证地址是否有效
pub fn validate_address(address: &str, chain: &str) -> Result<bool> {
    match chain {
        "eth" | "sepolia" | "polygon" | "bsc" | "bsctestnet" => {
            Ok(address.starts_with("0x") && address.len() == 42)
        }
        "solana" | "solana-devnet" => {
            // Solana 地址通常是 base58 编码的 32 字节公钥
            Ok(address.len() >= 32 && address.len() <= 44)
        }
        _ => Err(anyhow::anyhow!("Unsupported chain: {}", chain)),
    }
}

/// 验证金额是否有效
pub fn validate_amount(amount: &str) -> Result<bool> {
    match amount.parse::<f64>() {
        Ok(val) => Ok(val > 0.0),
        Err(e) => Err(anyhow::anyhow!("Invalid amount format: {}", e)),
    }
}
