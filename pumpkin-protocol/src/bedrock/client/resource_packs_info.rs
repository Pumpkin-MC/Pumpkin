use std::io::{Error, Write};

use crate::serial::PacketWrite;
use pumpkin_macros::packet;

#[packet(6)]
/// <https://mojang.github.io/bedrock-protocol-docs/html/ResourcePacksInfoPacket.html>
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

impl PacketWrite for CResourcePacksInfo {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.resource_pack_required.write(writer)?;
        self.has_addon_packs.write(writer)?;
        self.has_scripts.write(writer)?;
        self.is_vibrant_visuals_force_disabled.write(writer)?;
        self.world_template_id.write(writer)?;
        self.world_template_version.write(writer)?;
        self.resource_packs_size.write(writer)?;
        self.resource_packs.write(writer)?;
        Ok(())
    }
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
    #[must_use] 
    pub const fn new(
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
