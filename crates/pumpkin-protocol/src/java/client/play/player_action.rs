use pumpkin_util::text::TextComponent;

use crate::{Property, VarInt};

use super::PlayerInfoFlags;

pub enum PlayerAction<'a> {
    AddPlayer {
        name: &'a str,
        properties: &'a [Property],
    },
    InitializeChat(Option<InitChat>),
    UpdateGameMode(VarInt),
    UpdateListed(bool),
    UpdateLatency(VarInt),
    UpdateDisplayName(Option<&'a TextComponent>),
    /// Added in 1.21.2
    UpdateListOrder(VarInt),
    /// Added in 1.21.4
    /// Toggles the visibility of the player's hat layer (second skin layer).
    UpdateHat(bool),
}

impl PlayerAction<'_> {
    /// The flag announcing action.
    #[must_use]
    pub const fn flag(&self) -> PlayerInfoFlags {
        match self {
            Self::AddPlayer { .. } => PlayerInfoFlags::ADD_PLAYER,
            Self::InitializeChat(_) => PlayerInfoFlags::INITIALIZE_CHAT,
            Self::UpdateGameMode(_) => PlayerInfoFlags::UPDATE_GAME_MODE,
            Self::UpdateListed(_) => PlayerInfoFlags::UPDATE_LISTED,
            Self::UpdateLatency(_) => PlayerInfoFlags::UPDATE_LATENCY,
            Self::UpdateDisplayName(_) => PlayerInfoFlags::UPDATE_DISPLAY_NAME,
            Self::UpdateListOrder(_) => PlayerInfoFlags::UPDATE_LIST_PRIORITY,
            Self::UpdateHat(_) => PlayerInfoFlags::UPDATE_HAT,
        }
    }
}

pub struct InitChat {
    pub session_id: uuid::Uuid,
    pub expires_at: i64,
    pub public_key: Box<[u8]>,
    pub signature: Box<[u8]>,
}
