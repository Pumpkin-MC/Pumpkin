use crate::serial::PacketWrite;
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(6)]
pub struct CResourcePacksInfo {
    resource_pack_required: bool,
    has_addon_packs: bool,
    has_scripts: bool,
    is_vibrant_visuals_force_disabled: bool,
    world_template_id: uuid::Uuid,
    world_template_version: String,
    resource_packs_size: u16,
    resource_packs: Vec<ResourcePack>,
}

#[derive(PacketWrite)]
pub struct ResourcePack {
    pack_id: uuid::Uuid,
    version: String,
    size: u64,
    content_key: String,
    subpack_name: String,
    content_identity: String,
    has_scripts: bool,
    is_addon_pack: bool,
    is_raytracing_capable: bool,
    cdn_url: String,
}

impl CResourcePacksInfo {
    pub fn new(
        resource_pack_required: bool,
        has_addon_packs: bool,
        has_scripts: bool,
        is_vibrant_visuals_force_disabled: bool,
        world_template_id: uuid::Uuid,
        world_template_version: String,
        resource_packs: Vec<ResourcePack>,
    ) -> Self {
        Self {
            resource_pack_required,
            has_addon_packs,
            has_scripts,
            is_vibrant_visuals_force_disabled,
            world_template_id,
            world_template_version,
            resource_packs_size: resource_packs.len() as u16,
            resource_packs,
        }
    }
}
