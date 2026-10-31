use std::time::Instant;

use pumpkin_protocol::java::client::login::CEncryptionRequest;
use rsa::pkcs8::EncodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::Sha1;
use sha2::Digest;
use tracing::{debug, error};

use crate::net::EncryptionError;

pub struct KeyStore {
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,
}

impl KeyStore {
    #[must_use]
    pub fn new() -> Self {
        let instant = Instant::now();
        debug!("Creating encryption keys...");
        let private_key = Self::generate_private_key();

        let public_key = private_key.to_public_key();

        let public_key_der = public_key
            .to_public_key_der()
            .map(|der| der.into_vec().into_boxed_slice())
            .unwrap_or_default();

        debug!("Created RSA keys, took {}ms", instant.elapsed().as_millis());

        Self {
            private_key,
            public_key_der,
        }
    }

    fn generate_private_key() -> RsaPrivateKey {
        let mut rng = rand::rng();

        RsaPrivateKey::new(&mut rng, 1024).unwrap_or_else(|_| {
            let mut fallback_rng = rand::rng();
            RsaPrivateKey::new(&mut fallback_rng, 1024).unwrap_or_else(|_| {
                error!("Failed to generate RSA key");
                std::process::exit(1);
            })
        })
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
    if bytes.is_empty() {
        return "0".to_string();
    }
    let is_negative = (bytes[0] & 0x80) != 0;
    if is_negative {
        let mut carry = 1u16;
        let mut negated = vec![0u8; bytes.len()];
        for (src, dst) in bytes.iter().rev().zip(negated.iter_mut().rev()) {
            let sum = u16::from(!*src) + carry;
            *dst = sum as u8;
            carry = sum >> 8;
        }
        let hex_str = hex::encode(negated);
        let trimmed = hex_str.trim_start_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            format!("-{trimmed}")
        }
    } else {
        let hex_str = hex::encode(bytes);
        let trimmed = hex_str.trim_start_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_digest_minecraft_vectors() {
        let digest_notch = Sha1::digest(b"Notch");
        assert_eq!(
            auth_digest(&digest_notch),
            "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48"
        );

        let digest_jeb = Sha1::digest(b"jeb_");
        assert_eq!(
            auth_digest(&digest_jeb),
            "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1"
        );

        let digest_simon = Sha1::digest(b"simon");
        assert_eq!(
            auth_digest(&digest_simon),
            "88e16a1019277b15d58faf0541e11910eb756f6"
        );

        assert_eq!(auth_digest(&[]), "0");
        assert_eq!(auth_digest(&[0, 0, 0]), "0");
    }
}
