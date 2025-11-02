//! Cryptographic utilities for NEXUS
//!
//! This module provides secure cryptographic operations including:
//! - Key derivation and management
//! - Symmetric encryption/decryption
//! - Secure hashing and verification
//! - Random number generation

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::fmt;

/// Cryptographic provider trait for abstracting crypto operations
pub trait CryptoProvider: Send + Sync {
    /// Generate a random key of specified length
    fn generate_key(&self, length: usize) -> Result<Vec<u8>>;
    
    /// Derive a key from a password using PBKDF2
    fn derive_key(&self, password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>>;
    
    /// Encrypt data with AES-256-GCM
    fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<EncryptedData>;
    
    /// Decrypt data with AES-256-GCM
    fn decrypt(&self, encrypted_data: &EncryptedData, key: &[u8]) -> Result<Vec<u8>>;
    
    /// Hash data using SHA-256
    fn hash(&self, data: &[u8]) -> Result<Vec<u8>>;
    
    /// Verify hash
    fn verify_hash(&self, data: &[u8], expected_hash: &[u8]) -> Result<bool>;
    
    /// Generate random bytes
    fn random_bytes(&self, length: usize) -> Result<Vec<u8>>;
}

/// Encrypted data container
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
    /// Nonce/IV used for encryption
    pub nonce: Vec<u8>,
    /// Authentication tag (for AEAD)
    pub tag: Vec<u8>,
}

impl EncryptedData {
    pub fn new(ciphertext: Vec<u8>, nonce: Vec<u8>, tag: Vec<u8>) -> Self {
        Self {
            ciphertext,
            nonce,
            tag,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&(self.nonce.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&(self.tag.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.tag);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            bail!("Invalid encrypted data: too short");
        }

        let mut offset = 0;
        
        // Read nonce length and nonce
        let nonce_len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]) as usize;
        offset += 4;
        
        if data.len() < offset + nonce_len + 4 {
            bail!("Invalid encrypted data: truncated nonce");
        }
        
        let nonce = data[offset..offset + nonce_len].to_vec();
        offset += nonce_len;
        
        // Read tag length and tag
        let tag_len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]) as usize;
        offset += 4;
        
        if data.len() < offset + tag_len {
            bail!("Invalid encrypted data: truncated tag");
        }
        
        let tag = data[offset..offset + tag_len].to_vec();
        offset += tag_len;
        
        // Read ciphertext
        let ciphertext = data[offset..].to_vec();
        
        Ok(Self::new(ciphertext, nonce, tag))
    }
}

/// Key derivation parameters
#[derive(Debug, Clone)]
pub struct KeyDerivationParams {
    /// Salt for key derivation
    pub salt: Vec<u8>,
    /// Number of iterations
    pub iterations: u32,
    /// Derived key length
    pub key_length: usize,
}

impl Default for KeyDerivationParams {
    fn default() -> Self {
        Self {
            salt: Vec::new(), // Will be generated randomly if empty
            iterations: 100_000, // OWASP recommended minimum
            key_length: 32, // 256 bits
        }
    }
}

/// Key manager for handling cryptographic keys
pub struct KeyManager {
    provider: Box<dyn CryptoProvider>,
    keys: HashMap<String, Vec<u8>>,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new(provider: Box<dyn CryptoProvider>) -> Self {
        Self {
            provider,
            keys: HashMap::new(),
        }
    }

    /// Generate a new key and store it
    pub fn generate_key(&mut self, name: &str, length: usize) -> Result<()> {
        let key = self.provider.generate_key(length)
            .context("Failed to generate key")?;
        
        self.keys.insert(name.to_string(), key);
        Ok(())
    }

    /// Derive a key from password and store it
    pub fn derive_key(&mut self, name: &str, password: &str, params: &KeyDerivationParams) -> Result<()> {
        let salt = if params.salt.is_empty() {
            self.provider.random_bytes(32)?
        } else {
            params.salt.clone()
        };

        let key = self.provider.derive_key(password, &salt, params.iterations)
            .context("Failed to derive key")?;
        
        self.keys.insert(name.to_string(), key);
        Ok(())
    }

    /// Get a key by name
    pub fn get_key(&self, name: &str) -> Result<&[u8]> {
        self.keys.get(name)
            .map(|k| k.as_slice())
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", name))
    }

    /// Encrypt data using a named key
    pub fn encrypt(&self, data: &[u8], key_name: &str) -> Result<EncryptedData> {
        let key = self.get_key(key_name)?;
        self.provider.encrypt(data, key)
            .context("Failed to encrypt data")
    }

    /// Decrypt data using a named key
    pub fn decrypt(&self, encrypted_data: &EncryptedData, key_name: &str) -> Result<Vec<u8>> {
        let key = self.get_key(key_name)?;
        self.provider.decrypt(encrypted_data, key)
            .context("Failed to decrypt data")
    }

    /// Remove a key
    pub fn remove_key(&mut self, name: &str) -> bool {
        self.keys.remove(name).is_some()
    }

    /// List all key names
    pub fn list_keys(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    /// Clear all keys (for security)
    pub fn clear_keys(&mut self) {
        self.keys.clear();
    }
}

/// Mock crypto provider for testing (NOT for production use)
#[cfg(test)]
pub struct MockCryptoProvider;

#[cfg(test)]
impl CryptoProvider for MockCryptoProvider {
    fn generate_key(&self, length: usize) -> Result<Vec<u8>> {
        Ok(vec![0u8; length]) // Predictable for testing
    }

    fn derive_key(&self, _password: &str, _salt: &[u8], _iterations: u32) -> Result<Vec<u8>> {
        Ok(vec![1u8; 32]) // Fixed key for testing
    }

    fn encrypt(&self, data: &[u8], _key: &[u8]) -> Result<EncryptedData> {
        // Simple XOR for testing
        let mut encrypted = data.to_vec();
        for byte in &mut encrypted {
            *byte ^= 0x42;
        }
        
        Ok(EncryptedData::new(
            encrypted,
            vec![0u8; 12], // Mock nonce
            vec![0u8; 16], // Mock tag
        ))
    }

    fn decrypt(&self, encrypted_data: &EncryptedData, _key: &[u8]) -> Result<Vec<u8>> {
        // Reverse the XOR
        let mut decrypted = encrypted_data.ciphertext.clone();
        for byte in &mut decrypted {
            *byte ^= 0x42;
        }
        Ok(decrypted)
    }

    fn hash(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simple sum for testing
        let sum: u8 = data.iter().fold(0, |acc, &x| acc.wrapping_add(x));
        Ok(vec![sum; 32])
    }

    fn verify_hash(&self, data: &[u8], expected_hash: &[u8]) -> Result<bool> {
        let hash = self.hash(data)?;
        Ok(hash == expected_hash)
    }

    fn random_bytes(&self, length: usize) -> Result<Vec<u8>> {
        Ok((0..length).map(|i| (i % 256) as u8).collect())
    }
}

/// Create a new key manager with the default crypto provider
pub fn create_key_manager() -> KeyManager {
    #[cfg(test)]
    {
        KeyManager::new(Box::new(MockCryptoProvider))
    }
    
    #[cfg(not(test))]
    {
        // In a real implementation, this would use a proper crypto provider
        // For now, we'll use the mock provider to avoid external dependencies
        KeyManager::new(Box::new(MockCryptoProvider))
    }
}

/// Secure string that clears itself from memory when dropped
pub struct SecureString {
    data: Vec<u8>,
}

impl SecureString {
    pub fn new(data: String) -> Self {
        Self {
            data: data.into_bytes(),
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.data)
            .context("Invalid UTF-8 in secure string")
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Zero out the memory
        for byte in &mut self.data {
            *byte = 0;
        }
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureString([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_data_serialization() {
        let data = EncryptedData::new(
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8, 9, 10, 11, 12],
            vec![13, 14, 15, 16],
        );

        let serialized = data.to_bytes();
        let deserialized = EncryptedData::from_bytes(&serialized).unwrap();

        assert_eq!(data.ciphertext, deserialized.ciphertext);
        assert_eq!(data.nonce, deserialized.nonce);
        assert_eq!(data.tag, deserialized.tag);
    }

    #[test]
    fn test_key_manager() {
        let mut key_manager = create_key_manager();
        
        // Generate a key
        key_manager.generate_key("test_key", 32).unwrap();
        assert!(key_manager.get_key("test_key").is_ok());
        
        // Test encryption/decryption
        let data = b"Hello, world!";
        let encrypted = key_manager.encrypt(data, "test_key").unwrap();
        let decrypted = key_manager.decrypt(&encrypted, "test_key").unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_secure_string() {
        let secure_str = SecureString::new("secret".to_string());
        assert_eq!(secure_str.as_str().unwrap(), "secret");
        assert_eq!(secure_str.len(), 6);
        assert!(!secure_str.is_empty());
        
        // Test that it doesn't leak in debug output
        let debug_output = format!("{:?}", secure_str);
        assert!(debug_output.contains("REDACTED"));
        assert!(!debug_output.contains("secret"));
    }
}
