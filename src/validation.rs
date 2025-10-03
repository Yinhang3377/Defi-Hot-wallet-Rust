use ethers::types::U256;

/// 楠岃瘉鐩稿叧鐨勯敊璇被鍨?
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}

/// 楠岃瘉涓€涓湴鍧€鐨勬牸寮忔槸鍚﹀熀鏈湁鏁堛€?
///
/// # Arguments
/// * `address` - 瑕侀獙璇佺殑鍦板潃瀛楃涓层€?
///
/// # Returns
/// `Ok(())` 濡傛灉鍦板潃鏈夋晥锛屽惁鍒欒繑鍥?`ValidationError`銆?
pub fn validate_address(address: &str) -> Result<(), ValidationError> {
    if address.is_empty() {
        return Err(ValidationError::InvalidAddress("Address cannot be empty".to_string()));
    }
    if !address.starts_with("0x") {
        return Err(ValidationError::InvalidAddress("Address must start with '0x'".to_string()));
    }
    // 绠€鍗曠殑鍗佸叚杩涘埗瀛楃妫€鏌?
    if address[2..].chars().any(|c| !c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidAddress("Address contains invalid hexadecimal characters".to_string()));
    }
    Ok(())
}

/// 涓€涓畝鍖栫殑浜ゆ槗缁撴瀯浣擄紝鐢ㄤ簬婕旂ず銆?
#[derive(Debug)]
pub struct Transaction {
    pub to: String,
    pub from: String,
    pub amount: U256,
}

impl Transaction {
    /// 鍒涘缓涓€涓柊鐨勪氦鏄撳疄渚嬨€?
    pub fn new(to: &str, from: &str, amount: U256) -> Self {
        Self {
            to: to.to_string(),
            from: from.to_string(),
            amount,
        }
    }
}

/// 楠岃瘉涓€涓氦鏄撴槸鍚︽湁鏁堛€?
///
/// # Arguments
/// * `tx` - 瑕侀獙璇佺殑浜ゆ槗銆?
///
/// # Returns
/// `Ok(())` 濡傛灉浜ゆ槗鏈夋晥锛屽惁鍒欒繑鍥?`ValidationError`銆?
pub fn validate_transaction(tx: &Transaction) -> Result<(), ValidationError> {
    validate_address(&tx.to)?;
    validate_address(&tx.from)?;

    // 绀轰緥锛氭ā鎷熻祫閲戜笉瓒崇殑妫€鏌?
    let max_amount = U256::from(1_000_000_000); // 鍋囪鏈€澶у厑璁搁噾棰?
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
        // 浣跨敤鏈夋晥鐨勫湴鍧€鏍煎紡
        let tx = Transaction::new("0x742d35Cc6634C0532925a3b844Bc454e4438f44e", "0x742d35Cc6634C0532925a3b844Bc454e4438f44e", U256::from(100));
        assert!(validate_transaction(&tx).is_ok());
    }

    #[test]
    fn test_validate_transaction_insufficient_funds() {
        // 浣跨敤涓€涓秴杩囩‖缂栫爜闄愰鐨勫ぇ閲戦
        // 鍚屾椂浣跨敤鏈夋晥鐨勫湴鍧€鏍煎紡
        let tx = Transaction::new("0x742d35Cc6634C0532925a3b844Bc454e4438f44e", "0x742d35Cc6634C0532925a3b844Bc454e4438f44e", U256::from(2_000_000_000));
        assert!(validate_transaction(&tx).is_err());
    }
}
