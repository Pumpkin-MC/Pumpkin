use pumpkin_data::packet::serverbound::PLAY_SET_COMMAND_BLOCK;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

use crate::codec::var_int::VarInt;

#[derive(Deserialize)]
#[packet(PLAY_SET_COMMAND_BLOCK)]
pub struct SSetCommandBlock {
    pub pos: BlockPos,
    pub command: String,
    pub mode: VarInt,
    pub flags: i8,
}

pub enum CommandBlockMode {
    Chain,
    Repeating,
    /// Redstone only
    Impulse,
}

impl TryFrom<VarInt> for CommandBlockMode {
    type Error = ();

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        match value.0 {
            0 => Ok(Self::Chain),
            1 => Ok(Self::Repeating),
            2 => Ok(Self::Impulse),
            _ => Err(()),
        }
    }
}
