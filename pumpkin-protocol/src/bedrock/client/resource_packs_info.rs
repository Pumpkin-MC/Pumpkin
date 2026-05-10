use crate::serial::PacketWrite;
use pumpkin_macros::packet;
use std::io::{Error, Write};
pub struct ResourcePackEntry {
    pub uuid: String,
    pub version: String,
    pub size: u64,
    pub content_key: String,
    pub sub_pack_name: String,
    pub content_id: String,
    pub has_scripts: bool,
    pub addon_pack: bool,
    pub rtx_enabled: bool,
    pub download_url: String,
}

impl PacketWrite for ResourcePackEntry {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.uuid.write(writer)?;
        self.version.write(writer)?;
        // Bedrock uses Little Endian u64 for size
        writer.write_all(&self.size.to_le_bytes())?;
        self.content_key.write(writer)?;
        self.sub_pack_name.write(writer)?;
        self.content_id.write(writer)?;
        self.has_scripts.write(writer)?;
        Ok(())
    }
}

#[packet(6)]
pub struct CResourcePacksInfo {
    pub resource_pack_required: bool,
    pub has_addon_packs: bool,
    pub has_scripts: bool,
    pub is_vibrant_visuals_force_disabled: bool,
    pub world_template_id: uuid::Uuid,
    pub world_template_version: String,
    pub resource_packs: Vec<ResourcePackEntry>,
}

impl PacketWrite for CResourcePacksInfo {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.resource_pack_required.write(writer)?;
        self.has_addon_packs.write(writer)?;
        self.has_scripts.write(writer)?;
        self.is_vibrant_visuals_force_disabled.write(writer)?;

        self.world_template_id.write(writer)?;

        self.world_template_version.write(writer)?;

        let len = self.resource_packs.len() as u16;
        writer.write_all(&len.to_le_bytes())?;

        for entry in &self.resource_packs {
            entry.write(writer)?;
        }

        Ok(())
    }
}
