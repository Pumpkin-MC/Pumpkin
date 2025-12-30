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

pub const MOJANG_BEDROCK_PUBLIC_KEY_BASE64: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAECRXueJeTDqNRRgJi/vlRufByu/2G0i2Ebt6YMar5QX/R0DIIyrJMcUpruK4QveTfJSTp3Shlq4Gk34cD/4GUWwkv0DVuzeuB+tXija7HBxii03NHDbPAD0AKnLr2wdAp";
const MOJANG_AUTHENTICATION_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}";
const MOJANG_PREVENT_PROXY_AUTHENTICATION_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}";
const MOJANG_SERVICES_URL: &str = "https://api.minecraftservices.com/";

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
    Ok(profile)
}

pub fn validate_textures(property: &Property, config: &TextureConfig) -> Result<(), TextureError> {
    let from64 = general_purpose::STANDARD
        .decode(&property.value)
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
    let scheme = url.scheme().ok_or(TextureError::InvalidURL)?;
    
    // Exact match for scheme (not suffix match)
    if !config
        .allowed_url_schemes
        .iter()
        .any(|allowed_scheme| scheme.as_str() == allowed_scheme)
    {
        return Err(TextureError::DisallowedUrlScheme(scheme.to_string()));
    }
    
    let authority = url.authority().ok_or(TextureError::InvalidURL)?;
    let domain = authority.host();
    
    // Check for suspicious patterns in domain
    // These patterns could indicate path traversal or URL encoding attacks
    if domain.contains("..") || domain.contains('%') {
        return Err(TextureError::DisallowedUrlDomain(domain.to_string()));
    }
    
    // Exact match for domain (not suffix match)
    // This prevents attacks like "evil-textures.minecraft.net" matching "minecraft.net"
    if !config
        .allowed_url_domains
        .iter()
        .any(|allowed_domain| domain == *allowed_domain)
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
    use proptest::prelude::*;

    /// Helper to create a TextureConfig with specific allowed domains and schemes
    fn config_with_domains(domains: Vec<String>, schemes: Vec<String>) -> TextureConfig {
        TextureConfig {
            enabled: true,
            allowed_url_schemes: schemes,
            allowed_url_domains: domains,
            types: Default::default(),
        }
    }

    /// Property test: For any texture URL, the domain SHALL exactly match an allowed domain (not suffix match).
    /// This prevents attacks like "evil-textures.minecraft.net" matching "minecraft.net"
    /// **Feature: security-hardening, Property 7: Texture URL Exact Match**
    /// **Validates: Requirements 6.1, 6.2**
    #[test]
    fn test_property_exact_domain_match_rejects_suffix() {
        // Test that suffix matching is NOT allowed
        let config = config_with_domains(
            vec!["minecraft.net".to_string()],
            vec!["https".to_string()],
        );

        // These should all be REJECTED because they only suffix-match, not exact match
        let malicious_urls = [
            "https://evil-minecraft.net/texture.png",
            "https://fake.minecraft.net/texture.png",
            "https://textures.minecraft.net/texture.png",
            "https://attacker-minecraft.net/texture.png",
            "https://xminecraft.net/texture.png",
        ];

        for url_str in malicious_urls {
            let url: Uri = url_str.parse().unwrap();
            let result = is_texture_url_valid(&url, &config);
            assert!(
                result.is_err(),
                "URL {} should be rejected (suffix match not allowed), but was accepted",
                url_str
            );
        }

        // This should be ACCEPTED because it's an exact match
        let valid_url: Uri = "https://minecraft.net/texture.png".parse().unwrap();
        let result = is_texture_url_valid(&valid_url, &config);
        assert!(
            result.is_ok(),
            "URL https://minecraft.net/texture.png should be accepted (exact match)"
        );
    }

    /// Property test: URLs with suspicious patterns (.., %, encoded chars) SHALL be rejected
    /// **Feature: security-hardening, Property 7: Texture URL Exact Match**
    /// **Validates: Requirements 6.1, 6.2**
    #[test]
    fn test_property_suspicious_patterns_rejected() {
        let config = config_with_domains(
            vec!["minecraft.net".to_string(), "textures..minecraft.net".to_string()],
            vec!["https".to_string()],
        );

        // URLs with suspicious patterns should be rejected
        let suspicious_urls = [
            "https://textures..minecraft.net/texture.png", // double dots
            "https://minecraft%2enet/texture.png",         // percent encoding
        ];

        for url_str in suspicious_urls {
            if let Ok(url) = url_str.parse::<Uri>() {
                let result = is_texture_url_valid(&url, &config);
                assert!(
                    result.is_err(),
                    "URL {} with suspicious pattern should be rejected",
                    url_str
                );
            }
            // If URL parsing fails, that's also acceptable (invalid URL)
        }
    }

    /// Property test: Exact scheme matching (not suffix match)
    /// **Feature: security-hardening, Property 7: Texture URL Exact Match**
    /// **Validates: Requirements 6.1, 6.2**
    #[test]
    fn test_property_exact_scheme_match() {
        let config = config_with_domains(
            vec!["minecraft.net".to_string()],
            vec!["https".to_string()],
        );

        // HTTP should be rejected when only HTTPS is allowed
        let http_url: Uri = "http://minecraft.net/texture.png".parse().unwrap();
        let result = is_texture_url_valid(&http_url, &config);
        assert!(
            result.is_err(),
            "HTTP URL should be rejected when only HTTPS is allowed"
        );

        // HTTPS should be accepted
        let https_url: Uri = "https://minecraft.net/texture.png".parse().unwrap();
        let result = is_texture_url_valid(&https_url, &config);
        assert!(
            result.is_ok(),
            "HTTPS URL should be accepted when HTTPS is allowed"
        );
    }

    proptest! {
        /// Property test using proptest: For any domain that is NOT in the allowed list,
        /// the URL SHALL be rejected
        /// **Feature: security-hardening, Property 7: Texture URL Exact Match**
        /// **Validates: Requirements 6.1, 6.2**
        #[test]
        fn test_proptest_non_whitelisted_domain_rejected(
            subdomain in "[a-z]{1,10}",
            allowed_domain in "[a-z]{3,10}\\.[a-z]{2,4}",
        ) {
            // Create a domain that is a subdomain of the allowed domain
            let malicious_domain = format!("{}.{}", subdomain, allowed_domain);
            
            let config = config_with_domains(
                vec![allowed_domain.clone()],
                vec!["https".to_string()],
            );

            let url_str = format!("https://{}/texture.png", malicious_domain);
            if let Ok(url) = url_str.parse::<Uri>() {
                let result = is_texture_url_valid(&url, &config);
                // Subdomain should be rejected (not exact match)
                prop_assert!(
                    result.is_err(),
                    "Subdomain {} of allowed domain {} should be rejected",
                    malicious_domain,
                    allowed_domain
                );
            }
        }

        /// Property test: Exact domain match should be accepted
        /// **Feature: security-hardening, Property 7: Texture URL Exact Match**
        /// **Validates: Requirements 6.1, 6.2**
        #[test]
        fn test_proptest_exact_domain_accepted(
            domain in "[a-z]{3,10}\\.[a-z]{2,4}",
        ) {
            let config = config_with_domains(
                vec![domain.clone()],
                vec!["https".to_string()],
            );

            let url_str = format!("https://{}/texture.png", domain);
            if let Ok(url) = url_str.parse::<Uri>() {
                let result = is_texture_url_valid(&url, &config);
                prop_assert!(
                    result.is_ok(),
                    "Exact domain match {} should be accepted",
                    domain
                );
            }
        }

        /// Property test: URLs with percent encoding in domain should be rejected
        /// **Feature: security-hardening, Property 7: Texture URL Exact Match**
        /// **Validates: Requirements 6.1, 6.2**
        #[test]
        fn test_proptest_percent_encoding_rejected(
            prefix in "[a-z]{1,5}",
            suffix in "[a-z]{1,5}",
        ) {
            let config = config_with_domains(
                vec!["minecraft.net".to_string()],
                vec!["https".to_string()],
            );

            // Create a domain with percent encoding
            let malicious_domain = format!("{}%2e{}", prefix, suffix);
            let url_str = format!("https://{}.net/texture.png", malicious_domain);
            
            if let Ok(url) = url_str.parse::<Uri>() {
                let result = is_texture_url_valid(&url, &config);
                // Should be rejected due to percent encoding
                prop_assert!(
                    result.is_err(),
                    "Domain with percent encoding {} should be rejected",
                    malicious_domain
                );
            }
        }
    }
}
