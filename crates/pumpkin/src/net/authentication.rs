use std::{collections::HashMap, net::IpAddr};

use base64::{Engine, engine::general_purpose};
use pumpkin_config::{AuthenticationConfig, networking::auth::TextureConfig};
use pumpkin_protocol::Property;
use rsa::RsaPublicKey;
use rsa::pkcs8::DecodePublicKey;
use serde::Deserialize;
use thiserror::Error;
use ureq::http::{StatusCode, Uri};
use uuid::Uuid;

use super::GameProfile;

#[derive(Deserialize, Clone, Debug)]
#[expect(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTextures {
    timestamp: i64,
    profile_id: Uuid,
    profile_name: String,
    // Mojang always sends this, but third-party auth servers (drasl, Blessing Skin, ...)
    // omit it. It is unused here, so default it instead of failing to parse the profile.
    #[serde(default)]
    signature_required: bool,
    textures: HashMap<String, Texture>,
}

#[derive(Deserialize, Clone, Debug)]
#[expect(dead_code)]
pub struct Texture {
    url: String,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonPublicKey {
    pub public_key: String,
}
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MojangPublicKeys {
    pub profile_property_keys: Vec<JsonPublicKey>,
    pub player_certificate_keys: Vec<JsonPublicKey>,
    pub authentication_keys: Option<Vec<JsonPublicKey>>,
}

const MOJANG_AUTHENTICATION_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}";
const MOJANG_PREVENT_PROXY_AUTHENTICATION_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}&ip={ip}";
const MOJANG_SERVICES_URL: &str = "https://api.minecraftservices.com/";
const MOJANG_PROFILE_BY_NAME_URL: &str =
    "https://api.mojang.com/users/profiles/minecraft/{username}";
const MOJANG_PROFILE_BY_UUID_URL: &str =
    "https://sessionserver.mojang.com/session/minecraft/profile/{uuid}?unsigned=false";

fn create_agent(auth_config: &AuthenticationConfig) -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_millis(
            auth_config.connect_timeout as u64,
        )))
        .timeout_recv_response(Some(std::time::Duration::from_millis(
            auth_config.read_timeout as u64,
        )))
        .build();
    config.into()
}

fn format_auth_url(url_template: &str, username: &str, server_hash: &str, ip: &IpAddr) -> String {
    url_template
        .replace("{username}", username)
        .replace("{server_hash}", server_hash)
        .replace("{ip}", &ip.to_string())
}

/// Sends a GET request to Mojang's (or custom/fallback) authentication servers to verify a client's Minecraft account.
///
/// **Purpose:**
///
/// This function is used to ensure that a client connecting to the server has a valid, premium Minecraft account.
/// If the primary authentication server is down or unreachable, it falls back to trying configured fallback servers.
///
/// **How it Works:**
///
/// 1. A client with a premium account sends a login request to the session server.
/// 2. Session servers verify the client's credentials and add the player to their server session list.
/// 3. Pumpkin attempts to authenticate the player against the primary auth server, falling back to secondary auth servers if the primary is down.
///
/// See <https://pumpkinmc.org/developer/networking/authentication>
pub fn authenticate(
    username: &str,
    server_hash: &str,
    ip: &IpAddr,
    auth_config: &AuthenticationConfig,
) -> Result<GameProfile, AuthError> {
    let primary_url = if auth_config.prevent_proxy_connections {
        auth_config
            .prevent_proxy_connection_auth_url
            .as_deref()
            .unwrap_or(MOJANG_PREVENT_PROXY_AUTHENTICATION_URL)
    } else {
        auth_config
            .url
            .as_deref()
            .unwrap_or(MOJANG_AUTHENTICATION_URL)
    };

    let mut candidate_urls = Vec::with_capacity(1 + auth_config.fallbacks.len());
    candidate_urls.push(primary_url);
    for fallback in &auth_config.fallbacks {
        candidate_urls.push(fallback.as_str());
    }

    let agent = create_agent(auth_config);

    let mut unverified_count = 0;
    let mut last_unknown_status = None;

    for url_template in candidate_urls {
        let address = format_auth_url(url_template, username, server_hash, ip);

        let mut response = match agent.get(&address).call() {
            Ok(resp) => resp,
            Err(err) => {
                tracing::warn!(
                    "Authentication server at '{address}' is down or unreachable: {err}"
                );
                continue;
            }
        };

        let status = response.status();
        if status.is_server_error() {
            tracing::warn!("Authentication server at '{address}' returned server error: {status}");
            continue;
        }

        match status {
            StatusCode::OK => match response.body_mut().read_json::<GameProfile>() {
                Ok(profile) => return Ok(profile),
                Err(err) => {
                    tracing::warn!("Failed to parse GameProfile response from '{address}': {err}");
                }
            },
            StatusCode::NO_CONTENT => {
                unverified_count += 1;
            }
            other => {
                last_unknown_status = Some(other);
            }
        }
    }

    if unverified_count > 0 {
        Err(AuthError::UnverifiedUsername)
    } else if let Some(status) = last_unknown_status {
        Err(AuthError::UnknownStatusCode(status))
    } else {
        Err(AuthError::FailedResponse)
    }
}

pub fn validate_textures(property: &Property, config: &TextureConfig) -> Result<(), TextureError> {
    let from64 = general_purpose::STANDARD
        .decode(property.value.as_bytes())
        .map_err(|e| TextureError::DecodeError(e.to_string()))?;
    let textures: ProfileTextures =
        serde_json::from_slice(&from64).map_err(|e| TextureError::JSONError(e.to_string()))?;
    for texture in textures.textures {
        let url = texture
            .1
            .url
            .parse()
            .map_err(|_| TextureError::InvalidURL)?;
        is_texture_url_valid(&url, config)?;
    }
    Ok(())
}

pub fn is_texture_url_valid(url: &Uri, config: &TextureConfig) -> Result<(), TextureError> {
    let Some(scheme) = url.scheme() else {
        return Err(TextureError::InvalidURL);
    };
    if !config
        .allowed_url_schemes
        .iter()
        .any(|allowed_scheme| scheme.as_str().ends_with(allowed_scheme))
    {
        return Err(TextureError::DisallowedUrlScheme(scheme.to_string()));
    }
    let Some(domain) = url.authority() else {
        return Err(TextureError::InvalidURL);
    };
    if !config
        .allowed_url_domains
        .iter()
        .any(|allowed_domain| domain.as_str().ends_with(allowed_domain))
    {
        return Err(TextureError::DisallowedUrlDomain(domain.to_string()));
    }
    Ok(())
}

pub fn fetch_mojang_public_keys(
    auth_config: &AuthenticationConfig,
) -> Result<Vec<RsaPublicKey>, AuthError> {
    let services_url = auth_config
        .services_url
        .as_deref()
        .unwrap_or(MOJANG_SERVICES_URL);

    let url = format!("{services_url}/publickeys");

    let agent = create_agent(auth_config);
    let mut response = agent
        .get(&url)
        .call()
        .map_err(|_| AuthError::FailedResponse)?;

    match response.status() {
        StatusCode::OK => {}
        StatusCode::NO_CONTENT => Err(AuthError::FailedResponse)?,
        other => Err(AuthError::UnknownStatusCode(other))?,
    }

    let public_keys: MojangPublicKeys = response
        .body_mut()
        .read_json()
        .map_err(|_| AuthError::FailedParse)?;

    let as_rsa_keys = public_keys
        .player_certificate_keys
        .into_iter()
        .map(|key| {
            let decoded_key = general_purpose::STANDARD
                .decode(key.public_key.as_bytes())
                .map_err(|_| AuthError::FailedParse)?;
            RsaPublicKey::from_public_key_der(&decoded_key).map_err(|_| AuthError::FailedParse)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(as_rsa_keys)
}

#[derive(Deserialize, Clone, Debug)]
struct MojangProfileByNameResponse {
    id: String,
    name: String,
}

pub fn lookup_profile_by_name(
    name: &str,
    auth_config: &AuthenticationConfig,
) -> Result<Option<(Uuid, String)>, AuthError> {
    let primary_url = auth_config
        .profile_by_name_url
        .as_deref()
        .unwrap_or(MOJANG_PROFILE_BY_NAME_URL);

    let mut candidate_urls = Vec::with_capacity(1 + auth_config.profile_by_name_fallbacks.len());
    candidate_urls.push(primary_url);
    for fallback in &auth_config.profile_by_name_fallbacks {
        candidate_urls.push(fallback.as_str());
    }

    let agent = create_agent(auth_config);

    let mut not_found_count = 0;
    let mut last_unknown_status = None;

    for url_template in candidate_urls {
        let address = url_template.replace("{username}", name);

        let mut response = match agent.get(&address).call() {
            Ok(resp) => resp,
            Err(err) => {
                tracing::warn!(
                    "Profile lookup server at '{address}' is down or unreachable: {err}"
                );
                continue;
            }
        };

        let status = response.status();
        if status.is_server_error() {
            tracing::warn!("Profile lookup server at '{address}' returned server error: {status}");
            continue;
        }

        match status {
            StatusCode::OK => match response
                .body_mut()
                .read_json::<MojangProfileByNameResponse>()
            {
                Ok(profile) => {
                    let parsed_uuid =
                        Uuid::parse_str(&profile.id).map_err(|_| AuthError::FailedParse)?;
                    return Ok(Some((parsed_uuid, profile.name)));
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to parse profile by name response from '{address}': {err}"
                    );
                }
            },
            StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => {
                not_found_count += 1;
            }
            other => {
                last_unknown_status = Some(other);
            }
        }
    }

    if not_found_count > 0 {
        Ok(None)
    } else if let Some(status) = last_unknown_status {
        Err(AuthError::UnknownStatusCode(status))
    } else {
        Err(AuthError::FailedResponse)
    }
}

pub fn fetch_profile_by_uuid(
    uuid: Uuid,
    auth_config: &AuthenticationConfig,
) -> Result<Option<GameProfile>, AuthError> {
    let primary_url = auth_config
        .profile_by_uuid_url
        .as_deref()
        .unwrap_or(MOJANG_PROFILE_BY_UUID_URL);

    let mut candidate_urls = Vec::with_capacity(1 + auth_config.profile_by_uuid_fallbacks.len());
    candidate_urls.push(primary_url);
    for fallback in &auth_config.profile_by_uuid_fallbacks {
        candidate_urls.push(fallback.as_str());
    }

    let agent = create_agent(auth_config);

    let mut not_found_count = 0;
    let mut last_unknown_status = None;

    let uuid_simple = uuid.simple().to_string();

    for url_template in candidate_urls {
        let address = url_template
            .replace("{uuid}", &uuid_simple)
            .replace("{uuid_hyphenated}", &uuid.to_string());

        let mut response = match agent.get(&address).call() {
            Ok(resp) => resp,
            Err(err) => {
                tracing::warn!("Profile fetch server at '{address}' is down or unreachable: {err}");
                continue;
            }
        };

        let status = response.status();
        if status.is_server_error() {
            tracing::warn!("Profile fetch server at '{address}' returned server error: {status}");
            continue;
        }

        match status {
            StatusCode::OK => match response.body_mut().read_json::<GameProfile>() {
                Ok(profile) => return Ok(Some(profile)),
                Err(err) => {
                    tracing::warn!("Failed to parse GameProfile response from '{address}': {err}");
                }
            },
            StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => {
                not_found_count += 1;
            }
            other => {
                last_unknown_status = Some(other);
            }
        }
    }

    if not_found_count > 0 {
        Ok(None)
    } else if let Some(status) = last_unknown_status {
        Err(AuthError::UnknownStatusCode(status))
    } else {
        Err(AuthError::FailedResponse)
    }
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Authentication servers are down")]
    FailedResponse,
    #[error("Failed to verify username")]
    UnverifiedUsername,
    #[error("You are banned from Authentication servers")]
    Banned,
    #[error("Texture Error {0}")]
    TextureError(TextureError),
    #[error("You have disallowed actions from Authentication servers")]
    DisallowedAction,
    #[error("Failed to parse JSON into Game Profile")]
    FailedParse,
    #[error("Unknown Status Code {0}")]
    UnknownStatusCode(StatusCode),
}

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Invalid URL")]
    InvalidURL,
    #[error("Invalid URL scheme for player texture: {0}")]
    DisallowedUrlScheme(String),
    #[error("Invalid URL domain for player texture: {0}")]
    DisallowedUrlDomain(String),
    #[error("Failed to decode base64 player texture: {0}")]
    DecodeError(String),
    #[error("Failed to parse JSON from player texture: {0}")]
    JSONError(String),
}

#[cfg(test)]
mod tests {
    use super::ProfileTextures;

    // Third-party auth servers (drasl, Blessing Skin, littleskin.cn) don't send
    // `signatureRequired`. The profile must still parse. See issue #301.
    #[test]
    fn parses_profile_without_signature_required() {
        let json = r#"{
            "timestamp": 0,
            "profileId": "069a79f444e94726a5befca90e38aaf5",
            "profileName": "Notch",
            "textures": {}
        }"#;
        let profile: ProfileTextures =
            serde_json::from_slice(json.as_bytes()).expect("profile should parse");
        assert!(!profile.signature_required);
    }

    #[test]
    fn parses_profile_with_signature_required() {
        let json = r#"{
            "timestamp": 0,
            "profileId": "069a79f444e94726a5befca90e38aaf5",
            "profileName": "Notch",
            "signatureRequired": true,
            "textures": {}
        }"#;
        let profile: ProfileTextures =
            serde_json::from_slice(json.as_bytes()).expect("profile should parse");
        assert!(profile.signature_required);
    }

    #[test]
    fn format_auth_url() {
        let template =
            "https://auth.example.com/hasJoined?username={username}&serverId={server_hash}&ip={ip}";
        let formatted = super::format_auth_url(
            template,
            "Player1",
            "hash123",
            &"127.0.0.1".parse().unwrap(),
        );
        assert_eq!(
            formatted,
            "https://auth.example.com/hasJoined?username=Player1&serverId=hash123&ip=127.0.0.1"
        );
    }

    #[test]
    fn auth_config_fallbacks_deserialization() {
        let json_str = r#"{
            "enabled": true,
            "url": "https://primary.auth/hasJoined?username={username}&serverId={server_hash}",
            "fallbacks": [
                "https://fallback1.auth/hasJoined?username={username}&serverId={server_hash}",
                "https://fallback2.auth/hasJoined?username={username}&serverId={server_hash}"
            ]
        }"#;
        let config: pumpkin_config::AuthenticationConfig =
            serde_json::from_str(json_str).expect("config should deserialize");
        assert_eq!(
            config.url.as_deref(),
            Some("https://primary.auth/hasJoined?username={username}&serverId={server_hash}")
        );
        assert_eq!(config.fallbacks.len(), 2);
        assert_eq!(
            config.fallbacks[0],
            "https://fallback1.auth/hasJoined?username={username}&serverId={server_hash}"
        );
    }

    #[test]
    fn auth_config_fallback_urls_alias_deserialization() {
        let json_str = r#"{
            "enabled": true,
            "fallback_urls": [
                "https://fallback1.auth/hasJoined?username={username}&serverId={server_hash}"
            ]
        }"#;
        let config: pumpkin_config::AuthenticationConfig =
            serde_json::from_str(json_str).expect("config with alias should deserialize");
        assert_eq!(config.fallbacks.len(), 1);
        assert_eq!(
            config.fallbacks[0],
            "https://fallback1.auth/hasJoined?username={username}&serverId={server_hash}"
        );
    }
}
