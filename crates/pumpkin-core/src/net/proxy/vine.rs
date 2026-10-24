use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use arc_swap::ArcSwap;
use bytes::{BufMut, BytesMut};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use pumpkin_config::networking::proxy::VineConfig;
use pumpkin_protocol::{
    Property, java::client::login::CLoginPluginRequest, java::server::login::SLoginPluginResponse,
    ser::NetworkReadExt,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::debug;

use crate::net::{GameProfile, java::pending::PendingConnection};

pub const VINE_PLAYER_INFO_CHANNEL: &str = "vine:player_info";
pub const VINE_FORWARDING_VERSION: i32 = 1;
pub const MAX_TIMESTAMP_DRIFT_SECS: i64 = 30;

#[derive(Error, Debug)]
pub enum VineError {
    #[error("No response data received")]
    NoData,
    #[error("Vine response data too short (minimum 89 bytes)")]
    DataTooShort,
    #[error("No public key or secret configured for Vine proxy")]
    MissingKeyConfig,
    #[error("Invalid Ed25519 public key")]
    InvalidPublicKey,
    #[error("Failed to verify Ed25519 signature")]
    InvalidSignature,
    #[error("Failed to read forward version")]
    FailedReadForwardVersion,
    #[error("Unsupported forwarding version {0}. Expected {1}")]
    UnsupportedForwardVersion(i32, i32),
    #[error("Vine timestamp expired or desynchronized: skew of {0}s exceeds limit of {1}s")]
    TimestampExpired(i64, i64),
    #[error("Vine challenge nonce mismatch")]
    ChallengeMismatch,
    #[error("Missing expected challenge from pending connection")]
    MissingChallenge,
    #[error("Failed to read address")]
    FailedReadAddress,
    #[error("Failed to parse address")]
    FailedParseAddress,
    #[error("Failed to read game profile name")]
    FailedReadProfileName,
    #[error("Failed to read game profile UUID")]
    FailedReadProfileUUID,
    #[error("Failed to read game profile properties")]
    FailedReadProfileProperties,
}

/// Initiates Vine modern forwarding handshake by sending a `CLoginPluginRequest`
/// with a unique 16-byte challenge nonce to protect against replay attacks.
pub async fn vine_login(connection: &mut PendingConnection) {
    let message_id: i32 = rand::random();
    let challenge: [u8; 16] = rand::random();

    let mut buf = BytesMut::with_capacity(17);
    buf.put_u8(VINE_FORWARDING_VERSION as u8);
    buf.put_slice(&challenge);

    connection.vine_challenge = Some(challenge);

    connection
        .send_packet_now(&CLoginPluginRequest::new(
            message_id.into(),
            VINE_PLAYER_INFO_CHANNEL,
            &buf,
        ))
        .await;
}

/// Derives the Ed25519 verifying (public) key from configuration.
/// If `public_key` is provided in hex, parses it directly (32 bytes).
/// If `secret` is provided, computes SHA-256 to derive the 32-byte Ed25519 seed
/// and gets the corresponding public key.
pub fn get_verifying_key(config: &VineConfig) -> Result<VerifyingKey, VineError> {
    if !config.public_key.trim().is_empty() {
        let hex_str = config.public_key.trim();
        let bytes = hex::decode(hex_str).map_err(|_| VineError::InvalidPublicKey)?;
        if bytes.len() != 32 {
            return Err(VineError::InvalidPublicKey);
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        VerifyingKey::from_bytes(&key_bytes).map_err(|_| VineError::InvalidPublicKey)
    } else if !config.secret.trim().is_empty() {
        let secret = config.secret.trim();
        let seed: [u8; 32] = if secret.len() == 64
            && let Ok(decoded) = hex::decode(secret)
        {
            let mut s = [0u8; 32];
            s.copy_from_slice(&decoded);
            s
        } else {
            Sha256::digest(secret.as_bytes()).into()
        };
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        Ok(signing_key.verifying_key())
    } else {
        Err(VineError::MissingKeyConfig)
    }
}

fn read_game_profile(read: impl Read) -> Result<GameProfile, VineError> {
    let mut read = read;
    let id = read
        .get_uuid()
        .map_err(|_| VineError::FailedReadProfileUUID)?;

    let name = read
        .get_str()
        .map_err(|_| VineError::FailedReadProfileName)?;

    let properties = read
        .get_list(|data| {
            let name = data.get_str()?;
            let value = data.get_str()?;
            let signature = data.get_option(NetworkReadExt::get_str)?;

            Ok(Property {
                name,
                value,
                signature,
            })
        })
        .map_err(|_| VineError::FailedReadProfileProperties)?;

    Ok(GameProfile {
        id,
        name: name.into_string(),
        properties: ArcSwap::new(Arc::from(properties)),
        profile_actions: None,
    })
}

pub fn receive_vine_plugin_response(
    port: u16,
    config: &VineConfig,
    response: SLoginPluginResponse,
    expected_challenge: Option<[u8; 16]>,
) -> Result<(GameProfile, SocketAddr), VineError> {
    debug!("Received Vine plugin response");
    let expected_challenge = expected_challenge.ok_or(VineError::MissingChallenge)?;

    if let Some(data) = response.data {
        // Minimum size: 64 (Ed25519 signature) + 1 (VarInt version) + 8 (u64 timestamp) + 16 (challenge)
        if data.len() < 64 + 1 + 8 + 16 {
            return Err(VineError::DataTooShort);
        }

        let (sig_bytes, mut payload) = data.split_at(64);
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(sig_bytes);
        let signature = Signature::from_bytes(&sig_arr);

        // Verify Ed25519 signature against proxy's public key
        let verifying_key = get_verifying_key(config)?;
        verifying_key
            .verify(payload, &signature)
            .map_err(|_| VineError::InvalidSignature)?;

        // 1. Version
        let version = payload
            .get_var_int()
            .map_err(|_| VineError::FailedReadForwardVersion)?;
        if version.0 != VINE_FORWARDING_VERSION {
            return Err(VineError::UnsupportedForwardVersion(
                version.0,
                VINE_FORWARDING_VERSION,
            ));
        }

        // 2. Timestamp (u64 big-endian)
        let mut ts_bytes = [0u8; 8];
        payload
            .read_exact(&mut ts_bytes)
            .map_err(|_| VineError::DataTooShort)?;
        let timestamp = u64::from_be_bytes(ts_bytes);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let drift = (now as i64) - (timestamp as i64);
        if drift.abs() > MAX_TIMESTAMP_DRIFT_SECS {
            return Err(VineError::TimestampExpired(drift, MAX_TIMESTAMP_DRIFT_SECS));
        }

        // 3. Challenge nonce
        let mut challenge = [0u8; 16];
        payload
            .read_exact(&mut challenge)
            .map_err(|_| VineError::DataTooShort)?;
        if challenge != expected_challenge {
            return Err(VineError::ChallengeMismatch);
        }

        // 4. Client IP
        let addr_str = payload
            .get_str()
            .map_err(|_| VineError::FailedReadAddress)?;
        let ip: IpAddr = addr_str
            .parse()
            .map_err(|_| VineError::FailedParseAddress)?;
        let socket_addr = SocketAddr::new(ip, port);

        // 5. GameProfile (UUID, Name, Properties)
        let profile = read_game_profile(&mut payload)?;

        Ok((profile, socket_addr))
    } else {
        Err(VineError::NoData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use pumpkin_protocol::codec::var_int::VarInt;
    use pumpkin_protocol::ser::NetworkWriteExt;
    use uuid::Uuid;

    #[test]
    fn vine_key_derivation() {
        let config = VineConfig {
            enabled: true,
            public_key: String::new(),
            secret: "my-super-secret-seed".to_string(),
        };
        let key = get_verifying_key(&config).unwrap();
        let pub_hex = hex::encode(key.to_bytes());
        let config_with_hex = VineConfig {
            enabled: true,
            public_key: pub_hex,
            secret: String::new(),
        };
        let key2 = get_verifying_key(&config_with_hex).unwrap();
        assert_eq!(key, key2);
    }

    #[test]
    fn vine_plugin_response_verification() {
        let secret = "test-vine-secret";
        let seed: [u8; 32] = Sha256::digest(secret.as_bytes()).into();
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let pub_hex = hex::encode(verifying_key.to_bytes());

        let config = VineConfig {
            enabled: true,
            public_key: pub_hex,
            secret: String::new(),
        };

        let challenge = [42u8; 16];
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Build payload
        let mut payload = Vec::new();
        VarInt(VINE_FORWARDING_VERSION)
            .encode(&mut payload)
            .unwrap();
        payload.extend_from_slice(&now.to_be_bytes());
        payload.extend_from_slice(&challenge);
        payload.write_string("127.0.0.1").unwrap();
        let test_uuid = Uuid::new_v4();
        payload.extend_from_slice(test_uuid.as_bytes());
        payload.write_string("Steve").unwrap();
        VarInt(0).encode(&mut payload).unwrap(); // 0 properties

        // Sign payload
        let signature = signing_key.sign(&payload);
        let mut full_packet_data = Vec::new();
        full_packet_data.extend_from_slice(&signature.to_bytes());
        full_packet_data.extend_from_slice(&payload);

        let response = SLoginPluginResponse {
            message_id: VarInt(1),
            data: Some(full_packet_data.into_boxed_slice()),
        };

        let result = receive_vine_plugin_response(25565, &config, response, Some(challenge));
        assert!(result.is_ok());
        let (profile, addr) = result.unwrap();
        assert_eq!(profile.name, "Steve");
        assert_eq!(profile.id, test_uuid);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn vine_plugin_response_rejects_tampered_challenge() {
        let secret = "test-vine-secret";
        let seed: [u8; 32] = Sha256::digest(secret.as_bytes()).into();
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let pub_hex = hex::encode(signing_key.verifying_key().to_bytes());

        let config = VineConfig {
            enabled: true,
            public_key: pub_hex,
            secret: String::new(),
        };

        let challenge = [42u8; 16];
        let wrong_challenge = [99u8; 16];
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut payload = Vec::new();
        VarInt(VINE_FORWARDING_VERSION)
            .encode(&mut payload)
            .unwrap();
        payload.extend_from_slice(&now.to_be_bytes());
        payload.extend_from_slice(&challenge);
        payload.write_string("127.0.0.1").unwrap();
        payload.extend_from_slice(Uuid::new_v4().as_bytes());
        payload.write_string("Steve").unwrap();
        VarInt(0).encode(&mut payload).unwrap();

        let signature = signing_key.sign(&payload);
        let mut full_packet_data = Vec::new();
        full_packet_data.extend_from_slice(&signature.to_bytes());
        full_packet_data.extend_from_slice(&payload);

        let response = SLoginPluginResponse {
            message_id: VarInt(1),
            data: Some(full_packet_data.into_boxed_slice()),
        };

        let result = receive_vine_plugin_response(25565, &config, response, Some(wrong_challenge));
        assert!(matches!(result, Err(VineError::ChallengeMismatch)));
    }
}
