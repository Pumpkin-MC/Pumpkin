use pumpkin_data::packet::clientbound::CONFIG_COOKIE_REQUEST;
use pumpkin_macros::client_packet;

use crate::codec::identifier::Identifier;

#[derive(serde::Serialize)]
#[client_packet(CONFIG_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    key: &'a Identifier,
}

impl<'a> CCookieRequest<'a> {
    pub fn new(key: &'a Identifier) -> Self {
        Self { key }
    }
}
