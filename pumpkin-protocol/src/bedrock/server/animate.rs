use std::io::{Error, Read, Write};

use pumpkin_macros::packet;

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::{PacketRead, PacketWrite},
};

#[derive(Debug)]
#[packet(44)]
pub struct SAnimate {
    pub action: AnimateAction,
    pub runtime_entity_id: VarULong,
    pub boat_rowing_time: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimateAction {
    NoAction = 0,
    SwingArm = 1,
    WakeUp = 2,
    CriticalHit = 3,
    MagicCriticalHit = 4,
    RowRight = 128,
    RowLeft = 129,
}

impl PacketRead for AnimateAction {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let action = u8::read(reader)?;
        match action {
            0 => Ok(Self::NoAction),
            1 => Ok(Self::SwingArm),
            2 => Ok(Self::WakeUp),
            3 => Ok(Self::CriticalHit),
            4 => Ok(Self::MagicCriticalHit),
            128 => Ok(Self::RowRight),
            129 => Ok(Self::RowLeft),
            _ => Err(Error::other(format!(
                "Invalid animate action ID: {}",
                action
            ))),
        }
    }
}

impl PacketWrite for AnimateAction {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}

impl PacketRead for SAnimate {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let action = AnimateAction::read(reader)?;
        let runtime_entity_id = VarULong::read(reader)?;
        let boat_rowing_time =
            if action == AnimateAction::RowRight || action == AnimateAction::RowLeft {
                Some(f32::read(reader)?)
            } else {
                None
            };
        let _swing_source = bool::read(reader)?;
        Ok(Self {
            action,
            runtime_entity_id,
            boat_rowing_time,
        })
    }
}

impl PacketWrite for SAnimate {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.action.write(writer)?;
        self.runtime_entity_id.write(writer)?;
        if let Some(rowing_time) = self.boat_rowing_time {
            rowing_time.write(writer)?;
        }
        // swingSource
        false.write(writer)?;
        Ok(())
    }
}
