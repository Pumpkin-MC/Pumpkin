use std::borrow::Cow;

use pumpkin_data::packet::clientbound::PLAY_COOKIE_REQUEST;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::identifier::Identifier;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CPlayCookieRequest<'a> {
    key: Cow<'a, Identifier>,
}

impl<'a> CPlayCookieRequest<'a> {
    pub fn new(key: Cow<'a, Identifier>) -> Self {
        Self { key }
    }
}
