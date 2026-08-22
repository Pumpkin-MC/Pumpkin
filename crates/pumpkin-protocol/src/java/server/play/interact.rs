use pumpkin_data::packet::serverbound::play::INTERACT;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{
    ServerPacket,
    codec::{lp_vector_3d::LpVector3d, var_int::VarInt},
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(INTERACT)]
pub struct SInteract {
    pub entity_id: VarInt,
    pub r#type: VarInt,
    pub target_position: Option<Vector3<f64>>,
    pub hand: Option<VarInt>,
    pub sneaking: bool,
}

// Great job Mojang ;D
impl<'a> ServerPacket<'a> for SInteract {
    fn read(mut read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        // 26.1+ removes the 'type' field and uses doubles for location
        if version >= &JavaMinecraftVersion::V_26_1 {
            let entity_id = read.get_var_int()?;
            let hand = Some(read.get_var_int()?);
            let target_position = Some(LpVector3d::read(&mut read)?.0);
            let sneaking = read.get_bool()?;

            return Ok(Self {
                entity_id,
                r#type: VarInt(2), // InteractAt for compatibility
                target_position,
                hand,
                sneaking,
            });
        }

        let entity_id = if version >= &JavaMinecraftVersion::V_1_8 {
            read.get_var_int()?
        } else {
            VarInt(read.get_i32_be()?)
        };

        let r#type = if version >= &JavaMinecraftVersion::V_1_8 {
            read.get_var_int()?
        } else {
            VarInt(i32::from(read.get_u8()?))
        };

        let action = ActionType::try_from(r#type.0)
            .map_err(|_| ReadingError::Message("invalid action type".to_string()))?;

        let target_position: Option<Vector3<f64>> = match action {
            ActionType::Interact | ActionType::Attack => None,
            ActionType::InteractAt => {
                if version >= &JavaMinecraftVersion::V_1_8 {
                    Some(
                        Vector3::new(read.get_f32_be()?, read.get_f32_be()?, read.get_f32_be()?)
                            .to_f64(),
                    )
                } else {
                    None
                }
            }
        };

        let hand = if version >= &JavaMinecraftVersion::V_1_9 {
            match action {
                ActionType::Interact | ActionType::InteractAt => Some(read.get_var_int()?),
                ActionType::Attack => None,
            }
        } else {
            match action {
                ActionType::Interact | ActionType::InteractAt => Some(VarInt(0)),
                ActionType::Attack => None,
            }
        };

        let sneaking = if version >= &JavaMinecraftVersion::V_1_16 {
            read.get_bool()?
        } else {
            false
        };

        Ok(Self {
            entity_id,
            r#type,
            target_position,
            hand,
            sneaking,
        })
    }
}

impl crate::ClientPacket for SInteract {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        if version >= &JavaMinecraftVersion::V_26_1 {
            write.write_var_int(&self.entity_id)?;
            write.write_var_int(&self.hand.unwrap_or(VarInt(0)))?;
            if let Some(pos) = self.target_position {
                LpVector3d(pos).write(&mut write)?;
            } else {
                LpVector3d(Vector3::new(0.0, 0.0, 0.0)).write(&mut write)?;
            }
            write.write_bool(self.sneaking)?;
            return Ok(());
        }

        if version >= &JavaMinecraftVersion::V_1_8 {
            write.write_var_int(&self.entity_id)?;
            write.write_var_int(&self.r#type)?;
            let action = ActionType::try_from(self.r#type.0).map_err(|_| {
                crate::ser::WritingError::Message("invalid action type".to_string())
            })?;
            if action == ActionType::InteractAt {
                if let Some(target) = self.target_position {
                    write.write_f32_be(target.x as f32)?;
                    write.write_f32_be(target.y as f32)?;
                    write.write_f32_be(target.z as f32)?;
                } else {
                    write.write_f32_be(0.0)?;
                    write.write_f32_be(0.0)?;
                    write.write_f32_be(0.0)?;
                }
            }
            if version >= &JavaMinecraftVersion::V_1_9
                && (action == ActionType::Interact || action == ActionType::InteractAt)
            {
                write.write_var_int(&self.hand.unwrap_or(VarInt(0)))?;
            }
            if version >= &JavaMinecraftVersion::V_1_16 {
                write.write_bool(self.sneaking)?;
            }
        } else {
            write.write_i32_be(self.entity_id.0)?;
            write.write_u8(self.r#type.0 as u8)?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ActionType {
    Interact = 0,
    Attack = 1,
    InteractAt = 2,
}

#[derive(Debug)]
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
