use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_STATUS_ONLY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use super::{FLAG_HORIZONTAL_COLLISION, FLAG_ON_GROUND};

#[java_packet(PLAY_MOVE_PLAYER_STATUS_ONLY)]
pub struct SSetPlayerGround {
    pub on_ground: bool,
    pub horizontal_collision: bool,
}

impl<'a> ServerPacket<'a> for SSetPlayerGround {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let collision = bytebuf.get_u8()?;
        Ok(Self {
            on_ground: collision & FLAG_ON_GROUND != 0,
            horizontal_collision: collision & FLAG_HORIZONTAL_COLLISION != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_movement_status_flag() {
        let mut horizontal_only = [FLAG_HORIZONTAL_COLLISION].as_slice();
        let packet =
            SSetPlayerGround::read(&mut horizontal_only, &JavaMinecraftVersion::V_26_2).unwrap();
        assert!(!packet.on_ground);
        assert!(packet.horizontal_collision);

        let mut both = [FLAG_ON_GROUND | FLAG_HORIZONTAL_COLLISION].as_slice();
        let packet = SSetPlayerGround::read(&mut both, &JavaMinecraftVersion::V_26_2).unwrap();
        assert!(packet.on_ground);
        assert!(packet.horizontal_collision);
    }
}
