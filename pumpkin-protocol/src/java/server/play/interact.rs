use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_INTERACT;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

#[packet(PLAY_INTERACT)]
pub struct SInteract {
    pub entity_id: VarInt,
    pub r#type: VarInt,
    pub target_position: Option<Vector3<f32>>,
    pub hand: Option<VarInt>,
    pub sneaking: bool,
}

// Great job Mojang ;D
impl ServerPacket for SInteract {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        let entity_id = read.get_var_int()?;
        let r#type = read.get_var_int()?;
        let action = ActionType::try_from(r#type.0)
            .map_err(|_| ReadingError::Message("invalid action type".to_string()))?;
        let target_position: Option<Vector3<f32>> = match action {
            ActionType::Interact => None,
            ActionType::Attack => None,
            ActionType::InteractAt => Some(Vector3::new(
                read.get_f32_be()?,
                read.get_f32_be()?,
                read.get_f32_be()?,
            )),
        };
        let hand = match action {
            ActionType::Interact => Some(read.get_var_int()?),
            ActionType::Attack => None,
            ActionType::InteractAt => Some(read.get_var_int()?),
        };

        Ok(Self {
            entity_id,
            r#type,
            target_position,
            hand,
            sneaking: read.get_bool()?,
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ActionType {
    Interact,
    Attack,
    InteractAt,
}

pub struct InvalidActionType;

impl TryFrom<i32> for ActionType {
    type Error = InvalidActionType;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Interact),
            1 => Ok(Self::Attack),
            2 => Ok(Self::InteractAt),
            _ => Err(InvalidActionType),
        }
    }
}
