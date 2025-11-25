//! Symmetric encryption using ChaCha20Poly1305
//!
//! Provides authenticated encryption with associated data (AEAD).

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::rngs::OsRng;
use zeroize::Zeroize;

use crate::error::{E2eeError, Result};

/// AEAD cipher for message encryption
pub struct MessageCipher {
    cipher: ChaCha20Poly1305,
}

impl MessageCipher {
    /// Create cipher from 32-byte key
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        Self {
            cipher: ChaCha20Poly1305::new(key),
        }
    }

    /// Encrypt plaintext with optional associated data
    ///
    /// Returns nonce || ciphertext (12 + len(plaintext) + 16 bytes)
    pub fn encrypt(&self, plaintext: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext = self
            .cipher
            .encrypt(&nonce, chacha20poly1305::aead::Payload {
                msg: plaintext,
                aad: associated_data,
            })
            .map_err(|_| E2eeError::Encryption("ChaCha20Poly1305 encryption failed".to_string()))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Encrypt with deterministic nonce (for testing only!)
    #[cfg(test)]
    pub fn encrypt_with_nonce(&self, plaintext: &[u8], nonce: &[u8; 12], associated_data: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);

        let ciphertext = self
            .cipher
            .encrypt(nonce, chacha20poly1305::aead::Payload {
                msg: plaintext,
                aad: associated_data,
            })
            .map_err(|_| E2eeError::Encryption("ChaCha20Poly1305 encryption failed".to_string()))?;

        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt ciphertext with optional associated data
    ///
    /// Input format: nonce (12 bytes) || ciphertext || tag (16 bytes)
    pub fn decrypt(&self, ciphertext: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 + 16 {
            return Err(E2eeError::Decryption("Ciphertext too short".to_string()));
        }

        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let encrypted = &ciphertext[12..];

        self.cipher
            .decrypt(nonce, chacha20poly1305::aead::Payload {
                msg: encrypted,
                aad: associated_data,
            })
            .map_err(|_| E2eeError::AuthenticationFailed)
    }
}

/// Derive encryption key from shared secret using HKDF
pub fn derive_key(shared_secret: &[u8], salt: &[u8], info: &[u8]) -> [u8; 32] {
    use hkdf::Hkdf;
    use sha2::Sha256;

    let hkdf = Hkdf::<Sha256>::new(Some(salt), shared_secret);
    let mut key = [0u8; 32];
    hkdf.expand(info, &mut key)
        .expect("HKDF expand should not fail for 32 bytes");
    key
}

/// Derive keys for Double Ratchet symmetric-key ratchet step
///
/// In the Double Ratchet, each DH ratchet produces a new root key and a chain key.
/// The chain key is used for either sending or receiving, not both.
pub struct DerivedKeys {
    pub chain_key: [u8; 32],
    pub next_root_key: [u8; 32],
}

impl DerivedKeys {
    /// Derive keys for a Double Ratchet step
    ///
    /// KDF produces (root_key, chain_key) from the previous root key and DH output
    pub fn derive(root_key: &[u8; 32], dh_output: &[u8; 32]) -> Self {
        let next_root_key = derive_key(dh_output, root_key, b"unhidra-e2ee-root");
        let chain_key = derive_key(dh_output, &next_root_key, b"unhidra-e2ee-chain");

        Self {
            chain_key,
            next_root_key,
        }
    }
}

impl Zeroize for DerivedKeys {
    fn zeroize(&mut self) {
        self.chain_key.zeroize();
        self.next_root_key.zeroize();
    }
}

impl Drop for DerivedKeys {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Message key derivation from chain key
pub fn derive_message_key(chain_key: &[u8; 32], message_number: u64) -> [u8; 32] {
    derive_key(
        chain_key,
        &message_number.to_le_bytes(),
        b"unhidra-e2ee-message",
    )
}

/// Advance chain key
pub fn advance_chain_key(chain_key: &[u8; 32]) -> [u8; 32] {
    derive_key(chain_key, &[], b"unhidra-e2ee-chain-advance")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let cipher = MessageCipher::new(&key);

        let plaintext = b"Hello, World!";
        let aad = b"message-id-123";

        let ciphertext = cipher.encrypt(plaintext, aad).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_aad_fails() {
        let key = [42u8; 32];
        let cipher = MessageCipher::new(&key);

        let plaintext = b"Hello, World!";
        let ciphertext = cipher.encrypt(plaintext, b"correct-aad").unwrap();

        let result = cipher.decrypt(&ciphertext, b"wrong-aad");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let key = [42u8; 32];
        let cipher = MessageCipher::new(&key);

        let plaintext = b"Hello, World!";
        let mut ciphertext = cipher.encrypt(plaintext, b"").unwrap();

        // Tamper with ciphertext
        ciphertext[20] ^= 0xFF;

        let result = cipher.decrypt(&ciphertext, b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_key_deterministic() {
        let secret = [1u8; 32];
        let salt = [2u8; 16];

        let key1 = derive_key(&secret, &salt, b"test");
        let key2 = derive_key(&secret, &salt, b"test");

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_info() {
        let secret = [1u8; 32];
        let salt = [2u8; 16];

        let key1 = derive_key(&secret, &salt, b"info1");
        let key2 = derive_key(&secret, &salt, b"info2");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derived_keys() {
        let root_key = [1u8; 32];
        let dh_output = [2u8; 32];

        let keys = DerivedKeys::derive(&root_key, &dh_output);

        // Keys should be different from each other and from inputs
        assert_ne!(keys.chain_key, keys.next_root_key);
        assert_ne!(keys.chain_key, root_key);
        assert_ne!(keys.next_root_key, root_key);

        // Derivation should be deterministic
        let keys2 = DerivedKeys::derive(&root_key, &dh_output);
        assert_eq!(keys.chain_key, keys2.chain_key);
        assert_eq!(keys.next_root_key, keys2.next_root_key);
    }
}
