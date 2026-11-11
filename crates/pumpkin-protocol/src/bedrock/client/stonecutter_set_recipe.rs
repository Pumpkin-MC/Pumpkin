use pumpkin_macros::packet;

use crate::{
    codec::{var_int::VarInt, var_long::VarLong},
    serial::PacketWrite,
};

/// Sent by the server to synchronize the selected stonecutter recipe index to the client.
#[derive(Clone, Debug, PartialEq, Eq, PacketWrite)]
#[packet(333)]
pub struct CStonecutterSetRecipe {
    /// Entity unique ID of the player owning the container.
    pub player_unique_id: VarLong,
    /// Container ID of the stonecutter.
    pub container_id: u8,
    /// Selected recipe index.
    pub recipe_index: VarInt,
}
