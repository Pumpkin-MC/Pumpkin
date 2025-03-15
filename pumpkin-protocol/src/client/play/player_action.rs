use crate::{Property, VarInt};

pub enum PlayerAction<'a> {
    AddPlayer {
        name: &'a str,
        properties: &'a [Property],
    },
    InitializeChat(u8),
    UpdateGameMode(VarInt),
    UpdateListed(bool),
    UpdateLatency(u8),
    UpdateDisplayName(u8),
    UpdateListOrder,
}
