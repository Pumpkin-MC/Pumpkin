// Last verified for v2169

use std::{io::Error, str::FromStr};

use pumpkin_macros::packet;

use crate::{
    bedrock::enum_as_str::EnumAsStr,
    codec::var_ulong::VarULong,
    serial::{PacketRead, PacketWrite},
};

/// Animation action performed on an actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PacketRead, PacketWrite)]
#[repr(u8)]
pub enum AnimateAction {
    NoAction = 0,
    /// Arm swinging animation.
    SwingArm = 1,
    /// Waking up from bed animation.
    WakeUp = 3,
    /// Critical hit particle burst.
    CriticalHit = 4,
    /// Magic critical hit sparkle burst.
    MagicCriticalHit = 5,
}

/// Cause or trigger for an arm swing action.
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

/// Plays animations (arm swing, critical strike particles, waking up) on an actor.
#[derive(Debug, PacketRead, PacketWrite)]
#[packet(44)]
pub struct SAnimate {
    /// Type of animation to display.
    pub action: AnimateAction,
    /// Runtime entity ID of the target actor.
    pub target_actor_runtime_id: VarULong,
    /// Floating point payload (used for rowing animation angles).
    pub data: f32,
    /// Source context that triggered an arm swing.
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
