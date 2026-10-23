use pumpkin_data::packet::serverbound::play::MOVE_PLAYER_POS;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(MOVE_PLAYER_POS)]
pub struct SPlayerPosition {
    pub position: Vector3<f64>,
    /// bit 0: [`FLAG_ON_GROUND`], bit 1: [`FLAG_IN_WALL`]
    pub collision: u8,
}

impl<'a> ServerPacket<'a> for SPlayerPosition {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let x = bytebuf.get_f64_be()?;
        let y = bytebuf.get_f64_be()?;

        let z = bytebuf.get_f64_be()?;
        let collision = bytebuf.get_u8()?;
        Ok(Self {
            position: Vector3::new(x, y, z),
            collision,
        })
    }
}

impl crate::ClientPacket for SPlayerPosition {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;

        write.write_f64_be(self.position.z)?;
        write.write_u8(self.collision)?;
        Ok(())
    }
}
