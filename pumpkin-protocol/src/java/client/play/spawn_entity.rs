use std::io::Write;

use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_id_remap::remap_entity_id_for_version;
use pumpkin_data::packet::clientbound::PLAY_ADD_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

use crate::{
    ClientPacket, VarInt,
    codec::velocity::Velocity,
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(PLAY_ADD_ENTITY)]
pub struct CSpawnEntity {
    pub entity_id: VarInt,
    pub entity_uuid: uuid::Uuid,
    pub r#type: VarInt,
    pub position: Vector3<f64>,
    pub velocity: Velocity,
    pub pitch: u8,    // angle
    pub yaw: u8,      // angle
    pub head_yaw: u8, // angle
    pub data: VarInt,
}

impl CSpawnEntity {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        entity_id: VarInt,
        entity_uuid: uuid::Uuid,
        r#type: VarInt,
        position: Vector3<f64>,
        pitch: f32,    // angle
        yaw: f32,      // angle
        head_yaw: f32, // angle
        data: VarInt,
        velocity: Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            pitch: (pitch * 256.0 / 360.0).floor() as u8,
            yaw: (yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8,
            head_yaw: (head_yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8,
            data,
            velocity: Velocity(velocity),
        }
    }
}

impl ClientPacket for CSpawnEntity {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;
        write.write_uuid(&self.entity_uuid)?;
        let remapped_type =
            VarInt(remap_entity_id_for_version(self.r#type.0 as u16, *version) as i32);
        write.write_var_int(&remapped_type)?;

        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;

        // Angles
        if version >= &MinecraftVersion::V_1_21_9 {
            self.velocity.write(&mut write)?;
        }
        write.write_u8(self.pitch)?;
        write.write_u8(self.yaw)?;
        write.write_u8(self.head_yaw)?;

        let data = if self.r#type.0 == i32::from(EntityType::FALLING_BLOCK.id) {
            u16::try_from(self.data.0).map_or(self.data, |state_id| {
                VarInt(i32::from(remap_block_state_for_version(state_id, *version)))
            })
        } else {
            self.data
        };
        write.write_var_int(&data)?;

        if version < &MinecraftVersion::V_1_21_9 {
            self.velocity.write_legacy(&mut write)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CSpawnEntity;
    use crate::{ClientPacket, VarInt, codec::velocity::encode_legacy_velocity_component};
    use pumpkin_util::version::MinecraftVersion;

    fn legacy_tail(velocity: pumpkin_util::math::vector3::Vector3<f64>) -> [u8; 6] {
        let x = encode_legacy_velocity_component(velocity.x);
        let y = encode_legacy_velocity_component(velocity.y);
        let z = encode_legacy_velocity_component(velocity.z);
        let xb = x.to_be_bytes();
        let yb = y.to_be_bytes();
        let zb = z.to_be_bytes();
        [xb[0], xb[1], yb[0], yb[1], zb[0], zb[1]]
    }

    fn encode_spawn(version: MinecraftVersion) -> Vec<u8> {
        let velocity = pumpkin_util::math::vector3::Vector3::new(0.5, -0.5, 0.25);
        let packet = CSpawnEntity::new(
            VarInt(1),
            uuid::Uuid::nil(),
            VarInt(1),
            pumpkin_util::math::vector3::Vector3::new(1.0, 2.0, 3.0),
            0.0,
            90.0,
            90.0,
            VarInt(42),
            velocity,
        );
        let mut out = Vec::new();
        packet.write_packet_data(&mut out, &version).unwrap();
        out
    }

    #[test]
    fn spawn_entity_uses_legacy_velocity_tail_for_1_21_8() {
        // V_1_21_7 enum variant represents protocol 772 (used by 1.21.7 and 1.21.8).
        let velocity = pumpkin_util::math::vector3::Vector3::new(0.5, -0.5, 0.25);
        let expected_tail = legacy_tail(velocity);
        let encoded = encode_spawn(MinecraftVersion::V_1_21_7);

        assert!(encoded.ends_with(&expected_tail));
    }

    #[test]
    fn spawn_entity_does_not_use_legacy_velocity_tail_for_1_21_9() {
        let velocity = pumpkin_util::math::vector3::Vector3::new(0.5, -0.5, 0.25);
        let expected_tail = legacy_tail(velocity);
        let encoded = encode_spawn(MinecraftVersion::V_1_21_9);

        assert!(!encoded.ends_with(&expected_tail));
    }
}
