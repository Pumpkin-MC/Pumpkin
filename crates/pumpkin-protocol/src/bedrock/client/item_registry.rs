use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(162)]
pub struct CItemRegistry {
    // https://mojang.github.io/bedrock-protocol-docs/docs/ItemRegistryPacket.html
    pub items: Vec<ItemData>,
}

#[derive(PacketWrite)]
pub struct ItemData {
    pub item_name: String,
    pub item_id: i16,
    pub is_component_based: bool,

    // TODO: ItemVersion enum
    pub item_version: VarInt,

    // Normally would be `Nbt`, but for simplicity elsewhere, this is preserialized (via `Nbt::write_bedrock`)
    #[serial(no_prefix)]
    pub component_data: Vec<u8>,
}
