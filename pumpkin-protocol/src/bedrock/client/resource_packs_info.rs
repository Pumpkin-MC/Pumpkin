use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(PacketWrite)]
#[packet(6)]
pub struct CResourcePacksInfo {
    resource_pack_required: bool,
    has_addon_packs: bool,
    has_scripts: bool,
    is_vibrant_visuals_force_disabled: bool,
    world_template_id: uuid::Uuid,
    world_template_version: String,
    resource_packs_size: u16, // TODO: Add more
}

impl CResourcePacksInfo {
    #[must_use]
    #[expect(clippy::fn_params_excessive_bools)]
    pub fn new(
        resource_pack_required: bool,
        has_addon_packs: bool,
        has_scripts: bool,
        is_vibrant_visuals_force_disabled: bool,
        world_template_id: uuid::Uuid,
        world_template_version: String,
    ) -> Self {
        Self {
            resource_pack_required,
            has_addon_packs,
            has_scripts,
            is_vibrant_visuals_force_disabled,
            world_template_id,
            world_template_version,
            // TODO
            resource_packs_size: 0,
        }
    }
}
