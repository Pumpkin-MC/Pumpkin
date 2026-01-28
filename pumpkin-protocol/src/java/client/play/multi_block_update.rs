use pumpkin_data::packet::clientbound::PLAY_SECTION_BLOCKS_UPDATE;
use pumpkin_util::math::{
    position::{BlockPos, chunk_section_from_pos, pack_local_chunk_section},
    vector3::{self},
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
    /// Chunk section position (x << 42 | z << 20 | y)
    pub chunk_section: i64,
    /// Array of `VarLongs`: (Block State ID << 12 | Relative Position)
    pub updates: Vec<VarLong>,
}

impl CMultiBlockUpdate {
    #[must_use]
    pub fn new(updates: &[(BlockPos, u16)]) -> Self {
        let first_pos = updates[0].0;

        let chunk_section_vec = chunk_section_from_pos(&first_pos);
        let chunk_section = vector3::packed_chunk_pos(&chunk_section_vec);

        let packed_updates = updates
            .iter()
            .map(|(pos, state_id)| {
                let local_pos = pack_local_chunk_section(pos) as u64;
                let packed = (u64::from(*state_id) << 12) | (local_pos & 0xFFF);
                VarLong(packed as i64)
            })
            .collect();

        Self {
            chunk_section,
            updates: packed_updates,
        }
    }
}

impl Serialize for CMultiBlockUpdate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2 + self.updates.len())?;

        tuple.serialize_element(&self.chunk_section)?;
        tuple.serialize_element(&VarInt(self.updates.len() as i32))?;

        for update in &self.updates {
            tuple.serialize_element(update)?;
        }

        tuple.end()
    }
}
