// Last verified for v2169

use std::io::{Error, Read};

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketRead};

#[derive(Debug, PacketRead)]
#[packet(33)]
pub struct SInteract {
    pub action: Action,
    pub target_runtime_id: VarULong,
    pub position: Option<Vector3<f32>>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum Action {
    Invalid = 0,
    StopRiding = 3,
    InteractUpdate = 4,
    NpcOpen = 5,
    OpenInventory = 6,
}

impl PacketRead for Action {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match u8::read(reader)? {
            0 => Ok(Self::Invalid),
            3 => Ok(Self::StopRiding),
            4 => Ok(Self::InteractUpdate),
            5 => Ok(Self::NpcOpen),
            6 => Ok(Self::OpenInventory),
            _ => Err(Error::other("")),
        }
    }
}
