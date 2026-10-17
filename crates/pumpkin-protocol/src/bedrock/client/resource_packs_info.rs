// Last verified for v2169

use crate::serial::PacketWrite;
use pumpkin_macros::packet;

/// Information describing a resource pack available on the server.
#[derive(PacketWrite)]
pub struct PackInfoData {
    /// UUID and version string identifying the pack.
    pub pack_id_version: PackIdVersion,
    /// Size of the pack in bytes.
    pub pack_size: u64,
    /// Encryption key for marketplace or encrypted content.
    pub content_key: String,
    /// Specific subpack folder name, if applicable.
    pub subpack_name: String,
    /// Content identity identifier.
    pub content_identity: String,
    /// True if the pack contains JavaScript or game test scripts.
    pub has_scripts: bool,
    /// True if the pack includes custom add-on behaviors.
    pub is_addon_pack: bool,
    /// True if the pack includes PBR textures for ray tracing (RTX).
    pub is_ray_tracing_capable: bool,
    /// Optional CDN URL where the client can download the pack.
    pub cdn_url: String,
}

/// Sent by the server to list all resource and behavior packs required or available to download.
#[derive(PacketWrite)]
#[packet(6)]
pub struct CResourcePacksInfo {
    /// If true, the client must accept and download all packs to join.
    pub resource_pack_required: bool,
    /// Whether any packs in the list are add-on packs.
    pub has_addon_packs: bool,
    /// Whether any packs contain client-side scripting.
    pub has_scripts: bool,
    /// Whether vibrant visual shader enhancements should be disabled.
    pub force_disable_vibrant_visuals: bool,
    /// World template UUID and version, if joining a world template.
    pub world_template_id_and_version: PackIdVersion,
    /// Array of resource packs available on the server.
    pub resource_packs: Vec<PackInfoData>,
}

/// A resource pack UUID paired with its semantic version string.
#[derive(PacketWrite)]
pub struct PackIdVersion {
    /// Unique identifier for the pack.
    pub pack_uuid: uuid::Uuid,
    /// Semantic version string (e.g. `1.0.0`).
    pub pack_version: String,
}

impl PackIdVersion {
    #[must_use]
    pub const fn new(pack_uuid: uuid::Uuid, pack_version: String) -> Self {
        Self {
            pack_uuid,
            pack_version,
        }
    }
}
