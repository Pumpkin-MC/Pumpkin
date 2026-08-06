use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Server resource pack configuration for Java and Bedrock clients.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ResourcePackConfig {
    /// Java Edition client resource pack configuration.
    pub java: JavaResourcePackConfig,
    /// Bedrock Edition client resource pack configuration.
    pub bedrock: BedrockResourcePackConfig,
}

/// Java-specific resource pack configuration (Single URL/Hash)
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct JavaResourcePackConfig {
    /// Whether the resource pack system is enabled.
    pub enabled: bool,
    /// The URL to the resource pack.
    pub url: String,
    /// The SHA1 hash (40 characters) of the resource pack.
    pub sha1: String,
    /// Custom prompt text component shown to players; leave blank for none.
    pub prompt_message: String,
    /// Whether players are forced to accept the resource pack.
    pub force: bool,
}

/// Bedrock-specific configuration (Supports multiple local/remote packs)
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BedrockResourcePackConfig {
    /// Whether Bedrock resource packs are enabled.
    pub enabled: bool,
    /// If true, players cannot join without accepting packs.
    pub force: bool,
    /// List of packs to be sent to the client.
    pub packs: Vec<BedrockPack>,
}

/// Bedrock resource pack manifest configuration entry.
#[derive(Deserialize, Serialize)]
pub struct BedrockPack {
    /// Unique identifier for the Bedrock pack.
    pub uuid: Uuid,
    /// Version string of the Bedrock pack.
    pub version: String,
    /// Size of the pack in bytes.
    pub size: u64,
    /// Download URL for remote Bedrock packs.
    pub download_url: String,
    /// Optional encryption content key for encrypted packs.
    #[serde(default)]
    pub content_key: String,
    /// Optional sub-pack name inside the archive.
    #[serde(default)]
    pub sub_pack_name: String,
    /// Optional content identifier.
    #[serde(default)]
    pub content_id: String,
    /// Whether the pack contains client scripts.
    #[serde(default)]
    pub has_scripts: bool,
    /// Whether the pack is marked as an addon pack.
    #[serde(default)]
    pub addon_pack: bool,
    /// Whether Ray Tracing / RTX features are enabled for the pack.
    #[serde(default)]
    pub rtx_enabled: bool,
}
