use pumpkin_data::packet::clientbound::CONFIG_COOKIE_REQUEST;
use pumpkin_macros::packet;

use crate::codec::identifier::Identifier;

#[derive(serde::Serialize)]
#[packet(CONFIG_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    pub key: &'a Identifier,
}

impl<'a> CCookieRequest<'a> {
    pub fn new(key: &'a Identifier) -> Self {
        Self { key }
    }
}
