use aes::Aes256;
use ctr::Ctr128BE;
use ctr::cipher::{KeyIvInit, StreamCipher};
use sha2::{Digest, Sha256};

type BedrockCtr = Ctr128BE<Aes256>;

pub struct BedrockEncryptor {
    cipher: BedrockCtr,
    key: [u8; 32],
    send_counter: u64,
}

impl BedrockEncryptor {
    #[must_use]
    pub fn new(key: &[u8; 32]) -> Self {
        let mut iv = [0u8; 16];
        iv[..12].copy_from_slice(&key[..12]);
        iv[12..].copy_from_slice(&[0, 0, 0, 2]);

        Self {
            cipher: BedrockCtr::new(key.into(), &iv.into()),
            key: *key,
            send_counter: 0,
        }
    }

    pub fn encrypt(&mut self, data: &mut Vec<u8>) {
        // data contains the payload after 0xfe
        let mut hasher = Sha256::new();
        hasher.update(self.send_counter.to_le_bytes());
        hasher.update(&data[..]);
        hasher.update(self.key);

        let hash = hasher.finalize();
        data.extend_from_slice(&hash[..8]);

        self.cipher.apply_keystream(data);

        self.send_counter += 1;
    }
}

pub struct BedrockDecryptor {
    cipher: BedrockCtr,
    key: [u8; 32],
    send_counter: u64,
}

impl BedrockDecryptor {
    #[must_use]
    pub fn new(key: &[u8; 32]) -> Self {
        let mut iv = [0u8; 16];
        iv[..12].copy_from_slice(&key[..12]);
        iv[12..].copy_from_slice(&[0, 0, 0, 2]);

        Self {
            cipher: BedrockCtr::new(key.into(), &iv.into()),
            key: *key,
            send_counter: 0,
        }
    }

    #[expect(clippy::needless_borrow)] // False positive
    pub fn decrypt(&mut self, data: &mut Vec<u8>) -> Result<(), String> {
        let ciphertext = data.clone();
        self.cipher.apply_keystream(data);

        if data.len() < 8 {
            return Err("Encrypted packet must be at least 8 bytes long".to_string());
        }

        let (payload, checksum) = data.split_at(data.len() - 8);

        let mut hasher = Sha256::new();
        hasher.update(self.send_counter.to_le_bytes());
        hasher.update(payload);
        hasher.update(self.key);

        let our_checksum = &hasher.finalize()[..8];

        if checksum != our_checksum {
            let cipher_prefix = if ciphertext.len() > 16 {
                &ciphertext[..16]
            } else {
                &ciphertext
            };
            let plain_prefix = if data.len() > 16 { &data[..16] } else { &data };
            return Err(format!(
                "Invalid checksum: expected {:x?}, got {:x?}. Cipher prefix: {:x?}, Plain prefix: {:x?}, Key: {:x?}, Counter: {}",
                our_checksum, checksum, cipher_prefix, plain_prefix, self.key, self.send_counter
            ));
        }

        data.truncate(payload.len());
        self.send_counter += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcm_compatibility() {
        let key = [1u8; 32];
        let mut iv = [0u8; 16];
        iv[..12].copy_from_slice(&key[..12]);
        iv[12..].copy_from_slice(&[0, 0, 0, 2]);

        let mut cipher = BedrockCtr::new((&key).into(), (&iv).into());
        let mut data = b"Hello Bedrock encryption!".to_vec();
        cipher.apply_keystream(&mut data);
        let expected = &[
            0xfa, 0x1c, 0xd6, 0xf6, 0x06, 0xd7, 0x47, 0x96, 0x8e, 0xd7, 0x60, 0xfe, 0xc5, 0x1c,
            0x2b, 0xc7, 0x7e, 0x46, 0x17, 0x74, 0x25, 0x96, 0x34, 0x0c, 0xac,
        ];
        assert_eq!(data, expected);
    }
}
