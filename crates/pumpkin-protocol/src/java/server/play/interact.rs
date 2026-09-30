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
    fn read(
        mut read: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        // 26.1+ removes the 'type' field and uses doubles for location
        let entity_id = read.get_var_int()?;
        let hand = Some(read.get_var_int()?);
        let target_position = Some(LpVector3d::read(&mut read)?.0);
        let sneaking = read.get_bool()?;

        Ok(Self {
            entity_id,
            r#type: VarInt(2), // InteractAt for compatibility
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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.entity_id)?;
        write.write_var_int(&self.hand.unwrap_or(VarInt(0)))?;
        if let Some(pos) = self.target_position {
            LpVector3d(pos).write(&mut write)?;
        } else {
            LpVector3d(Vector3::new(0.0, 0.0, 0.0)).write(&mut write)?;
        }
        write.write_bool(self.sneaking)?;

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
