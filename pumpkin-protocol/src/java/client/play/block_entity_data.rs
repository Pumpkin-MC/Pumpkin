use pumpkin_data::packet::clientbound::PLAY_BLOCK_ENTITY_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use serde::Serialize;

use crate::{VarInt, ser::network_serialize_no_prefix};

/// Updates the NBT data of a block entity (e.g., signs, chests, or banners).
///
/// This packet is sent by the server when a block entity's state changes
/// (like text on a sign) or when the block entity is loaded into the client's view.
#[derive(Serialize)]
#[java_packet(PLAY_BLOCK_ENTITY_DATA)]
pub struct CBlockEntityData {
    /// The world coordinates of the block entity.
    pub location: BlockPos,
    /// The type of block entity being updated (e.g., Mob Spawner, Command Block).
    pub r#type: VarInt,
    /// The raw NBT payload containing the block's specific data.
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub nbt_data: Box<[u8]>,
}

impl CBlockEntityData {
    pub fn new(location: BlockPos, r#type: VarInt, nbt_data: Box<[u8]>) -> Self {
        Self {
            location,
            r#type,
            nbt_data,
        }
    }
}
