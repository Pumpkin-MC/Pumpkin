use pumpkin_data::packet::serverbound::play::PLAYER_COMMAND;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(PLAYER_COMMAND)]
pub struct SPlayerCommand {
    pub entity_id: VarInt,
    pub action: Action,
    pub jump_boost: VarInt,
}

#[derive(PartialEq, Eq, Debug)]
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

impl<'a> ServerPacket<'a> for SPlayerCommand {
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let (entity_id, action_id, jump_boost) = if *version >= JavaMinecraftVersion::V_1_8 {
            (
                read.get_var_int()?,
                read.get_var_int()?,
                read.get_var_int()?,
            )
        } else {
            (
                VarInt(read.get_i32_be()?),
                VarInt(i32::from(read.get_u8()?)),
                VarInt(read.get_i32_be()?),
            )
        };

        let action = if version < &JavaMinecraftVersion::V_1_21_6 {
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

impl crate::ClientPacket for SPlayerCommand {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        let action_id = if version < &JavaMinecraftVersion::V_1_21_6 {
            match self.action {
                Action::StartSneaking => 0,
                Action::StopSneaking => 1,
                Action::LeaveBed => 2,
                Action::StartSprinting => 3,
                Action::StopSprinting => 4,
                Action::StartHorseJump => 5,
                Action::StopHorseJump => 6,
                Action::OpenVehicleInventory => 7,
                Action::StartFlyingElytra => 8,
            }
        } else {
            match self.action {
                Action::StartSneaking | Action::StopSneaking => {
                    return Err(crate::ser::WritingError::Message(
                        "Sneaking action removed in 1.21.6+".into(),
                    ));
                }
                Action::LeaveBed => 0,
                Action::StartSprinting => 1,
                Action::StopSprinting => 2,
                Action::StartHorseJump => 3,
                Action::StopHorseJump => 4,
                Action::OpenVehicleInventory => 5,
                Action::StartFlyingElytra => 6,
            }
        };
        if *version >= JavaMinecraftVersion::V_1_8 {
            write.write_var_int(&self.entity_id)?;
            write.write_var_int(&VarInt(action_id))?;
            write.write_var_int(&self.jump_boost)?;
        } else {
            write.write_i32_be(self.entity_id.0)?;
            write.write_u8(action_id as u8)?;
            write.write_i32_be(self.jump_boost.0)?;
        }
        Ok(())
    }
}
