use anyhow::Result;
use sha2::{Sha256, Digest};
use tracing::{debug, info, warn};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigConfig {
    pub threshold: u8,
    pub total_signers: u8,
    pub signers: Vec<String>, // Public keys or addresses
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigTransaction {
    pub id: String,
    pub to_address: String,
    pub amount: String,
    pub network: String,
    pub signatures: HashMap<String, Vec<u8>>,
    pub threshold: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct MultiSignature {
    pending_transactions: HashMap<String, MultiSigTransaction>,
}

impl MultiSignature {
    pub fn new() -> Self {
        info!("🔐 Initializing Multi-Signature manager");
        Self {
            pending_transactions: HashMap::new(),
        }
    }
    
    pub fn create_multisig_config(threshold: u8, signers: Vec<String>) -> Result<MultiSigConfig> {
        if threshold == 0 || threshold > signers.len() as u8 {
            return Err(anyhow::anyhow!(
                "Invalid threshold: {} (must be 1-{}))", 
                threshold, 
                signers.len()
            ));
        }
        
        if signers.is_empty() {
            return Err(anyhow::anyhow!("At least one signer is required"));
        }
        
        info!("✅ Created {}-of-{} multi-signature configuration", threshold, signers.len());
        
        Ok(MultiSigConfig {
            threshold,
            total_signers: signers.len() as u8,
            signers,
        })
    }
    
    pub fn propose_transaction(
        &mut self,
        id: String,
        to_address: String,
        amount: String,
        network: String,
        threshold: u8,
    ) -> Result<()> {
        if self.pending_transactions.contains_key(&id) {
            return Err(anyhow::anyhow!("Transaction with ID {} already exists", id));
        }
        
        let transaction = MultiSigTransaction {
            id: id.clone(),
            to_address,
            amount,
            network,
            signatures: HashMap::new(),
            threshold,
            created_at: chrono::Utc::now(),
        };
        
        self.pending_transactions.insert(id.clone(), transaction);
        
        info!("📝 Proposed multi-sig transaction: {}", id);
        Ok(())
    }
    
    pub fn sign_transaction(&mut self, tx_id: &str, signer: &str, signature: Vec<u8>) -> Result<bool> {
        let transaction = self.pending_transactions.get_mut(tx_id)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?;
        
        // Verify signature (simplified - in production, use proper signature verification)
        if !self.verify_signature(&signature, signer, tx_id)? {
            return Err(anyhow::anyhow!("Invalid signature from signer: {}", signer));
        }
        
        transaction.signatures.insert(signer.to_string(), signature);
        
        let signatures_count = transaction.signatures.len() as u8;
        let is_complete = signatures_count >= transaction.threshold;
        
        if is_complete {
            info!("✅ Multi-sig transaction {} is ready for execution ({}/{} signatures)", 
                  tx_id, signatures_count, transaction.threshold);
        } else {
            info!("📝 Multi-sig transaction {} signed by {} ({}/{} signatures)", 
                  tx_id, signer, signatures_count, transaction.threshold);
        }
        
        Ok(is_complete)
    }
    
    pub fn execute_transaction(&mut self, tx_id: &str) -> Result<MultiSigTransaction> {
        let transaction = self.pending_transactions.remove(tx_id)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?;
        
        if (transaction.signatures.len() as u8) < transaction.threshold {
            // Put it back since it's not ready
            self.pending_transactions.insert(tx_id.to_string(), transaction);
            return Err(anyhow::anyhow!("Insufficient signatures: {}/{}", 
                                     transaction.signatures.len(), transaction.threshold));
        }
        
        info!("🚀 Executing multi-sig transaction: {}", tx_id);
        Ok(transaction)
    }
    
    pub fn get_transaction(&self, tx_id: &str) -> Option<&MultiSigTransaction> {
        self.pending_transactions.get(tx_id)
    }
    
    pub fn list_pending_transactions(&self) -> Vec<&MultiSigTransaction> {
        self.pending_transactions.values().collect()
    }
    
    pub fn cancel_transaction(&mut self, tx_id: &str) -> Result<()> {
        self.pending_transactions.remove(tx_id)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?;
        
        warn!("❌ Cancelled multi-sig transaction: {}", tx_id);
        Ok(())
    }
    
    fn verify_signature(&self, signature: &[u8], signer: &str, message: &str) -> Result<bool> {
        // Simplified signature verification
        // In production, this would use proper ECDSA/EdDSA verification
        debug!("Verifying signature from {} for message: {}", signer, message);
        
        let expected = self.create_mock_signature(signer, message);
        Ok(signature == expected.as_slice())
    }
    
    fn create_mock_signature(&self, signer: &str, message: &str) -> Vec<u8> {
        // Mock signature creation for testing
        let mut hasher = Sha256::new();
        hasher.update(signer.as_bytes());
        hasher.update(message.as_bytes());
        hasher.finalize().to_vec()
    }
    
    pub fn create_signature_for_testing(&self, signer: &str, message: &str) -> Vec<u8> {
        self.create_mock_signature(signer, message)
    }
}

impl Default for MultiSignature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_multisig_config() {
        let signers = vec![
            "signer1".to_string(),
            "signer2".to_string(),
            "signer3".to_string(),
        ];
        
        let config = MultiSignature::create_multisig_config(2, signers).unwrap();
        assert_eq!(config.threshold, 2);
        assert_eq!(config.total_signers, 3);
        assert_eq!(config.signers.len(), 3);
    }
    
    #[test]
    fn test_multisig_transaction_flow() {
        let mut multisig = MultiSignature::new();
        
        // Propose transaction
        multisig.propose_transaction(
            "tx1".to_string(),
            "0x1234".to_string(),
            "1.0".to_string(),
            "eth".to_string(),
            2,
        ).unwrap();
        
        // First signature
        let sig1 = multisig.create_signature_for_testing("signer1", "tx1");
        let complete = multisig.sign_transaction("tx1", "signer1", sig1).unwrap();
        assert!(!complete);
        
        // Second signature (should complete)
        let sig2 = multisig.create_signature_for_testing("signer2", "tx1");
        let complete = multisig.sign_transaction("tx1", "signer2", sig2).unwrap();
        assert!(complete);
        
        // Execute transaction
        let tx = multisig.execute_transaction("tx1").unwrap();
        assert_eq!(tx.id, "tx1");
        assert_eq!(tx.signatures.len(), 2);
    }
    
    #[test]
    fn test_insufficient_signatures() {
        let mut multisig = MultiSignature::new();
        
        multisig.propose_transaction(
            "tx1".to_string(),
            "0x1234".to_string(),
            "1.0".to_string(),
            "eth".to_string(),
            2,
        ).unwrap();
        
        // Only one signature
        let sig1 = multisig.create_signature_for_testing("signer1", "tx1");
        multisig.sign_transaction("tx1", "signer1", sig1).unwrap();
        
        // Try to execute with insufficient signatures
        let result = multisig.execute_transaction("tx1");
        assert!(result.is_err());
    }
}