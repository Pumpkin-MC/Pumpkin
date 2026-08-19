use crate::ClientPacket;
use crate::VarInt;
use crate::packet::MultiVersionJavaPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::{PLAY_ENTITY_POSITION_SYNC, PLAY_TELEPORT_ENTITY};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;

/// Updates the exact position, rotation, and velocity of an entity.
///
/// This packet is used for server-side authority over entity movement.
/// In the latest protocol versions, this replaces several older "Relative Move"
/// packets to provide more precise synchronization and reduce "rubber-banding."
///
/// Note: This packet must NOT be used for the player receiving the packet or
/// any entity the player is currently riding.
pub struct CEntityPositionSync {
    /// The Entity ID of the entity being moved.
    pub entity_id: VarInt,
    /// The absolute position of the entity in the world.
    pub position: Vector3<f64>,
    /// The current velocity (delta) of the entity, used by the client
    /// for smooth interpolation.
    pub delta: Vector3<f64>,
    /// The absolute yaw (horizontal rotation) in degrees.
    pub yaw: f32,
    /// The absolute pitch (vertical rotation) in degrees.
    pub pitch: f32,
    /// Whether the entity is currently touching the ground.
    pub on_ground: bool,
}

impl CEntityPositionSync {
    #[must_use]
    pub const fn new(
        entity_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            position,
            delta,
            yaw,
            pitch,
            on_ground,
        }
    }
}

impl MultiVersionJavaPacket for CEntityPositionSync {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        if version >= JavaMinecraftVersion::V_1_21_2 {
            PLAY_ENTITY_POSITION_SYNC.to_id(version)
        } else {
            PLAY_TELEPORT_ENTITY.to_id(version)
        }
    }
}

impl ClientPacket for CEntityPositionSync {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            write.write_f64_be(self.delta.x)?;
            write.write_f64_be(self.delta.y)?;
            write.write_f64_be(self.delta.z)?;
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
        } else {
            write.write_u8((self.yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8)?;
            write.write_u8((self.pitch.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8)?;
        }
        write.write_bool(self.on_ground)?;
        Ok(())
    }
}
