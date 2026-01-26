use pumpkin_data::packet::clientbound::PLAY_SECTION_BLOCKS_UPDATE;
use pumpkin_util::math::{
    position::{BlockPos, chunk_section_from_pos, pack_local_chunk_section},
    vector3::{self, Vector3},
};

use pumpkin_macros::java_packet;
use serde::{Serialize, ser::SerializeTuple};

use crate::codec::{var_int::VarInt, var_long::VarLong};

/// Updates multiple blocks within a single 16x16x16 chunk section.
///
/// This packet is much more efficient than sending multiple individual
/// `CBlockUpdate` packets when many changes occur in the same area
/// (e.g., explosions, structure generation, or large-scale terraforming).
#[java_packet(PLAY_SECTION_BLOCKS_UPDATE)]
pub struct CMultiBlockUpdate {
    /// The coordinates of the chunk section being updated.
    /// Calculated as (block_coord >> 4).
    pub chunk_section: Vector3<i32>,
    /// A list of relative positions and their new block state IDs.
    /// The i16 encodes the relative position within the section.
    pub positions_to_state_ids: Vec<(i16, i32)>,
}

impl CMultiBlockUpdate {
    pub fn new(positions_to_state_ids: Vec<(BlockPos, u16)>) -> Self {
        let chunk_section = chunk_section_from_pos(&positions_to_state_ids[0].0);
        Self {
            chunk_section,
            positions_to_state_ids: positions_to_state_ids
                .into_iter()
                .map(|(position, state_id)| (pack_local_chunk_section(&position), state_id as i32))
                .collect(),
        }
    }
}

impl Serialize for CMultiBlockUpdate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2 + self.positions_to_state_ids.len())?;

        tuple.serialize_element(&vector3::packed_chunk_pos(&self.chunk_section))?;
        tuple.serialize_element(&VarInt(
            self.positions_to_state_ids.len().try_into().map_err(|_| {
                serde::ser::Error::custom(format!(
                    "{} is not representable as a VarInt!",
                    self.positions_to_state_ids.len()
                ))
            })?,
        ))?;

        for (position, state_id) in &self.positions_to_state_ids {
            let long = (*state_id as u64) << 12 | (*position as u64);
            let var_long = VarLong::from(long as i64);
            tuple.serialize_element(&var_long)?;
        }

        tuple.end()
    }
}
