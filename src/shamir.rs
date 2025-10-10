use std::num::NonZeroU8;

/// Shamir 绉樺瘑鍒嗕韩鐩稿叧鐨勯敊璇被鍨?
#[derive(Debug, thiserror::Error)]
pub enum ShamirError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Failed to split secret: {0}")]
    SplitFailed(String),
    #[error("Failed to combine shares: {0}")]
    CombineFailed(String),
}

/// 灏嗙瀵嗗垎鍓叉垚澶氫釜浠介銆?
///
/// # Arguments
/// * `secret` - 瑕佸垎鍓茬殑绉樺瘑鏁版嵁銆?
/// * `threshold` - 鎭㈠绉樺瘑鎵€闇€鐨勬渶灏忎唤棰濇暟 (k)銆?
/// * `total_shares` - 瑕佺敓鎴愮殑鎬讳唤棰濇暟 (n)銆?
///
/// # Returns
/// 涓€涓寘鍚?`total_shares` 涓唤棰濈殑鍚戦噺銆?
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

    // shamir 2.0.0+ API: split_secret(threshold, total_shares, secret)
    shamir::split_secret(k, n, secret) 
        .map_err(|e| ShamirError::SplitFailed(e.to_string()))
}

/// 浠庝竴缁勪唤棰濅腑鎭㈠绉樺瘑銆?
///
/// # Arguments
/// * `shares` - 鐢ㄤ簬鎭㈠绉樺瘑鐨勪唤棰濆垏鐗囥€?
///
/// # Returns
/// 鎭㈠鐨勭瀵嗘暟鎹€?
pub fn combine_shares(shares: &[Vec<u8>]) -> Result<Vec<u8>, ShamirError> {
    if shares.is_empty() {
        return Err(ShamirError::InvalidParameters("Shares cannot be empty".to_string()));
    }

    // 妫€鏌ヤ唤棰?ID 鏄惁鍞竴涓旈潪闆?
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

    // shamir 2.0.0+ API: recover_secret(shares)
    ::shamir::recover_secret(&share_slices).map_err(|e| ShamirError::CombineFailed(e.to_string()))
}

// 为了保持API兼容性，添加一个combine_secret别名函数
/// Alias for `combine_shares` to maintain API compatibility.
pub fn combine_secret(shares: &[Vec<u8>]) -> Result<Vec<u8>, ShamirError> {
    combine_shares(shares)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_combine() {
        let secret = b"test secret data";
        let shares = split_secret(secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);
        // 浣跨敤涓嶅悓鐨?3 涓唤棰濈粍鍚?
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
