//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/server/play/interact.rs`.

use crate::java::legacy::LegacyRead;
use crate::java::server::play::ActionType;
use crate::java::server::play::SInteract;
use crate::{
    codec::{lp_vector_3d::LpVector3d, var_int::VarInt},
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

impl<'a> LegacyRead<'a> for SInteract {
    fn read_legacy(
        mut read: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
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
