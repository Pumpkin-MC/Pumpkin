use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::warn;
use wasm_encoder::{CustomSection, Encode};
use wasmparser::{Parser, Payload};

pub const WASM_SIGNATURE_SECTION: &str = "wasm_signature";
pub const PUMPKIN_METADATA_SECTION: &str = "pumpkin.metadata";
pub const PUMPKIN_MARKET_PUBLIC_KEY_URL: &str =
    "https://market.pumpkinmc.org/api/v1/rest/public-key";

static MARKET_PUBLIC_KEY_CACHE: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PumpkinMetadata {
    pub marketplace_url: String,
    pub plugin_id: i64,
    pub plugin_name: String,
    pub version: String,
    pub dev_id: i64,
    pub dev_name: String,
    pub is_paid: bool,
    pub user_id: i64,
    pub license_key: Option<String>,
    pub issued_at: String,
}

/// Standard W3C Wasm-Sign signature envelope structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WasmSignatureEnvelope {
    pub version: u8,
    pub algorithm: String,
    pub public_key_hex: String,
    pub signature_hex: String,
}

/// Keypair manager for Ed25519 signing and verification.
pub struct KeyManager {
    signing_key: ed25519_dalek::SigningKey,
}

impl KeyManager {
    /// Loads or generates an Ed25519 keypair from a 32-byte secret hex.
    #[must_use]
    pub fn new(secret_hex: &str) -> Self {
        let mut seed = [0u8; 32];
        if let Ok(decoded) = hex::decode(secret_hex)
            && decoded.len() == 32
        {
            seed.copy_from_slice(&decoded);
        }
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        Self { signing_key }
    }

    #[must_use]
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }

    /// Sign WASM module using official W3C `wasmsign2::KeyPair` specification.
    #[must_use]
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.signing_key.sign(data).to_bytes().to_vec()
    }
}

fn read_leb128(bytes: &[u8]) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if shift >= 64 {
            return None;
        }
        result |= u64::from(byte & 0x7f) << shift;
        if (byte & 0x80) == 0 {
            return Some((result, i + 1));
        }
        shift += 7;
    }
    None
}

/// Strips any existing `pumpkin.metadata` and `wasm_signature` sections from a WASM binary.
pub fn strip_pumpkin_sections(wasm_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut last_valid_end = wasm_bytes.len();
    let parser = Parser::new(0);

    for payload in parser.parse_all(wasm_bytes) {
        match payload {
            Ok(Payload::Version { ref range, .. }) => {
                last_valid_end = range.end;
            }
            Ok(Payload::CustomSection(cs)) => {
                if cs.name() == PUMPKIN_METADATA_SECTION
                    || cs.name() == WASM_SIGNATURE_SECTION
                    || cs.name() == "pumpkin.signature"
                {
                } else {
                    last_valid_end = cs.range().end;
                }
            }
            Ok(p) => {
                if let Some((_, range)) = p.as_section() {
                    last_valid_end = range.end;
                }
            }
            Err(_) => {
                // Stopped parsing due to unparsable trailing bytes (e.g. invalid custom section appended)
                break;
            }
        }
    }

    Ok(wasm_bytes[..last_valid_end].to_vec())
}

/// Injects `pumpkin.metadata` and official `wasm_signature` into a WASM binary using `wasmsign2` standard section structure.
pub fn inject_pumpkin_sections(
    wasm_bytes: &[u8],
    meta: &PumpkinMetadata,
    key_manager: &KeyManager,
) -> Result<Vec<u8>, String> {
    let clean_wasm = strip_pumpkin_sections(wasm_bytes)?;

    let meta_json = serde_json::to_vec(meta).map_err(|e| e.to_string())?;

    let mut sign_payload = Vec::new();
    sign_payload.extend_from_slice(&clean_wasm);
    sign_payload.extend_from_slice(&meta_json);

    let raw_sig = key_manager.sign(&sign_payload);
    let sig_envelope = WasmSignatureEnvelope {
        version: 1,
        algorithm: "Ed25519".into(),
        public_key_hex: key_manager.public_key_hex(),
        signature_hex: hex::encode(&raw_sig),
    };
    let sig_json = serde_json::to_vec(&sig_envelope).map_err(|e| e.to_string())?;

    let meta_section = CustomSection {
        name: PUMPKIN_METADATA_SECTION.into(),
        data: meta_json.as_slice().into(),
    };

    let sig_section = CustomSection {
        name: WASM_SIGNATURE_SECTION.into(),
        data: sig_json.as_slice().into(),
    };

    let mut out = clean_wasm;
    meta_section.encode(&mut out);
    sig_section.encode(&mut out);

    Ok(out)
}

/// Verification result for inspecting a WASM binary.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub is_signed: bool,
    pub error: Option<String>,
    pub metadata: Option<PumpkinMetadata>,
    pub signature_envelope: Option<WasmSignatureEnvelope>,
    pub public_key_hex: String,
}

/// Fetches public key from market REST API, returning cached key if previously fetched.
///
/// # Errors
///
/// Returns an error if the public key cache lock cannot be acquired, if the HTTP request fails,
/// if the response body cannot be read, or if the returned key is empty.
pub fn fetch_market_public_key() -> Result<String, String> {
    let mut guard = MARKET_PUBLIC_KEY_CACHE
        .lock()
        .map_err(|e| format!("Failed to acquire public key cache lock: {e}"))?;

    if let Some(ref cached_key) = *guard {
        return Ok(cached_key.clone());
    }

    let mut response = ureq::get(PUMPKIN_MARKET_PUBLIC_KEY_URL)
        .header("User-Agent", "Pumpkin-MC")
        .call()
        .map_err(|e| format!("Failed to fetch public key from market: {e}"))?;

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read public key response: {e}"))?;

    let key = body.trim().trim_matches('"').to_string();
    if key.is_empty() {
        Err("Fetched public key is empty".into())
    } else {
        *guard = Some(key.clone());
        Ok(key)
    }
}

/// Verifies a WASM binary's pumpkin custom sections against a public key using `wasmsign2`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_pumpkin_wasm(wasm_bytes: &[u8], public_key_hex: &str) -> VerificationResult {
    let clean_wasm = match strip_pumpkin_sections(wasm_bytes) {
        Ok(w) => w,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                is_signed: false,
                error: Some(format!("Failed to parse WASM structure: {e}")),
                metadata: None,
                signature_envelope: None,
                public_key_hex: public_key_hex.to_string(),
            };
        }
    };

    let mut metadata_raw: Option<Vec<u8>> = None;
    let mut signature_raw: Option<Vec<u8>> = None;

    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_bytes) {
        if let Ok(Payload::CustomSection(cs)) = payload {
            if cs.name() == PUMPKIN_METADATA_SECTION {
                metadata_raw = Some(cs.data().to_vec());
            } else if cs.name() == WASM_SIGNATURE_SECTION || cs.name() == "pumpkin.signature" {
                signature_raw = Some(cs.data().to_vec());
            }
        }
    }

    if metadata_raw.is_none() || signature_raw.is_none() {
        let clean_wasm = strip_pumpkin_sections(wasm_bytes).unwrap_or_default();
        if clean_wasm.len() < wasm_bytes.len() {
            let trailing = &wasm_bytes[clean_wasm.len()..];

            let mut cursor = 0;
            while cursor < trailing.len() {
                if trailing[cursor] == 0 {
                    cursor += 1;
                }
                if let Some((section_len, len_bytes)) = read_leb128(&trailing[cursor..]) {
                    cursor += len_bytes;
                    let section_end = cursor + section_len as usize;
                    if section_end <= trailing.len() {
                        let section_data = &trailing[cursor..section_end];
                        if let Some((name_len, name_len_bytes)) = read_leb128(section_data)
                            && let name_start = name_len_bytes
                            && let name_end = name_start + name_len as usize
                            && name_end <= section_data.len()
                            && let Ok(name) =
                                std::str::from_utf8(&section_data[name_start..name_end])
                        {
                            let payload_data = &section_data[name_end..];
                            if name == PUMPKIN_METADATA_SECTION {
                                metadata_raw = Some(payload_data.to_vec());
                            } else if name == WASM_SIGNATURE_SECTION || name == "pumpkin.signature"
                            {
                                signature_raw = Some(payload_data.to_vec());
                            }
                        }
                    }
                    cursor = section_end;
                    continue;
                }
                break;
            }
        }
    }

    let (Some(meta_bytes), Some(sig_bytes)) = (metadata_raw, signature_raw) else {
        return VerificationResult {
            is_valid: false,
            is_signed: false,
            error: Some(
                "WASM does not contain Pumpkin Market metadata or standard wasm_signature custom sections."
                    .into(),
            ),
            metadata: None,
            signature_envelope: None,
            public_key_hex: public_key_hex.to_string(),
        };
    };

    let meta: PumpkinMetadata = match serde_json::from_slice(&meta_bytes) {
        Ok(m) => m,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                is_signed: true,
                error: Some(format!("Failed to parse pumpkin metadata: {e}")),
                metadata: None,
                signature_envelope: None,
                public_key_hex: public_key_hex.to_string(),
            };
        }
    };

    // Extract signature bytes and optional envelope
    let (raw_sig_bytes, sig_envelope) =
        if let Ok(envelope) = serde_json::from_slice::<WasmSignatureEnvelope>(&sig_bytes) {
            let Ok(sig_b) = hex::decode(&envelope.signature_hex) else {
                return VerificationResult {
                    is_valid: false,
                    is_signed: true,
                    error: Some("Invalid signature hex inside signature envelope.".into()),
                    metadata: Some(meta),
                    signature_envelope: None,
                    public_key_hex: public_key_hex.to_string(),
                };
            };
            (sig_b, Some(envelope))
        } else {
            (sig_bytes, None)
        };

    let mut sign_payload = Vec::new();
    sign_payload.extend_from_slice(&clean_wasm);
    sign_payload.extend_from_slice(&meta_bytes);

    let target_pub_key = sig_envelope.as_ref().map_or_else(
        || public_key_hex.to_string(),
        |env| env.public_key_hex.clone(),
    );

    let Ok(pub_key_bytes) = hex::decode(&target_pub_key) else {
        return VerificationResult {
            is_valid: false,
            is_signed: true,
            error: Some("Invalid public key hex format.".into()),
            metadata: Some(meta),
            signature_envelope: sig_envelope,
            public_key_hex: target_pub_key,
        };
    };
    let verifying_key = match VerifyingKey::try_from(pub_key_bytes.as_slice()) {
        Ok(vk) => vk,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                is_signed: true,
                error: Some(format!("Invalid Ed25519 public key: {e}")),
                metadata: Some(meta),
                signature_envelope: sig_envelope,
                public_key_hex: target_pub_key,
            };
        }
    };

    let signature = match Signature::from_slice(&raw_sig_bytes) {
        Ok(sig) => sig,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                is_signed: true,
                error: Some(format!("Invalid signature bytes: {e}")),
                metadata: Some(meta),
                signature_envelope: sig_envelope,
                public_key_hex: target_pub_key,
            };
        }
    };

    if let Err(e) = verifying_key.verify(&sign_payload, &signature) {
        VerificationResult {
            is_valid: false,
            is_signed: true,
            error: Some(format!("Signature verification failed: {e}")),
            metadata: Some(meta),
            signature_envelope: sig_envelope,
            public_key_hex: target_pub_key,
        }
    } else {
        VerificationResult {
            is_valid: true,
            is_signed: true,
            error: None,
            metadata: Some(meta),
            signature_envelope: sig_envelope,
            public_key_hex: target_pub_key,
        }
    }
}

/// Verifies a WASM plugin binary and logs appropriate warnings if unsigned or invalid.
pub fn verify_wasm_plugin(wasm_bytes: &[u8], path_str: &str) -> VerificationResult {
    let public_key = fetch_market_public_key().unwrap_or_default();
    let result = verify_pumpkin_wasm(wasm_bytes, &public_key);

    if !result.is_signed {
        warn!(
            "Plugin '{}' is unsigned. Unsigned plugins may come from untrusted sources.",
            path_str
        );
    } else if !result.is_valid {
        warn!(
            "Plugin '{}' signature verification failed: {}",
            path_str,
            result.error.as_deref().unwrap_or("Unknown error")
        );
    }

    result
}

/// Checks if a WASM plugin binary has a valid signature.
#[must_use]
pub fn is_wasm_signed(wasm_bytes: &[u8]) -> bool {
    let public_key = fetch_market_public_key().unwrap_or_default();
    let result = verify_pumpkin_wasm(wasm_bytes, &public_key);
    result.is_signed && result.is_valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_public_key_caching() {
        {
            let mut guard = MARKET_PUBLIC_KEY_CACHE.lock().unwrap();
            *guard = Some("cached_test_key_12345".to_string());
        };

        let key = fetch_market_public_key().expect("Should return cached key");
        assert_eq!(key, "cached_test_key_12345");

        {
            let mut guard = MARKET_PUBLIC_KEY_CACHE.lock().unwrap();
            *guard = None;
        };
    }
}
