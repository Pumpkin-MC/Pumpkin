use crate::{VarInt, codec::identifier::Identifier};
use pumpkin_data::packet::clientbound::PLAY_STORE_COOKIE;
use pumpkin_macros::packet;
use serde::Serialize;

/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian client only accepts cookies of up to 5 kiB in size.
#[derive(Serialize)]
#[packet(PLAY_STORE_COOKIE)]
pub struct CStoreCookie<'a> {
    key: &'a Identifier,
    payload_length: VarInt,
    payload: &'a [u8], // 5120,
}

impl<'a> CStoreCookie<'a> {
    pub fn new(key: &'a Identifier, payload: &'a [u8]) -> Self {
        Self {
            key,
            payload_length: VarInt(payload.len() as i32),
            payload,
        }
    }
}
