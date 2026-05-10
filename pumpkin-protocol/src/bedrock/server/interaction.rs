use std::io::{Error, Read};

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketRead};

#[derive(Debug, PacketRead)]
#[packet(33)]
pub struct SInteraction {
    // https://mojang.github.io/bedrock-protocol-docs/html/InteractPacket.html
    pub action: Action,
    pub target_runtime_id: VarULong,
    pub position: Option<Vector3<f32>>,
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

impl PacketRead for Action {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut byte = [0];
        reader.read_exact(&mut byte)?;

        let this = match byte[0] {
            0 => Self::Invalid,
            3 => Self::StopRiding,
            4 => Self::InteractUpdate,
            5 => Self::NpcOpen,
            6 => Self::OpenInventory,
            _ => return Err(Error::other("")),
        };

        Ok(this)
    }
}
