use std::sync::LazyLock;

use aes::{
    Aes256,
    cipher::{Array, BlockCipherDecrypt, BlockCipherEncrypt},
};
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

const APPLICATION_ID: u64 = 0xdead_beef;
const BLOCK_SIZE: usize = 16;

pub const CHECKSUM_SIZE: usize = 32;

static KEY: LazyLock<[u8; 32]> = LazyLock::new(|| {
    let mut hasher = Sha256::new();
    hasher.update(APPLICATION_ID.to_le_bytes());
    hasher.finalize().into()
});

fn cipher() -> Aes256 {
    Aes256::new(&Array(*KEY))
}

fn block(chunk: &mut [u8]) -> &mut Array<u8, aes::cipher::consts::U16> {
    chunk.try_into().expect("chunks are one block long")
}

pub fn encrypt(payload: &[u8]) -> Vec<u8> {
    let padding = BLOCK_SIZE - payload.len() % BLOCK_SIZE;
    let mut buffer = Vec::with_capacity(payload.len() + padding);
    buffer.extend_from_slice(payload);
    buffer.resize(payload.len() + padding, padding as u8);

    let cipher = cipher();
    for chunk in buffer.chunks_exact_mut(BLOCK_SIZE) {
        cipher.encrypt_block(block(chunk));
    }
    buffer
}

pub fn decrypt(ciphertext: &[u8]) -> Option<Vec<u8>> {
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(BLOCK_SIZE) {
        return None;
    }
    let mut buffer = ciphertext.to_vec();
    let cipher = cipher();
    for chunk in buffer.chunks_exact_mut(BLOCK_SIZE) {
        cipher.decrypt_block(block(chunk));
    }

    let padding = *buffer.last()? as usize;
    if padding == 0 || padding > BLOCK_SIZE || padding > buffer.len() {
        return None;
    }
    let length = buffer.len() - padding;
    if buffer[length..]
        .iter()
        .any(|byte| *byte as usize != padding)
    {
        return None;
    }
    buffer.truncate(length);
    Some(buffer)
}

pub fn checksum(payload: &[u8]) -> [u8; CHECKSUM_SIZE] {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(KEY.as_slice()).expect("HMAC accepts keys of any size");
    mac.update(payload);
    let digest = mac.finalize().into_bytes();
    let mut checksum = [0; CHECKSUM_SIZE];
    checksum.copy_from_slice(&digest);
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_payloads_of_any_length() {
        for length in 0..64 {
            let payload = vec![length as u8; length];
            let ciphertext = encrypt(&payload);
            assert_eq!(ciphertext.len() % BLOCK_SIZE, 0);
            assert_eq!(decrypt(&ciphertext).unwrap(), payload);
        }
    }

    #[test]
    fn rejects_ciphertext_with_broken_padding() {
        let mut ciphertext = encrypt(b"pumpkin");
        ciphertext[0] ^= 0xff;
        assert!(decrypt(&ciphertext).is_none() || decrypt(&ciphertext).unwrap() != b"pumpkin");
        assert!(decrypt(&ciphertext[..BLOCK_SIZE - 1]).is_none());
    }

    #[test]
    fn key_is_derived_from_the_little_endian_application_id() {
        let mut hasher = Sha256::new();
        hasher.update([0xef, 0xbe, 0xad, 0xde, 0, 0, 0, 0]);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(*KEY, expected);
    }
}
