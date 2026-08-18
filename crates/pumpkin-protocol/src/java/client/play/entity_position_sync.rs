use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_ENTITY_POSITION_SYNC;
use pumpkin_macros::java_packet;
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
#[java_packet(PLAY_ENTITY_POSITION_SYNC)]
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

impl ClientPacket for CEntityPositionSync {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_f64(self.position.x)?;
        write.write_f64(self.position.y)?;
        write.write_f64(self.position.z)?;
        write.write_f64(self.delta.x)?;
        write.write_f64(self.delta.y)?;
        write.write_f64(self.delta.z)?;
        write.write_f32(self.yaw)?;
        write.write_f32(self.pitch)?;
        write.write_bool(self.on_ground)?;
        Ok(())
    }
}
