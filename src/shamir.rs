use std::num::NonZeroU8;

/// Shamir 秘密分享相关的错误类型
#[derive(Debug, thiserror::Error)]
pub enum ShamirError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Failed to split secret: {0}")]
    SplitFailed(String),
    #[error("Failed to combine shares: {0}")]
    CombineFailed(String),
}

/// 将秘密分割成多个份额。
///
/// # Arguments
/// * `secret` - 要分割的秘密数据。
/// * `threshold` - 恢复秘密所需的最小份额数 (k)。
/// * `total_shares` - 要生成的总份额数 (n)。
///
/// # Returns
/// 一个包含 `total_shares` 个份额的向量。
pub fn split_secret(
    secret: &[u8],
    threshold: u8,
    total_shares: u8,
) -> Result<Vec<Vec<u8>>, ShamirError> {
    let k = NonZeroU8::new(threshold)
        .ok_or_else(|| ShamirError::InvalidParameters("Threshold cannot be zero".to_string()))?;
    let n = NonZeroU8::new(total_shares)
        .ok_or_else(|| ShamirError::InvalidParameters("Total shares cannot be zero".to_string()))?;

    if k > n {
        return Err(ShamirError::InvalidParameters(
            "Threshold cannot be greater than total shares".to_string(),
        ));
    }

    shamir::split_secret(k, n, secret)
        .map_err(|e| ShamirError::SplitFailed(e.to_string()))
}

/// 从一组份额中恢复秘密。
///
/// # Arguments
/// * `shares` - 用于恢复秘密的份额切片。
///
/// # Returns
/// 恢复的秘密数据。
pub fn combine_shares(shares: &[Vec<u8>]) -> Result<Vec<u8>, ShamirError> {
    if shares.is_empty() {
        return Err(ShamirError::InvalidParameters("Shares cannot be empty".to_string()));
    }

    // 检查份额 ID 是否唯一且非零
    let mut ids = std::collections::HashSet::new();
    for share in shares {
        if share.is_empty() {
            return Err(ShamirError::InvalidParameters("Share cannot be empty".to_string()));
        }
        if !ids.insert(share[0]) {
            return Err(ShamirError::InvalidParameters(format!("Duplicate share ID found: {}", share[0])));
        }
    }

    let share_slices: Vec<&[u8]> = shares.iter().map(|s| s.as_slice()).collect();

    shamir::combine_shares(&share_slices)
        .map_err(|e| ShamirError::CombineFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_combine() {
        let secret = b"test secret data";
        let shares = split_secret(secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);
        // 使用不同的 3 个份额组合
        let recovered = combine_shares(&[shares[0].clone(), shares[2].clone(), shares[4].clone()]).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_insufficient_shares() {
        let secret = b"test";
        let shares = split_secret(secret, 3, 5).unwrap();
        assert!(combine_shares(&shares[..2]).is_err());
    }

    #[test]
    fn test_invalid_shares() {
        assert!(combine_shares(&[]).is_err());
    }

    #[test]
    fn test_min_threshold() {
        let secret = b"min";
        let shares = split_secret(secret, 1, 1).unwrap();
        let recovered = combine_shares(&shares).unwrap();
        assert_eq!(recovered, secret);
    }
}