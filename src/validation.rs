use ethers::types::U256;

/// 验证相关的错误类型
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}

/// 验证一个地址的格式是否基本有效。
///
/// # Arguments
/// * `address` - 要验证的地址字符串。
///
/// # Returns
/// `Ok(())` 如果地址有效，否则返回 `ValidationError`。
pub fn validate_address(address: &str) -> Result<(), ValidationError> {
    if address.is_empty() {
        return Err(ValidationError::InvalidAddress("Address cannot be empty".to_string()));
    }
    if !address.starts_with("0x") {
        return Err(ValidationError::InvalidAddress("Address must start with '0x'".to_string()));
    }
    // 简单的十六进制字符检查
    if address[2..].chars().any(|c| !c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidAddress("Address contains invalid hexadecimal characters".to_string()));
    }
    Ok(())
}

/// 一个简化的交易结构体，用于演示。
#[derive(Debug)]
pub struct Transaction {
    pub to: String,
    pub from: String,
    pub amount: U256,
}

impl Transaction {
    /// 创建一个新的交易实例。
    pub fn new(to: &str, from: &str, amount: U256) -> Self {
        Self {
            to: to.to_string(),
            from: from.to_string(),
            amount,
        }
    }
}

/// 验证一个交易是否有效。
///
/// # Arguments
/// * `tx` - 要验证的交易。
///
/// # Returns
/// `Ok(())` 如果交易有效，否则返回 `ValidationError`。
pub fn validate_transaction(tx: &Transaction) -> Result<(), ValidationError> {
    validate_address(&tx.to)?;
    validate_address(&tx.from)?;

    // 示例：模拟资金不足的检查
    let max_amount = U256::from(1_000_000_000); // 假设最大允许金额
    if tx.amount > max_amount {
        return Err(ValidationError::InvalidTransaction("Insufficient funds for this amount".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address() {
        assert!(validate_address("0x1234567890abcdef").is_ok());
    }

    #[test]
    fn test_validate_address_invalid() {
        assert!(validate_address("invalid").is_err());
    }

    #[test]
    fn test_validate_address_empty() {
        assert!(validate_address("").is_err());
    }

    #[test]
    fn test_validate_transaction() {
        // 使用有效的地址格式
        let tx = Transaction::new("0x742d35Cc6634C0532925a3b844Bc454e4438f44e", "0x742d35Cc6634C0532925a3b844Bc454e4438f44e", U256::from(100));
        assert!(validate_transaction(&tx).is_ok());
    }

    #[test]
    fn test_validate_transaction_insufficient_funds() {
        // 使用一个超过硬编码限额的大金额
        // 同时使用有效的地址格式
        let tx = Transaction::new("0x742d35Cc6634C0532925a3b844Bc454e4438f44e", "0x742d35Cc6634C0532925a3b844Bc454e4438f44e", U256::from(2_000_000_000));
        assert!(validate_transaction(&tx).is_err());
    }
}