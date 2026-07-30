use std::{collections::HashMap, net::IpAddr};

use base64::{Engine, engine::general_purpose};
use pumpkin_config::{
    AuthenticationConfig, YggdrasilServiceConfig, networking::auth::TextureConfig,
};
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

// Authlib-Injector Yggdrasil metadata

/// Response from the Yggdrasil `GET /` metadata endpoint.
#[derive(Deserialize, Clone, Debug)]
struct YggdrasilMeta {
    #[serde(rename = "skinDomains")]
    skin_domains: Option<Vec<String>>,
    #[serde(rename = "signaturePublickey")]
    signature_publickey: Option<String>,
}

/// Derive the Yggdrasil API root URL from a `/hasJoined` URL.
fn derive_api_root(has_joined_url: &str) -> Option<String> {
    let path = has_joined_url.split('?').next()?;

    if let Some(base) = path.strip_suffix("/sessionserver/session/minecraft/hasJoined") {
        return Some(base.to_string());
    }
    if let Some(base) = path.strip_suffix("/session/minecraft/hasJoined") {
        return Some(base.to_string());
    }
    // Non-standard path: strip the last segment as a best-effort fallback.
    path.rsplit_once('/').map(|(base, _)| base.to_string())
}

/// Fetch the `GET /` metadata endpoint of a Yggdrasil service.
///
/// Returns `skinDomains` and `signaturePublickey` on success, `None` if
/// the endpoint is unavailable (404, timeout, …).
fn fetch_yggdrasil_meta(service: &YggdrasilServiceConfig) -> Option<YggdrasilMeta> {
    let root = derive_api_root(&service.url)?;
    let url = format!("{root}/");

    let response = ureq::get(&url).call().ok()?;
    if response.status() != StatusCode::OK {
        return None;
    }
    response.into_body().read_json::<YggdrasilMeta>().ok()
}

/// Domain matching per Authlib-Injector spec
///
/// - Rules starting with `.` match any domain whose suffix equals the rule
///   (e.g. `.example.com` matches `cdn.example.com`, **not** `example.com`).
/// - Rules without a leading `.` require an exact match.
fn is_domain_allowed(domain: &str, rule: &str) -> bool {
    if rule.starts_with('.') {
        domain.ends_with(rule)
    } else {
        domain == rule
    }
}

const MOJANG_AUTHENTICATION_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}";
const MOJANG_PREVENT_PROXY_AUTHENTICATION_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}";
const MOJANG_SERVICES_URL: &str = "https://api.minecraftservices.com/";
const MOJANG_PROFILE_BY_NAME_URL: &str =
    "https://api.mojang.com/users/profiles/minecraft/{username}";

/// Sends a GET request to Mojang's authentication servers to verify a client's Minecraft account.
///
/// **Purpose:**
///
/// This function is used to ensure that a client connecting to the server has a valid, premium Minecraft account. It's a crucial step in preventing unauthorized access and maintaining server security.
///
/// **How it Works:**
///
/// 1. A client with a premium account sends a login request to the Mojang session server.
/// 2. Mojang's servers verify the client's credentials and add the player to the their Servers
/// 3. Now our server will send a Request to the Session servers and check if the Player has joined the Session Server .
///
/// See <https://pumpkinmc.org/developer/networking/authentication>
pub fn authenticate(
    username: &str,
    server_hash: &str,
    ip: &IpAddr,
    auth_config: &AuthenticationConfig,
) -> Result<GameProfile, AuthError> {
    let address = if auth_config.prevent_proxy_connections {
        let auth_url = auth_config
            .prevent_proxy_connection_auth_url
            .as_deref()
            .unwrap_or(MOJANG_PREVENT_PROXY_AUTHENTICATION_URL);

        auth_url
            .replace("{username}", username)
            .replace("{server_hash}", server_hash)
            .replace("{ip}", &ip.to_string())
    } else {
        let auth_url = auth_config
            .url
            .as_deref()
            .unwrap_or(MOJANG_AUTHENTICATION_URL);

        auth_url
            .replace("{username}", username)
            .replace("{server_hash}", server_hash)
    };

    let mut response = ureq::get(address)
        .call()
        .map_err(|_| AuthError::FailedResponse)?;
    match response.status() {
        StatusCode::OK => {}
        StatusCode::NO_CONTENT => Err(AuthError::UnverifiedUsername)?,
        other => Err(AuthError::UnknownStatusCode(other))?,
    }
    let profile: GameProfile = response
        .body_mut()
        .read_json()
        .map_err(|_| AuthError::FailedParse)?;

    // Validate textures against the global config (single-auth path).
    for property in profile.properties.load().iter() {
        validate_textures(property, &auth_config.textures).map_err(AuthError::TextureError)?;
    }

    Ok(profile)
}

/// Authenticate a player against an individual Yggdrasil service.
fn authenticate_service(
    name: &str,
    username: &str,
    server_hash: &str,
    ip: &IpAddr,
    service: &YggdrasilServiceConfig,
    global_config: &AuthenticationConfig,
) -> Result<GameProfile, AuthError> {
    let address = service
        .url
        .replace("{username}", username)
        .replace("{server_hash}", server_hash)
        .replace("{ip}", &ip.to_string());

    let connect_timeout = if service.connect_timeout > 0 {
        service.connect_timeout
    } else {
        global_config.connect_timeout
    };
    let read_timeout = if service.read_timeout > 0 {
        service.read_timeout
    } else {
        global_config.read_timeout
    };

    let agent_config = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_millis(
            connect_timeout as u64,
        )))
        .timeout_global(Some(std::time::Duration::from_millis(read_timeout as u64)))
        .build();
    let agent = agent_config.new_agent();

    tracing::info!("[{name}] /hasJoined username={username} server_hash={server_hash}");

    let response = agent
        .get(&address)
        .call()
        .map_err(|_| AuthError::FailedResponse)?;

    match response.status() {
        StatusCode::OK => {
            tracing::info!("[{name}] /hasJoined → 200 OK");
        }
        StatusCode::NO_CONTENT => {
            tracing::info!("[{name}] /hasJoined → 204 (user not found)");
            Err(AuthError::UnverifiedUsername)?;
        }
        other => Err(AuthError::UnknownStatusCode(other))?,
    }

    // Log the canonical API location reported by Authlib-Injector services.
    // This helps verify that the configured URL matches what the service expects.
    if let Some(loc) = response.headers().get("x-authlib-injector-api-location")
        && let Ok(loc_str) = loc.to_str()
    {
        tracing::info!("[{name}] x-authlib-injector-api-location: {loc_str}");
    }

    let profile: GameProfile = response
        .into_body()
        .read_json()
        .map_err(|_| AuthError::FailedParse)?;

    // Validate textures against the per-service config if present,
    // otherwise fall back to the global texture config.
    let base_config = service.textures.as_ref().unwrap_or(&global_config.textures);

    // Query the Yggdrasil GET / metadata endpoint to auto-discover
    // skinDomains (§5.1).  This removes the need to manually list every
    // third-party texture domain in the config.
    let merged_config;
    let texture_config = if let Some(meta) = fetch_yggdrasil_meta(service) {
        if let Some(ref key) = meta.signature_publickey {
            tracing::debug!("[{name}] signaturePublickey: {} bytes", key.len());
        }
        if let Some(domains) = meta.skin_domains.as_ref()
            && !domains.is_empty()
        {
            let mut merged = base_config.clone();
            merged.allowed_url_domains.extend(domains.iter().cloned());
            tracing::debug!(
                "[{name}] merged {} discovered skinDomains",
                domains.len()
            );
            merged_config = merged;
            &merged_config
        } else {
            base_config
        }
    } else {
        base_config
    };

    for property in profile.properties.load().iter() {
        validate_textures(property, texture_config).map_err(AuthError::TextureError)?;
    }

    tracing::debug!("{username} authenticated by '{name}'");
    Ok(profile)
}

/// Find the service that has `username` in its `player_names` list.
/// Returns `None` if no service explicitly claims this player.
fn resolve_player_service<'a>(
    username: &str,
    entries: &'a HashMap<String, YggdrasilServiceConfig>,
    service_order: &'a [String],
) -> Option<(&'a str, &'a YggdrasilServiceConfig)> {
    for name in service_order {
        if let Some(svc) = entries.get(name)
            && svc
                .player_names
                .iter()
                .any(|n| n.eq_ignore_ascii_case(username))
        {
            return Some((name.as_str(), svc));
        }
    }
    None
}

/// Try every registered Yggdrasil service in order; first success wins.
///
/// Routing rules:
/// 1. If a service's `player_names` contains `username`, only that service
///    is tried — no fallback.
/// 2. Otherwise every service without `player_names` restrictions is tried
///    in `services` list order.
/// 3. When `auth_config.services` is empty, the legacy single-URL path is
///    used for backward compatibility.
pub fn authenticate_chain(
    username: &str,
    server_hash: &str,
    ip: &IpAddr,
    auth_config: &AuthenticationConfig,
) -> Result<GameProfile, AuthError> {
    if auth_config.services.is_empty() {
        return authenticate(username, server_hash, ip, auth_config);
    }

    // ── 1. Explicit routing ──
    if let Some((name, service)) = resolve_player_service(
        username,
        &auth_config.service_entries,
        &auth_config.services,
    ) {
        tracing::debug!("{username} routed to '{name}' (player_names match)");
        return authenticate_service(name, username, server_hash, ip, service, auth_config);
    }

    // ── 2. Priority chain (skip services that have player_names set) ──
    let mut attempted = 0usize;
    for name in &auth_config.services {
        let Some(service) = auth_config.service_entries.get(name) else {
            tracing::warn!(
                "Service '{name}' appears in `services` list but has no matching config table; skipping"
            );
            continue;
        };
        // Skip services with restricted player_names — those are exclusive.
        if !service.player_names.is_empty() {
            continue;
        }
        attempted += 1;
        match authenticate_service(name, username, server_hash, ip, service, auth_config) {
            Ok(profile) => return Ok(profile),
            Err(e) => {
                // Use warn level so individual service failures are visible
                // in normal log output — critical for diagnosing multi-auth issues.
                tracing::warn!("{username} not authenticated by '{name}': {e}");
            }
        }
    }

    tracing::warn!("{username}: tried {attempted} auth service(s), none succeeded");

    Err(AuthError::UnverifiedUsername)
}

pub fn validate_textures(property: &Property, config: &TextureConfig) -> Result<(), TextureError> {
    // Only validate the "textures" property; other properties
    // (e.g. launcher metadata injected by Authlib-Injector) are
    // not base64-encoded texture payloads.
    if property.name.as_ref() != "textures" {
        return Ok(());
    }

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
    let scheme = url.scheme().unwrap();
    if !config
        .allowed_url_schemes
        .iter()
        .any(|allowed_scheme| scheme.as_str().ends_with(allowed_scheme))
    {
        return Err(TextureError::DisallowedUrlScheme(scheme.to_string()));
    }
    let domain = url.authority().unwrap();
    if !config
        .allowed_url_domains
        .iter()
        .any(|rule| is_domain_allowed(domain.as_str(), rule))
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

    let mut response = ureq::get(url)
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

/// A single entry in the Yggdrasil `/api/profiles/minecraft` response.
#[derive(Deserialize, Clone, Debug)]
struct YggdrasilProfileEntry {
    id: String,
    name: String,
}

/// Derive a profile-lookup URL from a `/hasJoined` URL.
///
/// Strips the query string (everything after `?`) and the known hasJoined
/// path suffix, then appends `/api/profiles/minecraft`.
fn derive_profile_lookup_url(has_joined_url: &str) -> Option<String> {
    // Remove query string
    let path = has_joined_url.split('?').next()?;

    // Try Authlib-Injector style: /sessionserver/session/minecraft/hasJoined
    if let Some(base) = path.strip_suffix("/sessionserver/session/minecraft/hasJoined") {
        return Some(format!("{base}/api/profiles/minecraft"));
    }

    // Try Mojang style: /session/minecraft/hasJoined
    if let Some(base) = path.strip_suffix("/session/minecraft/hasJoined") {
        return Some(format!("{base}/api/profiles/minecraft"));
    }

    None
}

/// Look up a player by name in a single Yggdrasil service.
///
/// Uses the POST `/api/profiles/minecraft` endpoint defined by the
/// Authlib-Injector / Yggdrasil spec.
fn lookup_profile_in_service(
    username: &str,
    service: &YggdrasilServiceConfig,
) -> Result<Option<(Uuid, String)>, AuthError> {
    let lookup_url = service
        .profile_lookup_url
        .clone()
        .or_else(|| derive_profile_lookup_url(&service.url));

    let Some(ref lookup_url) = lookup_url else {
        return Ok(None);
    };

    // send_json automatically sets Content-Type: application/json
    let response = ureq::post(lookup_url)
        .send_json([username])
        .map_err(|_| AuthError::FailedResponse)?;

    match response.status() {
        StatusCode::OK => {}
        StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => return Ok(None),
        other => {
            tracing::debug!("Profile lookup for '{username}' returned unexpected status: {other}");
            return Ok(None);
        }
    }

    let entries: Vec<YggdrasilProfileEntry> = response
        .into_body()
        .read_json()
        .map_err(|_| AuthError::FailedParse)?;

    if let Some(entry) = entries.first() {
        let parsed_uuid = Uuid::parse_str(&entry.id).map_err(|_| AuthError::FailedParse)?;
        tracing::debug!(
            "Profile lookup for '{username}': found {parsed_uuid} ({})",
            entry.name
        );
        Ok(Some((parsed_uuid, entry.name.clone())))
    } else {
        Ok(None)
    }
}

/// Look up a player by name across all configured Yggdrasil services
/// (plus the legacy Mojang endpoint).
///
/// Tries services in order; first match wins.  Falls back to the legacy
/// Mojang profile API when no service is configured.
pub fn lookup_profile_by_name_chain(
    name: &str,
    auth_config: &AuthenticationConfig,
) -> Result<Option<(Uuid, String)>, AuthError> {
    if auth_config.services.is_empty() {
        return lookup_profile_by_name_legacy(name);
    }

    // Try Yggdrasil services
    for service_name in &auth_config.services {
        let Some(service) = auth_config.service_entries.get(service_name) else {
            continue;
        };
        // Skip services with restricted player_names — those are exclusive login routes.
        if !service.player_names.is_empty() {
            continue;
        }
        match lookup_profile_in_service(name, service) {
            Ok(Some(profile)) => return Ok(Some(profile)),
            Ok(None) => {} // not found, try next
            Err(e) => {
                tracing::debug!("Profile lookup for '{name}' failed on '{service_name}': {e}");
            }
        }
    }

    // Fall back to legacy Mojang endpoint
    lookup_profile_by_name_legacy(name)
}

/// Legacy Mojang-only profile lookup (GET api.mojang.com).
fn lookup_profile_by_name_legacy(name: &str) -> Result<Option<(Uuid, String)>, AuthError> {
    let url = MOJANG_PROFILE_BY_NAME_URL.replace("{username}", name);

    let mut response = ureq::get(url)
        .call()
        .map_err(|_| AuthError::FailedResponse)?;

    match response.status() {
        StatusCode::OK => {}
        StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => return Ok(None),
        other => Err(AuthError::UnknownStatusCode(other))?,
    }

    let profile: MojangProfileByNameResponse = response
        .body_mut()
        .read_json()
        .map_err(|_| AuthError::FailedParse)?;

    let parsed_uuid = Uuid::parse_str(&profile.id).map_err(|_| AuthError::FailedParse)?;
    Ok(Some((parsed_uuid, profile.name)))
}

pub fn lookup_profile_by_name(
    name: &str,
    auth_config: &AuthenticationConfig,
) -> Result<Option<(Uuid, String)>, AuthError> {
    lookup_profile_by_name_chain(name, auth_config)
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
    use super::*;
    use pumpkin_config::{AuthenticationConfig, YggdrasilServiceConfig};

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

    // ── Multi-auth chain tests ──

    #[test]
    fn chain_skips_failed_services_and_returns_error_when_all_fail() {
        use std::collections::HashMap;

        let mut entries = HashMap::new();
        entries.insert(
            "BadSvc1".to_string(),
            YggdrasilServiceConfig {
                url: "https://127.0.0.1:1/bad1?username={username}&serverId={server_hash}".into(),
                ..Default::default()
            },
        );

        let config = AuthenticationConfig {
            services: vec!["BadSvc1".into()],
            service_entries: entries,
            ..Default::default()
        };

        let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
        let result = authenticate_chain("TestPlayer", "fake-hash", &ip, &config);
        // Should fail (can't connect to a non-existent service) without panicking.
        assert!(result.is_err());
    }
}
