use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(162)]
pub struct CItemRegistry {
    // https://mojang.github.io/bedrock-protocol-docs/docs/ItemRegistryPacket.html
    pub items: Vec<ItemDefinition>,
}

pub struct ItemDefinition {
    pub name: String,
    pub id: i16,
    pub component_based: bool,
    pub item_version: VarInt,

    // Normally would be `Nbt`, but for simplicity elsewhere, this is preserialized (via `Nbt::write_bedrock`)
    pub component_data: Vec<u8>,
}

impl PacketWrite for ItemDefinition {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        self.name.write(writer)?;
        self.id.write(writer)?;
        self.component_based.write(writer)?;
        self.item_version.write(writer)?;
        writer.write_all(&self.component_data)?;

        Ok(())
    }
}
