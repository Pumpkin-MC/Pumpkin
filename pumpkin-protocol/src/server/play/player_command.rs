use bytes::Buf;
use pumpkin_data::packet::serverbound::PLAY_PLAYER_COMMAND;
use pumpkin_macros::packet;

use crate::{
    ServerPacket, VarInt,
    bytebuf::{ByteBuf, ReadingError},
};

#[packet(PLAY_PLAYER_COMMAND)]
pub struct SPlayerCommand {
    pub entity_id: VarInt,
    pub action: VarInt,
    pub jump_boost: VarInt,
}

pub enum Action {
    StartSneaking = 0,
    StopSneaking,
    LeaveBed,
    StartSprinting,
    StopSprinting,
    StartHorseJump,
    StopHorseJump,
    OpenVehicleInventory,
    StartFlyingElytra,
}

pub struct InvalidAction;

impl TryFrom<i32> for Action {
    type Error = InvalidAction;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::StartSneaking),
            1 => Ok(Self::StopSneaking),
            2 => Ok(Self::LeaveBed),
            3 => Ok(Self::StartSprinting),
            4 => Ok(Self::StopSprinting),
            5 => Ok(Self::StartHorseJump),
            6 => Ok(Self::StopHorseJump),
            7 => Ok(Self::OpenVehicleInventory),
            8 => Ok(Self::StartFlyingElytra),
            _ => Err(InvalidAction),
        }
    }
}

impl ServerPacket for SPlayerCommand {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            entity_id: bytebuf.try_get_var_int()?,
            action: bytebuf.try_get_var_int()?,
            jump_boost: bytebuf.try_get_var_int()?,
        })
    }
}
