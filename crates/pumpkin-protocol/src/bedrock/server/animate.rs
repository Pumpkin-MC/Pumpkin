// Last verified for v2169

use std::{
    io::{Error, Read, Write},
    str::FromStr,
};

use pumpkin_macros::packet;

use crate::{
    bedrock::enum_as_str::EnumAsStr,
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
pub enum ActorSwingSource {
    None,
    Build,
    Mine,
    Interact,
    Attack,
    UseItem,
    ThrowItem,
    DropItem,
    Event,
}

impl FromStr for ActorSwingSource {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            source => Err(Error::other(format!("Invalid swing source: {source}"))),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ActorSwingSource {
    fn to_string(&self) -> String {
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
        .into()
    }
}

#[derive(Debug, PacketRead, PacketWrite)]
#[packet(44)]
pub struct SAnimate {
    pub action: AnimateAction,
    pub target_actor_runtime_id: VarULong,
    pub data: f32,
    pub swing_source: Option<EnumAsStr<ActorSwingSource>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animate_uses_cereal_swing_source_encoding() {
        let packet = SAnimate {
            action: AnimateAction::SwingArm,
            target_actor_runtime_id: VarULong(42),
            data: 0.0,
            swing_source: Some(ActorSwingSource::Attack.into()),
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();

        assert_eq!(encoded, b"\x01\x2a\0\0\0\0\x01\x06attack");

        let decoded = SAnimate::read(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded.action, AnimateAction::SwingArm);
        assert_eq!(decoded.target_actor_runtime_id, VarULong(42));
        assert_eq!(decoded.data, 0.0);
        assert_eq!(decoded.swing_source, Some(ActorSwingSource::Attack.into()));
    }

    #[test]
    fn animate_omits_absent_swing_source_value() {
        let packet = SAnimate {
            action: AnimateAction::NoAction,
            target_actor_runtime_id: VarULong(1),
            data: 0.0,
            swing_source: None,
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();

        assert_eq!(encoded, [0, 1, 0, 0, 0, 0, 0]);
    }
}
