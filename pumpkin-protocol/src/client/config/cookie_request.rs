use pumpkin_data::packet::clientbound::CONFIG_COOKIE_REQUEST;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::identifier::Identifier;

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'a: 'de"))]
#[packet(CONFIG_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    #[serde(borrow)]
    pub key: Identifier<'a>,
}

impl<'a> CCookieRequest<'a> {
    pub fn new(key: Identifier<'a>) -> Self {
        Self { key }
    }
}
