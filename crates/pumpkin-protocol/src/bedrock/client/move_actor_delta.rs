use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;
use std::io::{Error, Write};

/// Flag indicating the X delta field is present.
pub const MOVE_ACTOR_DELTA_FLAG_HAS_X: u16 = 0x0001;
/// Flag indicating the Y delta field is present.
pub const MOVE_ACTOR_DELTA_FLAG_HAS_Y: u16 = 0x0002;
/// Flag indicating the Z delta field is present.
pub const MOVE_ACTOR_DELTA_FLAG_HAS_Z: u16 = 0x0004;
/// Flag indicating the pitch angle field is present.
pub const MOVE_ACTOR_DELTA_FLAG_HAS_PITCH: u16 = 0x0008;
/// Flag indicating the yaw angle field is present.
pub const MOVE_ACTOR_DELTA_FLAG_HAS_YAW: u16 = 0x0010;
/// Flag indicating the head yaw angle field is present.
pub const MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW: u16 = 0x0020;
/// Flag indicating the entity is on the ground.
pub const MOVE_ACTOR_DELTA_FLAG_ON_GROUND: u16 = 0x0040;
#[deprecated(note = "use MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE")]
pub const MOVE_ACTOR_DELTA_FLAG_TELEPORT: u16 = 0x0080;
/// Flag indicating the movement is forced by the server.
pub const MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE: u16 = 0x0080;
/// Flag indicating movement applies to local player entities.
pub const MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE_LOCAL_ENTITY: u16 = 0x0100;
/// Flag forcing completion of current movement interpolation.
pub const MOVE_ACTOR_DELTA_FLAG_FORCE_COMPLETION: u16 = 0x0200;

/// Bandwidth-optimized delta movement packet for updating an entity's position and rotation.
#[packet(111)]
pub struct CMoveActorDelta {
    /// Runtime entity ID of the moving actor.
    pub entity_runtime_id: VarULong,
    /// Bitmask specifying which coordinates, angles, and flags are present in the payload.
    pub flags: u16,
    /// Delta or absolute X coordinate.
    pub x: f32,
    /// Delta or absolute Y coordinate.
    pub y: f32,
    /// Delta or absolute Z coordinate.
    pub z: f32,
    /// Pitch angle encoded as an 8-bit rotation value.
    pub pitch: u8,
    /// Yaw angle encoded as an 8-bit rotation value.
    pub yaw: u8,
    /// Head yaw angle encoded as an 8-bit rotation value.
    pub head_yaw: u8,
    /// Server world simulation tick number.
    pub tick: VarULong,
}

impl CMoveActorDelta {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        entity_runtime_id: VarULong,
        flags: u16,
        x: f32,
        y: f32,
        z: f32,
        pitch: u8,
        yaw: u8,
        head_yaw: u8,
    ) -> Self {
        Self {
            entity_runtime_id,
            flags,
            x,
            y,
            z,
            pitch,
            yaw,
            head_yaw,
            tick: VarULong(1),
        }
    }
}

impl PacketWrite for CMoveActorDelta {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.entity_runtime_id.write(writer)?;
        let has_x = self.flags & MOVE_ACTOR_DELTA_FLAG_HAS_X != 0;
        has_x.write(writer)?;
        if has_x {
            self.x.write(writer)?;
        }
        let has_y = self.flags & MOVE_ACTOR_DELTA_FLAG_HAS_Y != 0;
        has_y.write(writer)?;
        if has_y {
            self.y.write(writer)?;
        }
        let has_z = self.flags & MOVE_ACTOR_DELTA_FLAG_HAS_Z != 0;
        has_z.write(writer)?;
        if has_z {
            self.z.write(writer)?;
        }
        let has_pitch = self.flags & MOVE_ACTOR_DELTA_FLAG_HAS_PITCH != 0;
        has_pitch.write(writer)?;
        if has_pitch {
            self.pitch.write(writer)?;
        }
        let has_yaw = self.flags & MOVE_ACTOR_DELTA_FLAG_HAS_YAW != 0;
        has_yaw.write(writer)?;
        if has_yaw {
            self.yaw.write(writer)?;
        }
        let has_head_yaw = self.flags & MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW != 0;
        has_head_yaw.write(writer)?;
        if has_head_yaw {
            self.head_yaw.write(writer)?;
        }
        (self.flags & MOVE_ACTOR_DELTA_FLAG_ON_GROUND != 0).write(writer)?;
        (self.flags & MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE != 0).write(writer)?;
        (self.flags & MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE_LOCAL_ENTITY != 0).write(writer)?;
        (self.flags & MOVE_ACTOR_DELTA_FLAG_FORCE_COMPLETION != 0).write(writer)?;
        self.tick.write(writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_actor_delta_appends_tick_after_flags() {
        let mut packet = CMoveActorDelta::new(VarULong(1), 0, 0.0, 0.0, 0.0, 0, 0, 0);
        assert_eq!(packet.tick.0, 1);
        packet.tick = VarULong(300);
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(bytes, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 172, 2]);
    }
}
