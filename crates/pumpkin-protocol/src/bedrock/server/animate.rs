use std::io::{Error, Read, Write};

use pumpkin_macros::packet;

use crate::{
    codec::var_ulong::VarULong,
    serial::{PacketRead, PacketWrite},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AnimateAction {
    NoAction = 0,
    SwingArm = 1,
    WakeUp = 3,
    CriticalHit = 4,
    MagicCriticalHit = 5,
}

impl PacketRead for AnimateAction {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let action = u8::read(reader)?;
        match action {
            0 => Ok(Self::NoAction),
            1 => Ok(Self::SwingArm),
            3 => Ok(Self::WakeUp),
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
#[repr(u8)]
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

impl PacketRead for AnimateSwingSource {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match String::read(reader)?.as_str() {
            "none" => Ok(Self::None),
            "build" => Ok(Self::Build),
            "mine" => Ok(Self::Mine),
            "interact" => Ok(Self::Interact),
            "attack" => Ok(Self::Attack),
            "useitem" => Ok(Self::UseItem),
            "throwitem" => Ok(Self::ThrowItem),
            "dropitem" => Ok(Self::DropItem),
            "event" => Ok(Self::Event),
            source => Err(Error::other(format!("Invalid swing source: {source}"))),
        }
    }
}

impl PacketWrite for AnimateSwingSource {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
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
        .write(writer)
    }
}

#[derive(Debug, PacketRead, PacketWrite)]
#[packet(44)]
pub struct SAnimate {
    pub action: AnimateAction,
    pub runtime_entity_id: VarULong,
    pub data: f32,
    pub swing_source: Option<AnimateSwingSource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animate_uses_cereal_swing_source_encoding() {
        let packet = SAnimate {
            action: AnimateAction::SwingArm,
            runtime_entity_id: VarULong(42),
            data: 0.0,
            swing_source: Some(AnimateSwingSource::Attack),
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();

        assert_eq!(encoded, b"\x01\x2a\0\0\0\0\x01\x06attack");

        let decoded = SAnimate::read(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded.action, AnimateAction::SwingArm);
        assert_eq!(decoded.runtime_entity_id, VarULong(42));
        assert_eq!(decoded.data, 0.0);
        assert_eq!(decoded.swing_source, Some(AnimateSwingSource::Attack));
    }

    #[test]
    fn animate_omits_absent_swing_source_value() {
        let packet = SAnimate {
            action: AnimateAction::NoAction,
            runtime_entity_id: VarULong(1),
            data: 0.0,
            swing_source: None,
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();

        assert_eq!(encoded, [0, 1, 0, 0, 0, 0, 0]);
    }
}
