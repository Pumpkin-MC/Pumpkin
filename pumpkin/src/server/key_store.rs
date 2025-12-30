use num_bigint::BigInt;
use pkcs8::EncodePublicKey;
use pumpkin_protocol::java::client::login::CEncryptionRequest;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::Sha1;
use sha2::Digest;

use crate::net::EncryptionError;

pub struct KeyStore {
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,
}

impl KeyStore {
    #[must_use]
    pub fn new() -> Self {
        log::debug!("Creating encryption keys...");
        let private_key = Self::generate_private_key();

        let public_key = private_key.to_public_key();

        let public_key_der = public_key
            .to_public_key_der()
            .expect("Failed to encode public key to SPKI DER")
            .as_bytes()
            .to_vec()
            .into_boxed_slice();

        Self {
            private_key,
            public_key_der,
        }
    }

    fn generate_private_key() -> RsaPrivateKey {
        // Found out that OsRng is faster than rand::thread_rng here
        let mut rng = rand::rng();

        // Use 2048-bit keys for stronger security (upgraded from 1024-bit)
        RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate a key")
    }

    pub fn encryption_request<'a>(
        &'a self,
        server_id: &'a str,
        verification_token: &'a [u8; 4],
        should_authenticate: bool,
    ) -> CEncryptionRequest<'a> {
        CEncryptionRequest::new(
            server_id,
            &self.public_key_der,
            verification_token,
            should_authenticate,
        )
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let decrypted = self
            .private_key
            .decrypt(Pkcs1v15Encrypt, data)
            .map_err(|_| EncryptionError::FailedDecrypt)?;
        Ok(decrypted)
    }

    pub fn get_digest(&self, secret: &[u8]) -> String {
        auth_digest(
            &Sha1::new()
                .chain_update(secret)
                .chain_update(&self.public_key_der)
                .finalize(),
        )
    }
}

pub fn auth_digest(bytes: &[u8]) -> String {
    BigInt::from_signed_bytes_be(bytes).to_str_radix(16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::traits::PublicKeyParts;

    /// Property test: For any generated RSA key pair, the key length SHALL be at least 2048 bits.
    /// **Feature: security-hardening, Property 1: RSA Key Length**
    /// **Validates: Requirements 1.1**
    #[test]
    fn test_property_rsa_key_minimum_2048_bits() {
        // Generate multiple keys to ensure consistency
        for _ in 0..5 {
            let key = KeyStore::generate_private_key();
            let key_bits = key.n().bits();

            assert!(
                key_bits >= 2048,
                "RSA key length should be at least 2048 bits, but got {} bits",
                key_bits
            );
        }
    }

    #[test]
    fn test_keystore_creates_valid_keys() {
        let keystore = KeyStore::new();

        // Verify private key has correct bit length
        let key_bits = keystore.private_key.n().bits();
        assert!(
            key_bits >= 2048,
            "KeyStore private key should be at least 2048 bits, got {} bits",
            key_bits
        );

        // Verify public key DER is not empty
        assert!(
            !keystore.public_key_der.is_empty(),
            "Public key DER should not be empty"
        );
    }

    #[test]
    fn test_decrypt_roundtrip() {
        let keystore = KeyStore::new();
        let original_data = b"test encryption data";

        // Encrypt with public key
        let public_key = keystore.private_key.to_public_key();
        let mut rng = rand::rng();
        let encrypted = public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, original_data)
            .expect("Encryption should succeed");

        // Decrypt with private key
        let decrypted = keystore
            .decrypt(&encrypted)
            .expect("Decryption should succeed");

        assert_eq!(
            decrypted, original_data,
            "Decrypted data should match original"
        );
    }
}
