use pumpkin_macros::packet;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::{codec::var_long::VarLong, serial::PacketWrite};

/// Pose/emote the entity displays while riding the block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(u8)]
pub enum PassengerOfBlockEmote {
    Standing = 0,
    Riding = 1,
    Laying = 2,
}

/// Data for a block that an entity is riding.
#[derive(Clone, Debug, PartialEq, PacketWrite)]
pub struct PassengerOfBlockData {
    /// World position of the block.
    pub block_position: BlockPos,
    /// Offset relative to the block position.
    pub offset: Vector3<f32>,
    /// Rotation in degrees from 0 to 360.
    pub rotation: f32,
    /// Rotation limit in degrees from 0 to 360.
    pub rotation_limit: f32,
    /// Visual riding emote/pose.
    pub emote_type: u8,
}

/// Sent by the server to attach or detach an entity to/from a block.
#[derive(Clone, Debug, PartialEq, PacketWrite)]
#[packet(335)]
pub struct CSetPassengerOfBlock {
    /// Unique entity ID of the passenger.
    pub passenger_unique_id: VarLong,
    /// Block passenger data, or None to stop riding.
    pub passenger_of_block: Option<PassengerOfBlockData>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_none_passenger_of_block_as_false() {
        let packet = CSetPassengerOfBlock {
            passenger_unique_id: VarLong(42),
            passenger_of_block: None,
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).expect("write packet");
        // VarLong(42) zigzag is 84, followed by false (0)
        assert_eq!(encoded, [84, 0]);
    }
}
