use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sharks::{Share, Sharks};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShamirSecretSharing {
    default_threshold: u8,
    default_total_shares: u8,
}

impl Default for ShamirSecretSharing {
    fn default() -> Self {
        // 默认 2-of-3
        Self {
            default_threshold: 2,
            default_total_shares: 3,
        }
    }
}

impl ShamirSecretSharing {
    // wallet.rs 里使用的无参构造
    pub fn new() -> Self {
        Self::default()
    }

    // 可选：自定义默认门限配置
    pub fn with_threshold(threshold: u8, total_shares: u8) -> Result<Self> {
        validate_params(threshold, total_shares)?;
        Ok(Self {
            default_threshold: threshold,
            default_total_shares: total_shares,
        })
    }

    // wallet.rs 调用：create_shares(&master_key, total_shares=3, threshold=2)
    pub fn create_shares(
        &self,
        secret: &[u8],
        total_shares: u8,
        threshold: u8,
    ) -> Result<Vec<Vec<u8>>> {
        if secret.is_empty() {
            return Err(anyhow!("secret must not be empty"));
        }
        validate_params(threshold, total_shares)?;

        let sharks = Sharks(threshold); // sharks 要 u8
        let dealer = sharks.dealer(secret); // 需要 &[u8]

        // Share -> Vec<u8>
        let encoded: Vec<Vec<u8>> = dealer
            .take(total_shares as usize)
            .map(|s: Share| Vec::<u8>::from(&s)) // 修复：从 &Share 转换
            .collect();
        Ok(encoded)
    }

    // 使用默认门限恢复（兼容原测试）
    pub fn reconstruct_secret(&self, shares_bytes: &[Vec<u8>]) -> Result<Vec<u8>> {
        self.reconstruct_with_threshold(shares_bytes, self.default_threshold)
    }

    // 按给定门限恢复
    pub fn reconstruct_with_threshold(
        &self,
        shares_bytes: &[Vec<u8>],
        threshold: u8,
    ) -> Result<Vec<u8>> {
        if shares_bytes.len() < (threshold as usize) {
            return Err(anyhow!(
                "insufficient shares: need {}, got {}",
                threshold,
                shares_bytes.len()
            ));
        }

        // Vec<u8> -> Share
        let parsed: Vec<Share> = shares_bytes
            .iter()
            .map(|b| Share::try_from(b.as_slice()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("invalid share bytes: {}", e))?;

        let sharks = Sharks(threshold);
        let recovered = sharks
            .recover(parsed.iter()) // 迭代 &Share
            .map_err(|e| anyhow!("reconstruction failed: {}", e))?;

        Ok(recovered)
    }

    pub fn verify_shares(&self, shares_bytes: &[Vec<u8>], expected_secret: &[u8]) -> Result<bool> {
        Ok(self.reconstruct_secret(shares_bytes)? == expected_secret)
    }

    pub fn get_threshold(&self) -> u8 {
        self.default_threshold
    }
    pub fn get_total_shares(&self) -> u8 {
        self.default_total_shares
    }
}

fn validate_params(threshold: u8, total_shares: u8) -> Result<()> {
    if threshold == 0 || total_shares == 0 {
        return Err(anyhow!("threshold and total_shares must be > 0"));
    }
    if threshold > total_shares {
        return Err(anyhow!("threshold cannot be greater than total_shares"));
    }
    if threshold < 2 {
        return Err(anyhow!("threshold must be at least 2"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_default_2_of_3() {
        let sss = ShamirSecretSharing::new();
        let secret = b"my super secret key";
        let shares = sss.create_shares(secret, 3, 2).unwrap();

        let recovered = sss.reconstruct_with_threshold(&shares[..2], 2).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn roundtrip_custom_3_of_5() {
        let sss = ShamirSecretSharing::with_threshold(3, 5).unwrap();
        let secret = b"hello";
        let shares = sss.create_shares(secret, 5, 3).unwrap();
        let recovered = sss.reconstruct_with_threshold(&shares[..3], 3).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn insufficient_shares_err() {
        let sss = ShamirSecretSharing::new();
        let secret = b"abc";
        let shares = sss.create_shares(secret, 3, 2).unwrap();
        let err = sss.reconstruct_with_threshold(&shares[..1], 2).unwrap_err();
        assert!(err.to_string().contains("insufficient shares"));
    }
}
