use pumpkin_util::text::TextComponent;

use crate::{Property, VarInt};

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

pub struct InitChat {
    pub session_id: uuid::Uuid,
    pub expires_at: i64,
    pub public_key: Box<[u8]>,
    pub signature: Box<[u8]>,
}
