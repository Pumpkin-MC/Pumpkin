use pumpkin_util::ProfileAction;
use serde::{Deserialize, Serialize};

/// Configuration for server authentication.
///
/// Handles Mojang authentication, proxy restrictions, player profiles, and textures.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct AuthenticationConfig {
    /// Whether to use Mojang authentication.
    pub enabled: bool,
    /// Optional custom authentication URL.
    pub url: Option<String>,
    /// Connection timeout in milliseconds.
    pub connect_timeout: u32,
    /// Read timeout in milliseconds.
    pub read_timeout: u32,
    /// Whether to prevent connections via proxy.
    pub prevent_proxy_connections: bool,
    /// Optional auth URL used when preventing proxy connections.
    pub prevent_proxy_connection_auth_url: Option<String>,
    /// Public services URL (used by Drasl and Mojang).
    pub services_url: Option<String>,
    /// Player profile handling.
    pub player_profile: PlayerProfileConfig,
    /// Texture handling configuration.
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
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct PlayerProfileConfig {
    /// Allow players flagged by Mojang (e.g. banned, forced name change).
    pub allow_banned_players: bool,

    /// Depends on [`PlayerProfileConfig::allow_banned_players`].
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
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct TextureConfig {
    /// Whether to use player textures.
    pub enabled: bool,
    /// Allowed URL schemes for texture URLs.
    pub allowed_url_schemes: Vec<String>,
    /// Allowed URL domains for texture URLs.
    pub allowed_url_domains: Vec<String>,
    /// Specific texture types.
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
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct TextureTypes {
    /// Use player skins.
    pub skin: bool,
    /// Use player capes.
    pub cape: bool,
    /// Use player elytras.
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
