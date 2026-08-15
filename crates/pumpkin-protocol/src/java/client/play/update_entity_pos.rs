use pumpkin_data::packet::clientbound::PLAY_MOVE_ENTITY_POS;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_MOVE_ENTITY_POS)]
pub struct CUpdateEntityPos {
    pub entity_id: VarInt,
    pub delta: Vector3<i16>,
    pub on_ground: bool,
}

impl CUpdateEntityPos {
    #[must_use]
    pub const fn new(entity_id: VarInt, delta: Vector3<i16>, on_ground: bool) -> Self {
        Self {
            entity_id,
            delta,
            on_ground,
        }
    }
}

impl ClientPacket for CUpdateEntityPos {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_i16_be(self.delta.x)?;
        write.write_i16_be(self.delta.y)?;
        write.write_i16_be(self.delta.z)?;
        write.write_bool(self.on_ground)?;
        Ok(())
    }
}
