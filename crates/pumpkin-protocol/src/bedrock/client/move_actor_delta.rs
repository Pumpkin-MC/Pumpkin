use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;
use std::io::{Error, Write};

pub const MOVE_ACTOR_DELTA_FLAG_HAS_X: u16 = 0x0001;
pub const MOVE_ACTOR_DELTA_FLAG_HAS_Y: u16 = 0x0002;
pub const MOVE_ACTOR_DELTA_FLAG_HAS_Z: u16 = 0x0004;
pub const MOVE_ACTOR_DELTA_FLAG_HAS_PITCH: u16 = 0x0008;
pub const MOVE_ACTOR_DELTA_FLAG_HAS_YAW: u16 = 0x0010;
pub const MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW: u16 = 0x0020;
pub const MOVE_ACTOR_DELTA_FLAG_ON_GROUND: u16 = 0x0040;
#[deprecated(note = "use MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE")]
pub const MOVE_ACTOR_DELTA_FLAG_TELEPORT: u16 = 0x0080;
pub const MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE: u16 = 0x0080;
pub const MOVE_ACTOR_DELTA_FLAG_FORCE_MOVE_LOCAL_ENTITY: u16 = 0x0100;
pub const MOVE_ACTOR_DELTA_FLAG_FORCE_COMPLETION: u16 = 0x0200;

#[packet(111)]
pub struct CMoveActorDelta {
    pub entity_runtime_id: VarULong,
    pub flags: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub pitch: u8,
    pub yaw: u8,
    pub head_yaw: u8,
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
        Ok(())
    }
}
