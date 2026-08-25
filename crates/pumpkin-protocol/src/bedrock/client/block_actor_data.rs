// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::serial::PacketWrite;

/// Synchronizes the complete block-actor data for a block position.
#[derive(PacketWrite)]
#[packet(56)]
pub struct CBlockActorData {
    pub block_position: BlockPos,
    pub actor_data_tags: NbtCompound,
}

impl CBlockActorData {
    #[must_use]
    pub const fn new(block_position: BlockPos, actor_data_tags: NbtCompound) -> Self {
        Self {
            block_position,
            actor_data_tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pumpkin_nbt::{Nbt, deserializer::NbtReadHelperBedrock};

    use super::*;
    use crate::{Packet, serial::PacketWrite};

    #[test]
    fn block_actor_data_uses_bedrock_network_nbt() {
        assert_eq!(<CBlockActorData as Packet>::PACKET_ID, 56);

        let mut data = NbtCompound::new();
        data.put_string("id", "Bed".to_string());
        data.put_byte("color", 11);

        let mut encoded = Vec::new();
        CBlockActorData {
            block_position: BlockPos::new(1, 64, -2),
            actor_data_tags: data,
        }
        .write(&mut encoded)
        .unwrap();

        assert_eq!(&encoded[..4], &[2, 128, 1, 3]);
        let mut reader = NbtReadHelperBedrock::new(Cursor::new(&encoded[4..]));
        let parsed = Nbt::read(&mut reader).unwrap();
        assert_eq!(parsed.get_string("id"), Some("Bed"));
        assert_eq!(parsed.get_byte("color"), Some(11));
    }
}
