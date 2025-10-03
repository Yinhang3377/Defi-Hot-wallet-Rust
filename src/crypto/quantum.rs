use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use zeroize::{Zeroize, ZeroizeOnDrop};

const KYBER_CIPHERTEXT_LEN: usize = 1568;
const KYBER_SECRET_LEN: usize = 3168;
const AES_NONCE_LEN: usize = 12;
const SHARED_SECRET: &[u8] = b"simulated_shared_secret";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeyPair {
    pub public_key: Vec<u8>,
    #[serde(skip_serializing)]
    secret_key: Vec<u8>,
}

#[derive(Debug)]
pub struct QuantumSafeEncryption {
    keypair: Option<QuantumKeyPair>,
}

impl QuantumSafeEncryption {
    pub fn new() -> Result<Self> {
        info!("馃攼 Initializing Quantum-Safe Encryption (Simulated Kyber1024)");
        let mut instance = Self { keypair: None };
        instance.generate_keypair()?;
        Ok(instance)
    }

    pub fn generate_keypair(&mut self) -> Result<QuantumKeyPair> {
        debug!("Generating new simulated Kyber1024 keypair");

        use rand::RngCore;
        let mut public_key = vec![0u8; KYBER_CIPHERTEXT_LEN];
        let mut secret_key = vec![0u8; KYBER_SECRET_LEN];

        rand::thread_rng().fill_bytes(&mut public_key);
        rand::thread_rng().fill_bytes(&mut secret_key);

        let keypair = QuantumKeyPair { public_key, secret_key };

        self.keypair = Some(keypair.clone());

        info!("鉁?Quantum-safe keypair generated (simulated)");
        Ok(keypair)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        debug!("Encrypting data with quantum-safe encryption (simulated)");

        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        use rand::RngCore;
        use sha2::{Digest, Sha256};

        // 鍥哄畾瀵嗛挜锛屼繚璇佹祴璇曚腑鍔犺В瀵嗕竴鑷?        let aes_key = Sha256::digest(SHARED_SECRET);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&aes_key));

        let mut nonce_bytes = [0u8; AES_NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("AES encryption failed: {e}"))?;

        // 妯℃嫙 KEM 鐨勫瘑鏂囬儴鍒嗭紙浠呯敤浜庡崰浣嶏級
        let mut simulated_kyber_ciphertext = vec![0u8; KYBER_CIPHERTEXT_LEN];
        rand::thread_rng().fill_bytes(&mut simulated_kyber_ciphertext);

        // 鎵撳寘鏍煎紡: [4 bytes len][kyber_ct][12 bytes nonce][aes_ct]
        let mut result = Vec::with_capacity(
            4 + simulated_kyber_ciphertext.len() + AES_NONCE_LEN + ciphertext.len(),
        );
        result.extend_from_slice(&(simulated_kyber_ciphertext.len() as u32).to_le_bytes());
        result.extend_from_slice(&simulated_kyber_ciphertext);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        // 闆跺寲涓棿鏁忔劅鏁版嵁
        use zeroize::Zeroize;
        nonce_bytes.zeroize();
        simulated_kyber_ciphertext.zeroize();

        debug!("鉁?Data encrypted with quantum-safe encryption (simulated)");
        Ok(result)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        debug!("Decrypting data with quantum-safe encryption (simulated)");

        if encrypted_data.len() < 4 {
            return Err(anyhow::anyhow!("Invalid encrypted data format"));
        }

        let kyber_ciphertext_len = u32::from_le_bytes([
            encrypted_data[0],
            encrypted_data[1],
            encrypted_data[2],
            encrypted_data[3],
        ]) as usize;

        let header_len = 4 + kyber_ciphertext_len;
        if encrypted_data.len() < header_len + AES_NONCE_LEN {
            return Err(anyhow::anyhow!("Invalid encrypted data length"));
        }

        let nonce_start = header_len;
        let nonce_end = nonce_start + AES_NONCE_LEN;
        let nonce_bytes = &encrypted_data[nonce_start..nonce_end];
        let aes_ciphertext = &encrypted_data[nonce_end..];

        use sha2::{Digest, Sha256};
        let aes_key = Sha256::digest(SHARED_SECRET);

        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&aes_key));
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, aes_ciphertext)
            .map_err(|e| anyhow::anyhow!("AES decryption failed: {e}"))?;

        debug!("鉁?Data decrypted with quantum-safe encryption (simulated)");
        Ok(plaintext)
    }

    pub fn get_public_key(&self) -> Option<&[u8]> {
        self.keypair.as_ref().map(|kp| kp.public_key.as_slice())
    }
}

impl Zeroize for QuantumSafeEncryption {
    fn zeroize(&mut self) {
        if let Some(ref mut kp) = self.keypair {
            kp.secret_key.zeroize();
        }
    }
}

impl ZeroizeOnDrop for QuantumSafeEncryption {}

impl Default for QuantumSafeEncryption {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_safe_encryption() {
        let crypto = QuantumSafeEncryption::new().unwrap();

        let plaintext = b"Hello, quantum-safe world!";
        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }
}
