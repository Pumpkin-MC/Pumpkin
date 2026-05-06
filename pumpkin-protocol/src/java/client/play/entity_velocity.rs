use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_MOTION;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

use crate::{
    ClientPacket, VarInt, WritingError, codec::lp_vector_3d::LpVector3d, ser::NetworkWriteExt,
};

/// Updates the velocity of an entity.
///
/// This packet informs the client of a sudden change in an entity's movement,
/// such as knockback from an attack, explosions, or being launched by a piston.
#[java_packet(PLAY_SET_ENTITY_MOTION)]
pub struct CEntityVelocity {
    /// The Entity ID of the entity whose velocity is being set
    pub entity_id: VarInt,
    /// The velocity vector
    pub velocity: LpVector3d,
}

impl CEntityVelocity {
    #[must_use]
    pub const fn new(entity_id: VarInt, velocity: Vector3<f64>) -> Self {
        Self {
            entity_id,
            velocity: LpVector3d(velocity),
        }
    }
}

impl ClientPacket for CEntityVelocity {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;

        // Protocol 773+ uses packed velocity; 772 and below use three i16 components.
        if version >= &MinecraftVersion::V_1_21_9 {
            self.velocity.write(&mut write)?;
        } else {
            self.velocity.write_legacy(&mut write)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::version::MinecraftVersion;

    use super::CEntityVelocity;
    use crate::{
        ClientPacket, VarInt,
        codec::lp_vector_3d::{LpVector3d, encode_legacy_velocity_component},
    };

    fn encode_packet(version: MinecraftVersion) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let packet = CEntityVelocity::new(
            VarInt(1),
            pumpkin_util::math::vector3::Vector3::new(0.5, -0.5, 0.25),
        );
        let mut out = Vec::new();
        packet.write_packet_data(&mut out, &version)?;
        Ok(out)
    }

    fn legacy_bytes(x: f64, y: f64, z: f64) -> Vec<u8> {
        let mut out = Vec::with_capacity(6);
        out.extend_from_slice(&encode_legacy_velocity_component(x).to_be_bytes());
        out.extend_from_slice(&encode_legacy_velocity_component(y).to_be_bytes());
        out.extend_from_slice(&encode_legacy_velocity_component(z).to_be_bytes());
        out
    }

    #[test]
    fn entity_velocity_uses_legacy_format_for_1_21_8() -> Result<(), Box<dyn std::error::Error>> {
        // V_1_21_7 enum variant represents protocol 772 (used by 1.21.7 and 1.21.8).
        let encoded = encode_packet(MinecraftVersion::V_1_21_7)?;
        let expected_legacy_tail = legacy_bytes(0.5, -0.5, 0.25);

        assert_eq!(encoded, [&[1], expected_legacy_tail.as_slice()].concat());
        Ok(())
    }

    #[test]
    fn entity_velocity_uses_packed_format_for_1_21_9() -> Result<(), Box<dyn std::error::Error>> {
        let encoded = encode_packet(MinecraftVersion::V_1_21_9)?;
        let legacy_like = legacy_bytes(0.5, -0.5, 0.25);

        assert_ne!(&encoded[1..], legacy_like.as_slice());

        // Ensure the packed bytes can still be decoded back to a velocity.
        let mut cursor = std::io::Cursor::new(&encoded[1..]);
        let decoded = LpVector3d::read(&mut cursor)?;
        assert!(decoded.0.x.is_finite());
        assert!(decoded.0.y.is_finite());
        assert!(decoded.0.z.is_finite());
        Ok(())
    }
}
