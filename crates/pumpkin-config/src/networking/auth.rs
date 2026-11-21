use pumpkin_util::ProfileAction;
use serde::{Deserialize, Serialize};

/// Configuration for server authentication.
///
/// Handles Mojang authentication, proxy restrictions, player profiles, and textures.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct AuthenticationConfig {
    /// Whether to use Mojang authentication.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Optional custom authentication URL.
    #[toml(default = Self::default().url)]
    pub url: Option<String>,
    /// Fallback authentication server URLs to use if the primary/official server is down.
    #[serde(alias = "fallback_urls")]
    #[toml(default = Self::default().fallbacks, alias = "fallback_urls")]
    pub fallbacks: Vec<String>,
    /// Optional custom profile lookup by username URL (template parameter `{username}`).
    #[toml(default = Self::default().profile_by_name_url)]
    pub profile_by_name_url: Option<String>,
    /// Optional fallback profile lookup by username URLs.
    #[serde(alias = "profile_by_name_fallback_urls")]
    #[toml(default = Self::default().profile_by_name_fallbacks, alias = "profile_by_name_fallback_urls")]
    pub profile_by_name_fallbacks: Vec<String>,
    /// Optional custom profile lookup by UUID URL (template parameter `{uuid}`).
    #[toml(default = Self::default().profile_by_uuid_url)]
    pub profile_by_uuid_url: Option<String>,
    /// Optional fallback profile lookup by UUID URLs.
    #[serde(alias = "profile_by_uuid_fallback_urls")]
    #[toml(default = Self::default().profile_by_uuid_fallbacks, alias = "profile_by_uuid_fallback_urls")]
    pub profile_by_uuid_fallbacks: Vec<String>,
    /// Connection timeout in milliseconds.
    #[toml(default = Self::default().connect_timeout)]
    pub connect_timeout: u32,
    /// Read timeout in milliseconds.
    #[toml(default = Self::default().read_timeout)]
    pub read_timeout: u32,
    /// Whether to prevent connections via proxy.
    #[toml(default = Self::default().prevent_proxy_connections)]
    pub prevent_proxy_connections: bool,
    /// Optional auth URL used when preventing proxy connections.
    #[toml(default = Self::default().prevent_proxy_connection_auth_url)]
    pub prevent_proxy_connection_auth_url: Option<String>,
    /// Public services URL (used by Drasl and Mojang).
    #[toml(default = Self::default().services_url)]
    pub services_url: Option<String>,
    /// Player profile handling.
    #[toml(default = Self::default().player_profile)]
    pub player_profile: PlayerProfileConfig,
    /// Texture handling configuration.
    #[toml(default = Self::default().textures)]
    pub textures: TextureConfig,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prevent_proxy_connections: false,
            player_profile: PlayerProfileConfig::default(),
            textures: TextureConfig::default(),
            url: None,
            fallbacks: Vec::new(),
            profile_by_name_url: None,
            profile_by_name_fallbacks: Vec::new(),
            profile_by_uuid_url: None,
            profile_by_uuid_fallbacks: Vec::new(),
            prevent_proxy_connection_auth_url: None,
            services_url: None,
            connect_timeout: 5000,
            read_timeout: 5000,
        }
    }
}

/// Configuration for player profile handling.
///
/// Controls whether banned players are allowed and which profile actions are permitted.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct PlayerProfileConfig {
    /// Allow players flagged by Mojang (e.g. banned, forced name change).
    #[toml(default = Self::default().allow_banned_players)]
    pub allow_banned_players: bool,
    /// Depends on [`PlayerProfileConfig::allow_banned_players`].
    #[toml(default = Self::default().allowed_actions, with = crate::toml_adapters::serde_vec)]
    pub allowed_actions: Vec<ProfileAction>,
}

impl Default for PlayerProfileConfig {
    fn default() -> Self {
        Self {
            allow_banned_players: false,
            allowed_actions: vec![
                ProfileAction::ForcedNameChange,
                ProfileAction::UsingBannedSkin,
            ],
        }
    }
}

/// Configuration for player textures.
///
/// Controls whether textures are applied, allowed URL schemes/domains, and texture types.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct TextureConfig {
    /// Whether to use player textures.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Allowed URL schemes for texture URLs.
    #[toml(default = Self::default().allowed_url_schemes)]
    pub allowed_url_schemes: Vec<String>,
    /// Allowed URL domains for texture URLs.
    #[toml(default = Self::default().allowed_url_domains)]
    pub allowed_url_domains: Vec<String>,
    /// Specific texture types.
    #[toml(default = Self::default().types)]
    pub types: TextureTypes,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_url_schemes: vec!["http".into(), "https".into()],
            allowed_url_domains: vec![".minecraft.net".into(), ".mojang.com".into()],
            types: TextureTypes::default(),
        }
    }
}

/// Specifies which player texture types are supported.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct TextureTypes {
    /// Use player skins.
    #[toml(default = Self::default().skin)]
    pub skin: bool,
    /// Use player capes.
    #[toml(default = Self::default().cape)]
    pub cape: bool,
    /// Use player elytras.
    #[toml(default = Self::default().elytra)]
    pub elytra: bool,
}

impl Default for TextureTypes {
    fn default() -> Self {
        Self {
            skin: true,
            cape: true,
            elytra: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthenticationConfig;

    #[test]
    fn auth_config_fallbacks_toml_deserialization() {
        let toml_str = r#"
enabled = true
url = "https://primary.auth/hasJoined?username={username}&serverId={server_hash}"
fallbacks = [
    "https://fallback1.auth/hasJoined?username={username}&serverId={server_hash}",
    "https://fallback2.auth/hasJoined?username={username}&serverId={server_hash}"
]
"#;
        let config: AuthenticationConfig =
            toml::from_str(toml_str).expect("config should deserialize");
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
    fn auth_config_fallback_urls_alias_toml_deserialization() {
        let toml_str = r#"
enabled = true
fallback_urls = [
    "https://fallback1.auth/hasJoined?username={username}&serverId={server_hash}"
]
"#;
        let config: AuthenticationConfig =
            toml::from_str(toml_str).expect("config with alias should deserialize");
        assert_eq!(config.fallbacks.len(), 1);
        assert_eq!(
            config.fallbacks[0],
            "https://fallback1.auth/hasJoined?username={username}&serverId={server_hash}"
        );
    }

    #[test]
    fn auth_config_profile_urls_toml_deserialization() {
        let toml_str = r#"
enabled = true
profile_by_name_url = "https://custom.auth/users/profiles/minecraft/{username}"
profile_by_name_fallback_urls = [
    "https://fallback.auth/users/profiles/minecraft/{username}"
]
profile_by_uuid_url = "https://custom.auth/session/minecraft/profile/{uuid}?unsigned=false"
profile_by_uuid_fallback_urls = [
    "https://fallback.auth/session/minecraft/profile/{uuid}?unsigned=false"
]
"#;
        let config: AuthenticationConfig =
            toml::from_str(toml_str).expect("profile config should deserialize");
        assert_eq!(
            config.profile_by_name_url.as_deref(),
            Some("https://custom.auth/users/profiles/minecraft/{username}")
        );
        assert_eq!(config.profile_by_name_fallbacks.len(), 1);
        assert_eq!(
            config.profile_by_uuid_url.as_deref(),
            Some("https://custom.auth/session/minecraft/profile/{uuid}?unsigned=false")
        );
        assert_eq!(config.profile_by_uuid_fallbacks.len(), 1);
    }
}
