use pumpkin_data::packet::clientbound::play::MOVE_MINECART_ALONG_TRACK;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

pub struct MinecartStep {
    pub position: Vector3<f64>,
    pub movement: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub weight: f32,
}

#[java_packet(MOVE_MINECART_ALONG_TRACK)]
pub struct CMoveMinecartAlongTrack<'a> {
    pub entity_id: VarInt,
    pub steps: &'a [MinecartStep],
}

impl<'a> CMoveMinecartAlongTrack<'a> {
    #[must_use]
    pub const fn new(entity_id: VarInt, steps: &'a [MinecartStep]) -> Self {
        Self { entity_id, steps }
    }
}

impl ClientPacket for CMoveMinecartAlongTrack<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_var_int(&VarInt(self.steps.len() as i32))?;
        for step in self.steps {
            write.write_f64_be(step.position.x)?;
            write.write_f64_be(step.position.y)?;
            write.write_f64_be(step.position.z)?;
            write.write_f64_be(step.movement.x)?;
            write.write_f64_be(step.movement.y)?;
            write.write_f64_be(step.movement.z)?;
            write.write_f32_be(step.yaw)?;
            write.write_f32_be(step.pitch)?;
            write.write_f32_be(step.weight)?;
        }
        Ok(())
    }
}
