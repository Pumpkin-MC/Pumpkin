use pumpkin_data::packet::clientbound::PLAY_COOKIE_REQUEST;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::identifier::Identifier;

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'a: 'de"))]
#[packet(PLAY_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CPlayCookieRequest<'a> {
    #[serde(borrow)]
    key: Identifier<'a>,
}

impl<'a> CPlayCookieRequest<'a> {
    pub fn new(key: Identifier<'a>) -> Self {
        Self { key }
    }
}
