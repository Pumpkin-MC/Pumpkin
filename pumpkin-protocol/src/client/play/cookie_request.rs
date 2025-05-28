use pumpkin_data::packet::clientbound::PLAY_COOKIE_REQUEST;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::identifier::Identifier;

#[derive(Serialize)]
#[packet(PLAY_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CPlayCookieRequest<'a> {
    key: &'a Identifier,
}

impl<'a> CPlayCookieRequest<'a> {
    pub fn new(key: &'a Identifier) -> Self {
        Self { key }
    }
}
