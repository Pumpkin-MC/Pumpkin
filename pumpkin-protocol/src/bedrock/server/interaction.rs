use crate::{
    ServerPacket,
    codec::var_ulong::VarULong,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_macros::packet;

#[derive(Debug)]
#[packet(33)]
pub struct SInteraction {
    pub action: Action,
    pub target_runtime_id: VarULong,
}

impl ServerPacket for SInteraction {
    fn read(mut read: impl std::io::Read) -> Result<Self, crate::ser::ReadingError> {
        let action = match read.get_i8()? {
            0 => Action::Invalid,
            3 => Action::StopRiding,
            4 => Action::InteractUpdate,
            5 => Action::NpcOpen,
            6 => Action::OpenInventory,
            _ => return Err(ReadingError::Message("Invalid Action".to_string())),
        };
        let target_runtime_id = read.get_var_ulong()?;
        Ok(SInteraction {
            action,
            target_runtime_id,
        })
    }
}

#[derive(Debug)]
#[repr(i8)]
pub enum Action {
    Invalid = 0,
    StopRiding = 3,
    InteractUpdate = 4,
    NpcOpen = 5,
    OpenInventory = 6,
}
