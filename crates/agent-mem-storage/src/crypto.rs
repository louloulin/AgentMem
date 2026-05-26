//! Cryptography Module for AgentMem
//! 
//! Provides AES-256-GCM encryption for data at rest.

use agent_mem_traits::{AgentMemError, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Base64 engine for encoding/decoding
const BASE64_ENGINE: base64::engine::general_purpose::GeneralPurpose = 
    base64::engine::general_purpose::STANDARD;

/// Decode base64 encoded key
fn decode_base64_key(key: &str) -> Result<Vec<u8>> {
    BASE64_ENGINE.decode(key)
        .map_err(|e| AgentMemError::StorageError(format!("Invalid base64 key: {}", e)))
}

/// Generate a new 32-byte encryption key (base64 encoded)
pub fn generate_encryption_key() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    
    let mut hasher1 = DefaultHasher::new();
    timestamp.hash(&mut hasher1);
    
    let mut hasher2 = DefaultHasher::new();
    (timestamp ^ 0x9e3779b97f4a7c15).hash(&mut hasher2);
    
    let mut key = vec![];
    key.extend_from_slice(&hasher1.finish().to_le_bytes());
    key.extend_from_slice(&hasher2.finish().to_le_bytes());
    
    while key.len() < 32 {
        let mut hasher = DefaultHasher::new();
        (timestamp ^ key.len() as u128).hash(&mut hasher);
        key.extend_from_slice(&hasher.finish().to_le_bytes());
    }
    
    BASE64_ENGINE.encode(&key[..32])
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_data(data: &[u8], key: &str) -> Result<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    // Decode key from base64
    let key_bytes = decode_base64_key(key)?;
    let key_array: [u8; 32] = key_bytes.as_slice().try_into()
        .map_err(|_| AgentMemError::StorageError("Key must be 32 bytes".to_string()))?;
    
    let cipher = Aes256Gcm::new_from_slice(&key_array)
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create cipher: {}", e)))?;
    
    // Generate pseudo-random nonce using timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let mut nonce_bytes = [0u8; 12];
    let hash1 = (timestamp ^ 0x9e3779b97f4a7c15) as u64;
    let hash2 = (timestamp ^ 0x3b9aca075591d5a7) as u64;
    nonce_bytes[0..8].copy_from_slice(&hash1.to_le_bytes());
    nonce_bytes[8..12].copy_from_slice(&(hash2 as u32).to_le_bytes());
    
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| AgentMemError::StorageError(format!("Encryption failed: {}", e)))?;
    
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_data(data: &[u8], key: &str) -> Result<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    if data.len() < 13 {
        return Err(AgentMemError::StorageError("Data too short for AES-256-GCM".to_string()));
    }
    
    let key_bytes = decode_base64_key(key)?;
    let key_array: [u8; 32] = key_bytes.as_slice().try_into()
        .map_err(|_| AgentMemError::StorageError("Key must be 32 bytes".to_string()))?;
    
    let cipher = Aes256Gcm::new_from_slice(&key_array)
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create cipher: {}", e)))?;
    
    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AgentMemError::StorageError(format!("Decryption failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_encryption_key();
        let data = b"Hello, AgentMem secure data!";
        
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
        assert_ne!(encrypted.as_slice(), data);
    }

    #[test]
    fn test_different_nonces() {
        let key = generate_encryption_key();
        let data = b"Test data";
        
        let encrypted1 = encrypt_data(data, &key).unwrap();
        let encrypted2 = encrypt_data(data, &key).unwrap();
        
        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);
        
        // But both should decrypt to same data
        assert_eq!(decrypt_data(&encrypted1, &key).unwrap(), data.to_vec());
        assert_eq!(decrypt_data(&encrypted2, &key).unwrap(), data.to_vec());
    }

    #[test]
    fn test_invalid_key() {
        let data = b"Test data";
        let result = encrypt_data(data, "invalid-base64!!!");
        assert!(result.is_err());
    }
}
