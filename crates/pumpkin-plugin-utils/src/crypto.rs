//! Cryptographic signature verification utilities.

use crate::models::WasmSignatureEnvelope;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use thiserror::Error;

/// Cryptographic verification errors.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Public key format is invalid hex or not 32 bytes.
    #[error("Invalid public key hex format: {0}")]
    InvalidPublicKeyHex(String),
    /// Public key is not a valid Ed25519 point.
    #[error("Invalid Ed25519 public key: {0}")]
    InvalidVerifyingKey(String),
    /// Signature hex format is invalid.
    #[error("Invalid signature hex format: {0}")]
    InvalidSignatureHex(String),
    /// Signature bytes are malformed.
    #[error("Invalid Ed25519 signature bytes: {0}")]
    InvalidSignatureBytes(String),
    /// Cryptographic verification failed (signature does not match data).
    #[error("Cryptographic signature verification failed: {0}")]
    VerificationFailed(String),
}

/// Verifies an Ed25519 signature over `clean_wasm + metadata_raw` against a public key.
///
/// If `expected_public_key_hex` is non-empty, the public key inside the envelope must match
/// the expected key (preventing substitution attacks).
///
/// # Errors
///
/// Returns `CryptoError` if parsing fails or if the signature does not verify.
pub fn verify_signature(
    clean_wasm: &[u8],
    metadata_raw: &[u8],
    signature_envelope: &WasmSignatureEnvelope,
    expected_public_key_hex: &str,
) -> Result<(), CryptoError> {
    // Determine the public key to verify with
    let pub_key_hex = if !expected_public_key_hex.is_empty() {
        if !signature_envelope.public_key_hex.is_empty()
            && !signature_envelope
                .public_key_hex
                .eq_ignore_ascii_case(expected_public_key_hex)
        {
            return Err(CryptoError::InvalidPublicKeyHex(format!(
                "Signature envelope public key ({}) does not match expected public key ({})",
                signature_envelope.public_key_hex, expected_public_key_hex
            )));
        }
        expected_public_key_hex
    } else if !signature_envelope.public_key_hex.is_empty() {
        &signature_envelope.public_key_hex
    } else {
        return Err(CryptoError::InvalidPublicKeyHex(
            "No public key available for signature verification".to_string(),
        ));
    };

    let pub_key_bytes =
        hex::decode(pub_key_hex).map_err(|e| CryptoError::InvalidPublicKeyHex(e.to_string()))?;

    let verifying_key = VerifyingKey::try_from(pub_key_bytes.as_slice())
        .map_err(|e| CryptoError::InvalidVerifyingKey(e.to_string()))?;

    let sig_bytes = hex::decode(&signature_envelope.signature_hex)
        .map_err(|e| CryptoError::InvalidSignatureHex(e.to_string()))?;

    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|e| CryptoError::InvalidSignatureBytes(e.to_string()))?;

    // Build the sign payload: clean WASM binary + metadata JSON
    let mut sign_payload = Vec::with_capacity(clean_wasm.len() + metadata_raw.len());
    sign_payload.extend_from_slice(clean_wasm);
    sign_payload.extend_from_slice(metadata_raw);

    verifying_key
        .verify(&sign_payload, &signature)
        .map_err(|e| CryptoError::VerificationFailed(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    #[test]
    fn signature_verification_success() {
        let seed = [42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let pub_key_hex = hex::encode(verifying_key.to_bytes());

        let clean_wasm = b"\0asm\x01\0\0\0";
        let meta_raw = br#"{"marketplace_url":"https://market.pumpkinmc.org","plugin_id":1,"plugin_name":"test","version":"0.1.0","dev_id":1,"dev_name":"dev","is_paid":true,"user_id":100,"license_key":"KEY-123","issued_at":"2026-08-17"}"#;

        let mut payload = Vec::new();
        payload.extend_from_slice(clean_wasm);
        payload.extend_from_slice(meta_raw);

        let sig = signing_key.sign(&payload);
        let env = WasmSignatureEnvelope {
            version: 1,
            algorithm: "Ed25519".to_string(),
            public_key_hex: pub_key_hex.clone(),
            signature_hex: hex::encode(sig.to_bytes()),
        };

        let res = verify_signature(clean_wasm, meta_raw, &env, &pub_key_hex);
        assert!(res.is_ok());
    }

    #[test]
    fn signature_verification_tamper_fails() {
        let seed = [42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let pub_key_hex = hex::encode(verifying_key.to_bytes());

        let clean_wasm = b"\0asm\x01\0\0\0";
        let meta_raw = br#"{"marketplace_url":"https://market.pumpkinmc.org","plugin_id":1,"plugin_name":"test","version":"0.1.0","dev_id":1,"dev_name":"dev","is_paid":true,"user_id":100,"license_key":"KEY-123","issued_at":"2026-08-17"}"#;

        let mut payload = Vec::new();
        payload.extend_from_slice(clean_wasm);
        payload.extend_from_slice(meta_raw);

        let sig = signing_key.sign(&payload);
        let env = WasmSignatureEnvelope {
            version: 1,
            algorithm: "Ed25519".to_string(),
            public_key_hex: pub_key_hex.clone(),
            signature_hex: hex::encode(sig.to_bytes()),
        };

        // Tamper with wasm bytes
        let tampered_wasm = b"\0asm\x01\0\0\x01";
        let res = verify_signature(tampered_wasm, meta_raw, &env, &pub_key_hex);
        assert!(res.is_err());

        // Tamper with metadata bytes (e.g. altered user_id)
        let tampered_meta = br#"{"marketplace_url":"https://market.pumpkinmc.org","plugin_id":1,"plugin_name":"test","version":"0.1.0","dev_id":1,"dev_name":"dev","is_paid":true,"user_id":999,"license_key":"KEY-123","issued_at":"2026-08-17"}"#;
        let res2 = verify_signature(clean_wasm, tampered_meta, &env, &pub_key_hex);
        assert!(res2.is_err());
    }
}
