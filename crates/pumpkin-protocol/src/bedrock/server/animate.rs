use std::{
    io::{Error, Read, Write},
    str::FromStr,
};

use pumpkin_macros::packet;

use crate::{
    codec::var_ulong::VarULong,
    serial::{PacketRead, PacketWrite},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimateAction {
    SwingArm = 1,
    WakeUp = 2,
    StopSleep = 3,
    CriticalHit = 4,
    MagicCriticalHit = 5,
}

impl PacketRead for AnimateAction {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let action = u8::read(reader)?;
        match action {
            1 => Ok(Self::SwingArm),
            2 => Ok(Self::WakeUp),
            3 => Ok(Self::StopSleep),
            4 => Ok(Self::CriticalHit),
            5 => Ok(Self::MagicCriticalHit),
            _ => Err(Error::other(format!("Invalid animate action ID: {action}"))),
        }
    }
}

impl PacketWrite for AnimateAction {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimateSwingSource {
    None = 1,
    Build = 2,
    Mine = 3,
    Interact = 4,
    Attack = 5,
    UseItem = 6,
    ThrowItem = 7,
    DropItem = 8,
    Event = 9,
}

impl FromStr for AnimateSwingSource {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "none" => Ok(Self::None),
            "build" => Ok(Self::Build),
            "mine" => Ok(Self::Mine),
            "interact" => Ok(Self::Interact),
            "attack" => Ok(Self::Attack),
            "useitem" => Ok(Self::UseItem),
            "throwitem" => Ok(Self::ThrowItem),
            "dropitem" => Ok(Self::DropItem),
            "event" => Ok(Self::Event),
            _ => Err(Error::other(format!("Unknown swing source: {s}"))),
        }
    }
}

impl AnimateSwingSource {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Build => "build",
            Self::Mine => "mine",
            Self::Interact => "interact",
            Self::Attack => "attack",
            Self::UseItem => "useitem",
            Self::ThrowItem => "throwitem",
            Self::DropItem => "dropitem",
            Self::Event => "event",
        }
    }
}

#[derive(Debug)]
#[packet(44)]
pub struct SAnimate {
    pub action: AnimateAction,
    pub runtime_entity_id: VarULong,
    pub data: f32,
    pub swing_source: Option<AnimateSwingSource>,
}

impl PacketRead for SAnimate {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let action = AnimateAction::read(reader)?;
        let runtime_entity_id = VarULong::read(reader)?;
        let data = f32::read(reader)?;

        let swing_source_str = Option::<String>::read(reader)?;
        let swing_source = match swing_source_str {
            Some(s) => Some(AnimateSwingSource::from_str(&s)?),
            None => None,
        };

        Ok(Self {
            action,
            runtime_entity_id,
            data,
            swing_source,
        })
    }
}

impl PacketWrite for SAnimate {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.action.write(writer)?;
        self.runtime_entity_id.write(writer)?;
        self.data.write(writer)?;

        let swing_source_str: Option<String> = self.swing_source.map(|s| s.as_str().to_string());
        swing_source_str.write(writer)?;

        Ok(())
    }
}
