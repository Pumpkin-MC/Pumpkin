// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Updates a client-side block animation, such as a chest lid opening or closing.
#[derive(PacketWrite)]
#[packet(26)]
pub struct CBlockEvent {
    /// World coordinates of the block undergoing the event.
    pub block_position: BlockPos,
    /// Type identifier of the block action being executed.
    pub event_type: VarInt,
    /// Type-specific integer parameter (e.g. 1 to open a chest lid, 0 to close it).
    pub event_value: VarInt,
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::position::BlockPos;

    use super::*;
    use crate::{Packet, serial::PacketWrite};

    #[test]
    fn chest_lid_event_uses_bedrock_wire_format() {
        assert_eq!(<CBlockEvent as Packet>::PACKET_ID, 26);

        let mut encoded = Vec::new();
        CBlockEvent {
            block_position: BlockPos::new(1, 64, -2),
            event_type: 1.into(),
            event_value: 3.into(),
        }
        .write(&mut encoded)
        .unwrap();

        assert_eq!(encoded, [2, 128, 1, 3, 2, 6]);
    }
}
