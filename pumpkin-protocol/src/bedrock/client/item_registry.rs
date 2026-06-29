use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(162)]
pub struct CItemRegistry {
    // https://mojang.github.io/bedrock-protocol-docs/docs/ItemRegistryPacket.html
    pub items: Vec<ItemDefinition>,
}

#[derive(PacketWrite)]
pub struct ItemDefinition {
    pub name: String,
    pub id: i16,
    pub component_based: bool,
    pub item_version: VarInt,

    // Normally would be `Nbt`, but for simplicity elsewhere, this is preserialized (via `Nbt::write_bedrock`)
    #[serial(no_prefix)]
    pub component_data: Vec<u8>,
}
