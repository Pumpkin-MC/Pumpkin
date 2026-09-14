// Last verified for v2169

use crate::serial::PacketWrite;
use pumpkin_macros::packet;

#[derive(PacketWrite)]
pub struct PackInfoData {
    pub pack_id_version: PackIdVersion,
    pub pack_size: u64,
    pub content_key: String,
    pub subpack_name: String,
    pub content_identity: String,
    pub has_scripts: bool,
    pub is_addon_pack: bool,
    pub is_ray_tracing_capable: bool,
    pub cdn_url: String,
}

#[derive(PacketWrite)]
#[packet(6)]
pub struct CResourcePacksInfo {
    pub resource_pack_required: bool,
    pub has_addon_packs: bool,
    pub has_scripts: bool,
    pub force_disable_vibrant_visuals: bool,
    pub world_template_id_and_version: PackIdVersion,
    pub resource_packs: Vec<PackInfoData>,
}

#[derive(PacketWrite)]
pub struct PackIdVersion {
    pub pack_uuid: uuid::Uuid,
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
