use crate::{Property, VarInt};

pub enum PlayerAction<'a> {
    AddPlayer {
        name: &'a str,
        properties: &'a [Property],
    },
    InitializeChat(Option<InitChat>),
    UpdateGameMode(VarInt),
    UpdateListed(bool),
    UpdateLatency(u8),
    UpdateDisplayName(u8),
    UpdateListOrder,
}

pub struct InitChat {
    pub session_id: uuid::Uuid,
    pub expires_at: i64,
    pub public_key: Box<[u8]>,
    pub signature: Box<[u8]>,
}
