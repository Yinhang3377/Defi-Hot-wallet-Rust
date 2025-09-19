use anyhow::Result;
use sharks::{Share, Sharks};
use zeroize::{Zeroize, ZeroizeOnDrop};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ShamirShare {
    pub index: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct ShamirSecretSharing {
    threshold: u8,
    total_shares: u8,
}

impl ShamirSecretSharing {
    pub fn new() -> Self {
        info!("🔐 Initializing Shamir Secret Sharing");
        Self {
            threshold: 2,
            total_shares: 3,
        }
    }
    
    pub fn with_threshold(threshold: u8, total_shares: u8) -> Result<Self> {
        if threshold > total_shares {
            return Err(anyhow::anyhow!("Threshold cannot be greater than total shares"));
        }
        if threshold == 0 || total_shares == 0 {
            return Err(anyhow::anyhow!("Threshold and total shares must be greater than 0"));
        }
        
        info!("🔐 Initializing Shamir Secret Sharing with {}-of-{} threshold", threshold, total_shares);
        Ok(Self {
            threshold,
            total_shares,
        })
    }
    
    pub fn create_shares(&self, secret: &[u8], total_shares: u8, threshold: u8) -> Result<Vec<Vec<u8>>> {
        debug!("Creating {}-of-{} Shamir secret shares", threshold, total_shares);
        
        if secret.is_empty() {
            return Err(anyhow::anyhow!("Secret cannot be empty"));
        }
        
        let sharks = Sharks(threshold);
        let dealer = sharks.dealer(secret);
        
        let shares: Vec<Vec<u8>> = dealer
            .take(total_shares as usize)
            .map(|share| {
                let mut share_bytes = Vec::new();
                share_bytes.push(share.x);
                share_bytes.extend_from_slice(&share.y);
                share_bytes
            })
            .collect();
        
        info!("✅ Created {} Shamir secret shares", shares.len());
        Ok(shares)
    }
    
    pub fn reconstruct_secret(&self, shares: &[Vec<u8>]) -> Result<Vec<u8>> {
        debug!("Reconstructing secret from {} shares", shares.len());
        
        if shares.len() < self.threshold as usize {
            return Err(anyhow::anyhow!(
                "Insufficient shares: need at least {}, got {}", 
                self.threshold, 
                shares.len()
            ));
        }
        
        let sharks = Sharks(self.threshold);
        
        // Convert our share format back to sharks::Share
        let shark_shares: Result<Vec<Share>, _> = shares
            .iter()
            .take(self.threshold as usize)
            .map(|share_bytes| {
                if share_bytes.is_empty() {
                    return Err(anyhow::anyhow!("Invalid share: empty"));
                }
                
                let x = share_bytes[0];
                let y = share_bytes[1..].to_vec();
                
                Ok(Share { x, y })
            })
            .collect();
        
        let shark_shares = shark_shares?;
        
        let secret = sharks.recover(&shark_shares)
            .map_err(|e| anyhow::anyhow!("Failed to recover secret: {}", e))?;
        
        info!("✅ Successfully reconstructed secret from shares");
        Ok(secret)
    }
    
    pub fn verify_shares(&self, shares: &[Vec<u8>], expected_secret: &[u8]) -> Result<bool> {
        if shares.len() < self.threshold as usize {
            return Ok(false);
        }
        
        let reconstructed = self.reconstruct_secret(shares)?;
        Ok(reconstructed == expected_secret)
    }
    
    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }
    
    pub fn get_total_shares(&self) -> u8 {
        self.total_shares
    }
}

impl Default for ShamirSecretSharing {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shamir_secret_sharing() {
        let shamir = ShamirSecretSharing::new();
        let secret = b"my super secret key";
        
        let shares = shamir.create_shares(secret, 3, 2).unwrap();
        assert_eq!(shares.len(), 3);
        
        // Test reconstruction with minimum shares
        let reconstructed = shamir.reconstruct_secret(&shares[..2]).unwrap();
        assert_eq!(secret, reconstructed.as_slice());
        
        // Test reconstruction with all shares
        let reconstructed = shamir.reconstruct_secret(&shares).unwrap();
        assert_eq!(secret, reconstructed.as_slice());
        
        // Test verification
        assert!(shamir.verify_shares(&shares[..2], secret).unwrap());
    }
    
    #[test]
    fn test_insufficient_shares() {
        let shamir = ShamirSecretSharing::new();
        let secret = b"my super secret key";
        
        let shares = shamir.create_shares(secret, 3, 2).unwrap();
        
        // Try to reconstruct with only 1 share (need 2)
        let result = shamir.reconstruct_secret(&shares[..1]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_custom_threshold() {
        let shamir = ShamirSecretSharing::with_threshold(3, 5).unwrap();
        let secret = b"my super secret key";
        
        let shares = shamir.create_shares(secret, 5, 3).unwrap();
        assert_eq!(shares.len(), 5);
        
        // Test reconstruction with minimum shares (3)
        let reconstructed = shamir.reconstruct_secret(&shares[..3]).unwrap();
        assert_eq!(secret, reconstructed.as_slice());
        
        // Test reconstruction with more than minimum
        let reconstructed = shamir.reconstruct_secret(&shares[..4]).unwrap();
        assert_eq!(secret, reconstructed.as_slice());
    }
}