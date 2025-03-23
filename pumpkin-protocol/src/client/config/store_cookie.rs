use std::borrow::Cow;

use crate::{VarInt, codec::identifier::Identifier};
use pumpkin_data::packet::clientbound::CONFIG_STORE_COOKIE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(CONFIG_STORE_COOKIE)]
/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian (vanilla) client only accepts cookies of up to 5 KiB in size.
pub struct CStoreCookie<'a> {
    key: Cow<'a, Identifier>,
    payload_length: VarInt,
    payload: &'a [u8], // 5120,
}

impl<'a> CStoreCookie<'a> {
    pub fn new(key: Cow<'a, Identifier>, payload: &'a [u8]) -> Self {
        Self {
            key,
            payload_length: VarInt(payload.len() as i32),
            payload,
        }
    }
}
