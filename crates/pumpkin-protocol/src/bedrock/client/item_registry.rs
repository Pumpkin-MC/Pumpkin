use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Sent by the server to register custom and vanilla item identifiers with their component definitions.
#[derive(PacketWrite)]
#[packet(162)]
pub struct CItemRegistry {
    // https://mojang.github.io/bedrock-protocol-docs/docs/ItemRegistryPacket.html
    /// List of registered item definitions and their component NBT data.
    pub items: Vec<ItemData>,
}

/// Registration data for a single item type in the network item registry.
#[derive(PacketWrite)]
pub struct ItemData {
    /// String identifier for the item (e.g. `minecraft:diamond_sword`).
    pub item_name: String,
    /// Numerical runtime ID of the item.
    pub item_id: i16,
    /// Whether the item uses data-driven components.
    pub is_component_based: bool,

    // TODO: ItemVersion enum
    /// Format version of the item schema.
    pub item_version: VarInt,

    // Normally would be `Nbt`, but for simplicity elsewhere, this is preserialized (via `Nbt::write_bedrock`)
    /// Pre-serialized Bedrock NBT compound representing the item components.
    #[serial(no_prefix)]
    pub component_data: Vec<u8>,
}
