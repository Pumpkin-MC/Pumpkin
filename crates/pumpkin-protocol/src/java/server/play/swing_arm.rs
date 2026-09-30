use crate::{ServerPacket, ser::ReadingError};
use pumpkin_data::packet::serverbound::play::PUNCH;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

pub struct SSwingArm {
    pub hand: VarInt,
}

/// 26.3 replaced the swing packet with punch, which no longer tells us the hand.
impl crate::packet::MultiVersionJavaPacket for SSwingArm {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        PUNCH.to_id(version)
    }
}

impl<'a> ServerPacket<'a> for SSwingArm {
    fn read(
        _bytebuf: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let hand = VarInt(0);
        Ok(Self { hand })
    }
}

impl crate::ClientPacket for SSwingArm {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        // The punch packet has no fields
        Ok(())
    }
}
