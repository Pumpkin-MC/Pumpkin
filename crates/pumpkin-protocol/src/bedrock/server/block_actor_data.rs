use pumpkin_macros::packet;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::serial::PacketRead;

/// Synchronizes block-actor updates from the client, such as edited sign text.
#[derive(PacketRead)]
#[packet(56)]
pub struct SBlockActorData {
    /// World coordinates of the block entity being updated.
    pub block_position: BlockPos,
    /// Network NBT compound containing all properties and state of the block entity.
    pub actor_data_tags: NbtCompound,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::{
        Packet,
        bedrock::client::CBlockActorData,
        serial::{PacketRead, PacketWrite},
    };

    #[test]
    fn block_actor_data_roundtrip() {
        assert_eq!(<SBlockActorData as Packet>::PACKET_ID, 56);

        let mut data = NbtCompound::new();
        data.put_string("id", "Sign".to_string());
        data.put_string("Text", "Hello\nWorld".to_string());

        let mut encoded = Vec::new();
        CBlockActorData {
            block_position: BlockPos::new(10, 64, -20),
            actor_data_tags: data,
        }
        .write(&mut encoded)
        .unwrap();

        let mut cursor = Cursor::new(&encoded);
        let parsed = SBlockActorData::read(&mut cursor).unwrap();

        assert_eq!(parsed.block_position, BlockPos::new(10, 64, -20));
        assert_eq!(parsed.actor_data_tags.get_string("id"), Some("Sign"));
        assert_eq!(
            parsed.actor_data_tags.get_string("Text"),
            Some("Hello\nWorld")
        );
    }
}
