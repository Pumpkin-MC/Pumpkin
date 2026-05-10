use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ResourcePackConfig {
    pub java: JavaResourcePackConfig,
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
    pub enabled: bool,
    /// If true, players cannot join without accepting packs.
    pub force: bool,
    /// List of packs to be sent to the client.
    pub packs: Vec<BedrockPack>,
}

#[derive(Deserialize, Serialize)]
pub struct BedrockPack {
    pub uuid: Uuid,
    pub version: String,
    pub size: u64,
    pub download_url: String,
    #[serde(default)]
    pub content_key: String,
    #[serde(default)]
    pub sub_pack_name: String,
    #[serde(default)]
    pub content_id: String,
    #[serde(default)]
    pub has_scripts: bool,
    #[serde(default)]
    pub addon_pack: bool,
    #[serde(default)]
    pub rtx_enabled: bool,
}
