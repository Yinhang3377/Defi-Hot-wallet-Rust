use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use vsss_rs::{
    curve25519::WrappedRistretto, shamir::Shamir, traits::SecretSharing, Error as VsssError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShamirSecretSharing {
    default_threshold: u8,
    default_total_shares: u8,
}

impl ShamirSecretSharing {
    // wallet.rs 里使用的无参构造
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new Shamir secret sharing configuration.
    ///
    /// # Arguments
    ///
    /// * `threshold` - The minimum number of shares required to reconstruct the secret.
    /// * `total_shares` - The total number of shares to generate.
    ///
    pub fn with_threshold(threshold: u8, total_shares: u8) -> Result<Self> {
        validate_params(threshold, total_shares)?;
        Ok(Self {
            default_threshold: threshold,
            default_total_shares: total_shares,
        })
    }

    /// Creates shares from a given secret.
    ///
    /// # Arguments
    /// * `secret` - The secret data to be split.
    /// * `total_shares` - The total number of shares to create.
    /// * `threshold` - The minimum number of shares for reconstruction.
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

        let shamir = Shamir {
            threshold: threshold as usize,
            share_count: total_shares as usize,
        };
        let shares = shamir
            .split_secret::<WrappedRistretto>(secret)
            .map_err(|e| anyhow!("Failed to split secret: {}", e))?;

        // Serialize each share to Vec<u8>
        shares
            .into_iter()
            .map(|s| bincode::serialize(&s).map_err(|e| anyhow!("Failed to serialize share: {}", e)))
            .collect()
    }

    /// Reconstructs the secret from a set of shares using the default threshold.
    pub fn reconstruct_secret(&self, shares_bytes: &[Vec<u8>]) -> Result<Vec<u8>> {
        self.reconstruct_with_threshold(shares_bytes, self.default_threshold)
    }

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

        // Deserialize Vec<u8> back to shares
        let shares: Vec<_> = shares_bytes
            .iter()
            .map(|bytes| {
                bincode::deserialize(bytes).map_err(|e| anyhow!("Invalid share data: {}", e))
            })
            .collect::<Result<Vec<_>>>()?;

        let shamir = Shamir {
            threshold: threshold as usize,
            share_count: shares.len(), // Not used in combine, but for consistency
        };

        shamir
            .combine_shares::<WrappedRistretto>(&shares)
            .map_err(|e| anyhow!("Failed to reconstruct secret: {}", e))
    }

    pub fn get_threshold(&self) -> u8 {
        self.default_threshold
    }
    pub fn get_total_shares(&self) -> u8 {
        self.default_total_shares
    }
}

impl Default for ShamirSecretSharing { 
    fn default() -> Self {
        // 合理默认：2-of-3
        Self { default_threshold: 2, default_total_shares: 3 }
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
