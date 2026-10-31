use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Server resource pack configuration for Java and Bedrock clients.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct ResourcePackConfig {
    /// Java Edition client resource pack configuration.
    #[toml(default = Self::default().java)]
    pub java: JavaResourcePackConfig,
    /// Bedrock Edition client resource pack configuration.
    #[toml(default = Self::default().bedrock)]
    pub bedrock: BedrockResourcePackConfig,
}

/// Java-specific resource pack configuration (Single URL/Hash)
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct JavaResourcePackConfig {
    /// Whether the resource pack system is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// The URL to the resource pack.
    #[toml(default = Self::default().url)]
    pub url: String,
    /// The SHA1 hash (40 characters) of the resource pack.
    #[toml(default = Self::default().sha1)]
    pub sha1: String,
    /// Custom prompt text component shown to players; leave blank for none.
    #[toml(default = Self::default().prompt_message)]
    pub prompt_message: String,
    /// Whether players are forced to accept the resource pack.
    #[toml(default = Self::default().force)]
    pub force: bool,
}

/// Bedrock-specific configuration (Supports multiple local/remote packs)
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BedrockResourcePackConfig {
    /// Whether Bedrock resource packs are enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// If true, players cannot join without accepting packs.
    #[toml(default = Self::default().force)]
    pub force: bool,
    /// List of packs to be sent to the client.
    #[toml(default = Self::default().packs)]
    pub packs: Vec<BedrockPack>,
}

/// Bedrock resource pack manifest configuration entry.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BedrockPack {
    /// Unique identifier for the Bedrock pack.
    #[toml(with = toml_spanner::helper::parse_string)]
    pub uuid: Uuid,
    /// Version string of the Bedrock pack.
    pub version: String,
    /// Size of the pack in bytes.
    pub size: u64,
    /// Download URL for remote Bedrock packs.
    pub download_url: String,
    /// Optional filesystem path to the pack file on the server.
    #[serde(default)]
    #[toml(default)]
    pub path: Option<std::path::PathBuf>,
    /// Optional encryption content key for encrypted packs.
    #[serde(default)]
    #[toml(default)]
    pub content_key: String,
    /// Optional sub-pack name inside the archive.
    #[serde(default)]
    #[toml(default)]
    pub sub_pack_name: String,
    /// Optional content identifier.
    #[serde(default)]
    #[toml(default)]
    pub content_id: String,
    /// Whether the pack contains client scripts.
    #[serde(default)]
    #[toml(default)]
    pub has_scripts: bool,
    /// Whether the pack is marked as an addon pack.
    #[serde(default)]
    #[toml(default)]
    pub addon_pack: bool,
    /// Whether Ray Tracing / RTX features are enabled for the pack.
    #[serde(default)]
    #[toml(default)]
    pub rtx_enabled: bool,
}
