use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_PLAYER_COMMAND;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(PLAY_PLAYER_COMMAND)]
pub struct SPlayerCommand {
    pub entity_id: VarInt,
    pub action: Action,
    pub jump_boost: VarInt,
}

pub enum Action {
    // <=1.21.5
    StartSneaking,
    // <=1.21.5
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

impl ServerPacket for SPlayerCommand {
    fn read(mut read: impl Read, version: &MinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = read.get_var_int()?;
        let action_id = read.get_var_int()?;
        let jump_boost = read.get_var_int()?;

        let action = if version < &MinecraftVersion::V_1_21_6 {
            match action_id.0 {
                0 => Ok(Action::StartSneaking),
                1 => Ok(Action::StopSneaking),
                2 => Ok(Action::LeaveBed),
                3 => Ok(Action::StartSprinting),
                4 => Ok(Action::StopSprinting),
                5 => Ok(Action::StartHorseJump),
                6 => Ok(Action::StopHorseJump),
                7 => Ok(Action::OpenVehicleInventory),
                8 => Ok(Action::StartFlyingElytra),
                _ => Err(ReadingError::Message("Invalid player command".to_string())),
            }
        } else {
            match action_id.0 {
                0 => Ok(Action::LeaveBed),
                1 => Ok(Action::StartSprinting),
                2 => Ok(Action::StopSprinting),
                3 => Ok(Action::StartHorseJump),
                4 => Ok(Action::StopHorseJump),
                5 => Ok(Action::OpenVehicleInventory),
                6 => Ok(Action::StartFlyingElytra),
                _ => Err(ReadingError::Message("Invalid player command".to_string())),
            }
        }?;
        Ok(Self {
            entity_id,
            action,
            jump_boost,
        })
    }
}
