use pumpkin_data::packet::clientbound::LOGIN_COOKIE_REQUEST;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::identifier::Identifier;

#[derive(Serialize)]
#[packet(LOGIN_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CLoginCookieRequest<'a> {
    key: Identifier<'a>,
}

impl<'a> CLoginCookieRequest<'a> {
    pub fn new(key: Identifier<'a>) -> Self {
        Self { key }
    }
}
