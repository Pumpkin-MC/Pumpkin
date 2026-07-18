use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(Clone, Copy)]
#[packet(43)]
pub struct CSetSpawnPosition {
    pub spawn_type: VarInt,
    pub position: BlockPos,
    pub dimension: VarInt,
    pub spawn_position: BlockPos,
}

impl CSetSpawnPosition {
    #[must_use]
    pub const fn new(
        spawn_type: i32,
        position: BlockPos,
        dimension: i32,
        spawn_position: BlockPos,
    ) -> Self {
        Self {
            spawn_type: VarInt(spawn_type),
            position,
            dimension: VarInt(dimension),
            spawn_position,
        }
    }
}

impl PacketWrite for CSetSpawnPosition {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.spawn_type.write(writer)?;
        self.position.write(writer)?;
        self.dimension.write(writer)?;
        self.spawn_position.write(writer)?;
        Ok(())
    }
}
