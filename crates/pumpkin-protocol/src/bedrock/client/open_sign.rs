use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::serial::PacketWrite;

/// Sent by the server to open the sign text editor on the Bedrock client.
#[derive(PacketWrite)]
#[packet(303)]
pub struct COpenSign {
    /// Position of the sign being edited.
    pub position: BlockPos,
    /// Whether the front side of the sign is being edited.
    pub front_side: bool,
}

impl COpenSign {
    #[must_use]
    pub const fn new(position: BlockPos, front_side: bool) -> Self {
        Self {
            position,
            front_side,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Packet;

    #[test]
    fn open_sign_packet_id() {
        assert_eq!(<COpenSign as Packet>::PACKET_ID, 303);
    }
}
