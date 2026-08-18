use pumpkin_data::packet::clientbound::PLAY_HURT_ANIMATION;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Triggers the "hurt" visual effect on an entity.
///
/// This packet causes the entity to turn red and perform a directional
/// camera shake or model tilt. It is typically sent immediately after
/// an entity's health is reduced.
#[java_packet(PLAY_HURT_ANIMATION)]
pub struct CHurtAnimation {
    /// The Entity ID of the entity that was hurt.
    pub entity_id: VarInt,
    /// The yaw (direction) from which the damage originated.
    /// This determines the direction the entity's model tilts.
    pub yaw: f32,
}

impl CHurtAnimation {
    #[must_use]
    pub const fn new(entity_id: VarInt, yaw: f32) -> Self {
        Self { entity_id, yaw }
    }
}

impl ClientPacket for CHurtAnimation {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_f32_be(self.yaw)?;
        Ok(())
    }
}
