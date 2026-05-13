//! # JWT Verifier for Minecraft: Bedrock Edition
//!
//! This module provides the core functionality for verifying the chain of JWT tokens
//! sent by a Minecraft: Bedrock Edition client. It handles cryptographic signature
//! verification, public key extraction, and decoding of player data.

use base64::{Engine as _, engine::general_purpose};
use ecdsa::Signature;
use p384::PublicKey;
use p384::ecdsa::{VerifyingKey, signature::Verifier};
use p384::pkcs8::DecodePublicKey;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

/// Represents the claims extracted from a Minecraft Bedrock player's JWT token.
///
/// This struct contains the player's display name, UUID, and XUID.
#[derive(Debug)]
pub struct PlayerClaims {
    /// The player's display name (in-game name).
    pub display_name: String,
    /// The player's unique identifier (UUID).
    pub uuid: String,
    /// The player's Xbox User ID (XUID).
    pub xuid: String,
}

/// Represents the possible errors that can occur during JWT verification.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Indicates that a JWT token has an invalid format (not enough parts).
    #[error("Invalid token format")]
    InvalidTokenFormat,
    /// Indicates that the 'x5u' (X.509 URL) header parameter is missing from a token.
    #[error("x5u not found in header")]
    MissingX5U,
    /// Indicates a failure in Base64 decoding.
    #[error("Base64 decoding failed: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    /// Indicates a failure in parsing JSON data.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    /// Indicates a failure in building a public key from its representation.
    #[error("Public key build failed: {0}")]
    PublicKeyBuild(String),
    /// Indicates that the token was not signed by the trusted Mojang public key.
    #[error("Token not signed by trusted Mojang key")]
    MojangKeyMismatch,
    /// Indicates that the token's signature is invalid.
    #[error("Invalid signature")]
    InvalidSignature,
    /// Indicates an error related to ECDSA signature operations.
    #[error("ECDSA signature error: {0}")]
    Ecdsa(#[from] ecdsa::Error),
}

/// Decodes a Base64 URL-safe encoded string with no padding.
///
/// # Arguments
///
/// * `s` - The Base64 URL-safe encoded string to decode.
///
/// # Returns
///
/// A `Result` containing the decoded bytes or a `base64::DecodeError`.
pub fn decode_b64_url_nopad(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::URL_SAFE_NO_PAD.decode(s)
}

/// Decodes a standard Base64 encoded string.
///
/// # Arguments
///
/// * `s` - The standard Base64 encoded string to decode.
///
/// # Returns
///
/// A `Result` containing the decoded bytes or a `base64::DecodeError`.
pub fn decode_b64_standard(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(s)
}

/// Builds a P-384 public key from a Base64 encoded string.
///
/// This function supports several common public key formats.
///
/// # Arguments
///
/// * `b64` - The Base64 encoded public key.
///
/// # Returns
///
/// A `Result` containing the `p384::PublicKey` or an `AuthError`.
pub fn build_public_key_from_b64(b64: &str) -> Result<PublicKey, AuthError> {
    let bytes = decode_b64_standard(b64)?;

    if !bytes.is_empty() && bytes[0] == 0x30 {
        PublicKey::from_public_key_der(&bytes).map_err(|e| AuthError::PublicKeyBuild(e.to_string()))
    } else if bytes.len() == 97 && bytes[0] == 0x04 {
        PublicKey::from_sec1_bytes(&bytes).map_err(|e| AuthError::PublicKeyBuild(e.to_string()))
    } else if bytes.len() == 96 {
        let mut sec1 = Vec::with_capacity(97);
        sec1.push(0x04u8);
        sec1.extend_from_slice(&bytes);
        PublicKey::from_sec1_bytes(&sec1).map_err(|e| AuthError::PublicKeyBuild(e.to_string()))
    } else {
        Err(AuthError::PublicKeyBuild(format!(
            "Unsupported key format/length: {} bytes",
            bytes.len()
        )))
    }
}

/// Decodes the header of a JWT and extracts the 'x5u' (X.509 URL) value.
///
/// # Arguments
///
/// * `header_b64` - The Base64 URL-safe encoded header of the JWT.
///
/// # Returns
///
/// A `Result` containing the 'x5u' value as a string or an `AuthError`.
pub fn decode_header_get_x5u(header_b64: &str) -> Result<String, AuthError> {
    let header_bytes = decode_b64_url_nopad(header_b64)?;
    let header_json: Value = serde_json::from_slice(&header_bytes)?;
    if let Some(x5u) = header_json.get("x5u")
        && let Some(s) = x5u.as_str()
    {
        return Ok(s.to_string());
    }
    Err(AuthError::MissingX5U)
}

#[derive(Debug, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    pub kty: String,
    #[serde(default)]
    pub alg: Option<String>,
    #[serde(default)]
    pub crv: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub kid: String,
}

impl Jwk {
    pub fn to_ec_public_key(&self) -> Result<PublicKey, AuthError> {
        if self.kty != "EC" {
            return Err(AuthError::PublicKeyBuild(format!(
                "Unsupported JWK kty for EC: {}",
                self.kty
            )));
        }

        if let Some(ref crv) = self.crv
            && crv != "P-384"
        {
            return Err(AuthError::PublicKeyBuild(format!(
                "Unsupported JWK crv: {crv}"
            )));
        }

        let x = self.x.as_ref().ok_or_else(|| {
            AuthError::PublicKeyBuild("JWK missing x coordinate for EC key".into())
        })?;
        let y = self.y.as_ref().ok_or_else(|| {
            AuthError::PublicKeyBuild("JWK missing y coordinate for EC key".into())
        })?;

        let x_bytes = decode_b64_url_nopad(x)?;
        let y_bytes = decode_b64_url_nopad(y)?;

        if x_bytes.len() != 48 || y_bytes.len() != 48 {
            return Err(AuthError::PublicKeyBuild(
                "Invalid P-384 coordinate lengths".into(),
            ));
        }

        let mut sec1 = Vec::with_capacity(97);
        sec1.push(0x04u8);
        sec1.extend_from_slice(&x_bytes);
        sec1.extend_from_slice(&y_bytes);

        PublicKey::from_sec1_bytes(&sec1).map_err(|e| AuthError::PublicKeyBuild(e.to_string()))
    }

    pub fn to_rsa_public_key(&self) -> Result<rsa::RsaPublicKey, AuthError> {
        if self.kty != "RSA" {
            return Err(AuthError::PublicKeyBuild(format!(
                "Unsupported JWK kty for RSA: {}",
                self.kty
            )));
        }

        let n = self
            .n
            .as_ref()
            .ok_or_else(|| AuthError::PublicKeyBuild("JWK missing n modulus for RSA key".into()))?;
        let e = self.e.as_ref().ok_or_else(|| {
            AuthError::PublicKeyBuild("JWK missing e exponent for RSA key".into())
        })?;

        let n_bytes = decode_b64_url_nopad(n)?;
        let e_bytes = decode_b64_url_nopad(e)?;

        let n_boxed = crypto_bigint::BoxedUint::from_be_slice_vartime(&n_bytes);
        let e_boxed = crypto_bigint::BoxedUint::from_be_slice_vartime(&e_bytes);

        rsa::RsaPublicKey::new(n_boxed, e_boxed)
            .map_err(|err| AuthError::PublicKeyBuild(err.to_string()))
    }
}

pub const OIDC_ISSUER: &str = "https://identity.minecraft-services.net";
pub const OIDC_AUDIENCE: &str = "api://auth-minecraft-services/multiplayer";
pub const OIDC_DISCOVERY_URL: &str =
    "https://client.discovery.minecraft-services.net/api/v1.0/discovery/MinecraftPE/builds/1.0.0.0";

pub fn fetch_oidc_jwks() -> Result<(String, Jwks), AuthError> {
    let discovery: Value = ureq::get(OIDC_DISCOVERY_URL)
        .call()
        .map_err(|e| AuthError::PublicKeyBuild(e.to_string()))?
        .body_mut()
        .read_to_string()
        .map_err(|e| AuthError::PublicKeyBuild(format!("Failed to read response: {e}")))
        .and_then(|s| {
            serde_json::from_str(&s).map_err(|e| AuthError::PublicKeyBuild(e.to_string()))
        })?;

    let service_uri = discovery
        .get("result")
        .and_then(|v| v.get("serviceEnvironments"))
        .and_then(|v| v.get("auth"))
        .and_then(|v| v.get("prod"))
        .and_then(|v| v.get("serviceUri"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::PublicKeyBuild("Discovery missing serviceUri".into()))?;

    let openid_config_url = format!("{service_uri}/.well-known/openid-configuration");
    let openid_config: Value = ureq::get(&openid_config_url)
        .call()
        .map_err(|e| AuthError::PublicKeyBuild(e.to_string()))?
        .body_mut()
        .read_to_string()
        .map_err(|e| AuthError::PublicKeyBuild(format!("Failed to read response: {e}")))
        .and_then(|s| {
            serde_json::from_str(&s).map_err(|e| AuthError::PublicKeyBuild(e.to_string()))
        })?;

    let jwks_uri = openid_config
        .get("jwks_uri")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::PublicKeyBuild("OpenID config missing jwks_uri".into()))?;

    let issuer = openid_config
        .get("issuer")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::PublicKeyBuild("OpenID config missing issuer".into()))?
        .to_string();

    let jwks: Jwks = ureq::get(jwks_uri)
        .call()
        .map_err(|e| AuthError::PublicKeyBuild(e.to_string()))?
        .body_mut()
        .read_to_string()
        .map_err(|e| AuthError::PublicKeyBuild(format!("Failed to read response: {e}")))
        .and_then(|s| {
            serde_json::from_str(&s).map_err(|e| AuthError::PublicKeyBuild(e.to_string()))
        })?;

    Ok((issuer, jwks))
}

pub fn verify_oidc_token(
    token: &str,
    expected_issuer: &str,
    jwks: &Jwks,
) -> Result<PlayerClaims, AuthError> {
    let mut parts = token.split('.');
    let header_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;
    let payload_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;
    let signature_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;

    let signing_input = format!("{header_b64}.{payload_b64}");

    let header_bytes = decode_b64_url_nopad(header_b64)?;
    let header: Value = serde_json::from_slice(&header_bytes)?;
    let kid = header
        .get("kid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::PublicKeyBuild("OIDC header missing kid".into()))?;
    let alg = header
        .get("alg")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::PublicKeyBuild("OIDC header missing alg".into()))?;

    let jwk = jwks
        .keys
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| AuthError::PublicKeyBuild(format!("Key not found in JWKS: {kid}")))?;

    if alg == "ES384" {
        verify_es384_signature(&jwk.to_ec_public_key()?, &signing_input, signature_b64)?;
    } else if alg == "RS256" {
        verify_rs256_signature(jwk, &signing_input, signature_b64)?;
    } else {
        return Err(AuthError::PublicKeyBuild(format!(
            "Unsupported OIDC algorithm: {alg}"
        )));
    }

    let payload_bytes = decode_b64_url_nopad(payload_b64)?;
    let v: Value = serde_json::from_slice(&payload_bytes)?;

    verify_oidc_claims(&v, Some(expected_issuer))?;

    Ok(extract_oidc_player_claims(&v))
}

pub fn verify_oidc_token_self_signed(token: &str) -> Result<PlayerClaims, AuthError> {
    let mut parts = token.split('.');
    let header_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;
    let payload_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;
    let signature_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;

    let signing_input = format!("{header_b64}.{payload_b64}");

    let header_bytes = decode_b64_url_nopad(header_b64)?;
    let header: Value = serde_json::from_slice(&header_bytes)?;
    let alg = header
        .get("alg")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::PublicKeyBuild("OIDC header missing alg".into()))?;
    if alg == "ES384" {
        verify_es384_signature(
            &PublicKey::from_public_key_der(&decode_b64_standard(&decode_header_get_x5u(
                header_b64,
            )?)?)
            .map_err(|_| AuthError::PublicKeyBuild("Couldn't build public key from x5u".into()))?,
            &signing_input,
            signature_b64,
        )?;
    } else {
        return Err(AuthError::PublicKeyBuild(format!(
            "Unsupported OIDC algorithm (for self-signed): {alg}"
        )));
    }

    let payload_bytes = decode_b64_url_nopad(payload_b64)?;
    let v: Value = serde_json::from_slice(&payload_bytes)?;

    verify_oidc_claims(&v, None)?;

    Ok(extract_oidc_player_claims(&v))
}

fn verify_es384_signature(
    public_key: &PublicKey,
    signing_input: &str,
    signature_b64: &str,
) -> Result<(), AuthError> {
    let verifying_key = VerifyingKey::from(public_key);

    let sig_bytes = decode_b64_url_nopad(signature_b64)?;
    let signature = Signature::from_slice(&sig_bytes).map_err(|_| AuthError::InvalidSignature)?;

    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .map_err(|_| AuthError::InvalidSignature)
}

fn verify_rs256_signature(
    jwk: &Jwk,
    signing_input: &str,
    signature_b64: &str,
) -> Result<(), AuthError> {
    use rsa::pkcs1v15::VerifyingKey as RsaVerifyingKey;
    use rsa::signature::Verifier;
    use sha2::Sha256;

    let public_key = jwk.to_rsa_public_key()?;
    let sig_bytes = decode_b64_url_nopad(signature_b64)?;

    let verifying_key: RsaVerifyingKey<Sha256> = RsaVerifyingKey::new(public_key);
    let signature = rsa::pkcs1v15::Signature::try_from(sig_bytes.as_slice())
        .map_err(|_| AuthError::InvalidSignature)?;

    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .map_err(|_| AuthError::InvalidSignature)
}

fn verify_oidc_claims(payload: &Value, expected_issuer: Option<&str>) -> Result<(), AuthError> {
    if let Some(expected_issuer) = expected_issuer {
        let iss = payload
            .get("iss")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::PublicKeyBuild("OIDC payload missing iss".into()))?;
        if iss != expected_issuer {
            return Err(AuthError::PublicKeyBuild(format!(
                "OIDC issuer mismatch: expected {expected_issuer}, got {iss}"
            )));
        }
    }

    let aud = payload
        .get("aud")
        .ok_or_else(|| AuthError::PublicKeyBuild("OIDC payload missing aud".into()))?;
    let aud_match = aud.as_str().map_or_else(
        || {
            aud.as_array()
                .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(OIDC_AUDIENCE)))
        },
        |s| s == OIDC_AUDIENCE,
    );
    if !aud_match {
        return Err(AuthError::PublicKeyBuild(format!(
            "OIDC audience mismatch: expected {OIDC_AUDIENCE}, got {aud:?}"
        )));
    }

    let exp = payload
        .get("exp")
        .and_then(Value::as_u64)
        .ok_or_else(|| AuthError::PublicKeyBuild("OIDC payload missing exp".into()))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if now > exp {
        return Err(AuthError::PublicKeyBuild("OIDC token expired".into()));
    }

    Ok(())
}

pub fn extract_oidc_token_player_claims(token: &str) -> Result<PlayerClaims, AuthError> {
    let mut parts = token.split('.');
    parts.next().ok_or(AuthError::InvalidTokenFormat)?;
    let payload_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;

    let payload_bytes = decode_b64_url_nopad(payload_b64)?;
    let v: Value = serde_json::from_slice(&payload_bytes)?;

    Ok(extract_oidc_player_claims(&v))
}

fn extract_oidc_player_claims(payload: &Value) -> PlayerClaims {
    let display_name = payload
        .get("xname")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let xuid = payload
        .get("xid")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let uuid = if xuid.is_empty() {
        let leguuid = payload
            .get("leguuid")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        if leguuid.is_empty() {
            let input = format!("pocket-auth-1-xuid:{xuid}");
            let mut bytes = *md5::compute(input.as_bytes());
            bytes[6] = (bytes[6] & 0x0f) | 0x30;
            bytes[8] = (bytes[8] & 0x3f) | 0x80;
            uuid::Uuid::from_bytes(bytes).to_string()
        } else {
            leguuid
        }
    } else {
        xuid_to_uuid(&xuid)
    };

    PlayerClaims {
        display_name,
        uuid,
        xuid,
    }
}

fn xuid_to_uuid(xuid: &str) -> String {
    let input = format!("pocket-auth-1-xuid:{xuid}");
    let mut bytes = *md5::compute(input.as_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}
