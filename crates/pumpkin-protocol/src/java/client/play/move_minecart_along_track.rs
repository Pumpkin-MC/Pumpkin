use pumpkin_data::packet::clientbound::play::MOVE_MINECART_ALONG_TRACK;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::{
    math::{pack_degrees, vector3::Vector3},
    version::JavaMinecraftVersion,
};

/// Vanilla `NewMinecartBehavior.MinecartStep`.
#[derive(Clone, Copy)]
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
            // `ByteBufCodecs.ROTATION_BYTE`
            write.write_u8(pack_degrees(step.yaw))?;
            write.write_u8(pack_degrees(step.pitch))?;
            write.write_f32_be(step.weight)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_rotations_are_bytes() {
        let step = MinecartStep {
            position: Vector3::new(0.0, 0.0, 0.0),
            movement: Vector3::new(0.0, 0.0, 0.0),
            yaw: 90.0,
            pitch: -45.0,
            weight: 1.0,
        };
        let mut buf = Vec::new();
        CMoveMinecartAlongTrack::new(VarInt(5), &[step])
            .write_packet_data(&mut buf, &JavaMinecraftVersion::V_26_3)
            .unwrap();
        // id, step count, six doubles, `ROTATION_BYTE` yaw and pitch, float weight.
        assert_eq!(buf.len(), 1 + 1 + 6 * 8 + 2 + 4);
        assert_eq!(buf[50..], [64, 0xE0, 0x3F, 0x80, 0x00, 0x00]);
    }
}
