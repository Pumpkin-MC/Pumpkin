use std::borrow::Cow;

use pumpkin_data::packet::clientbound::CONFIG_COOKIE_REQUEST;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::identifier::Identifier;

#[derive(Serialize, Deserialize)]
#[packet(CONFIG_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    pub key: Cow<'a, Identifier>,
}

impl<'a> CCookieRequest<'a> {
    pub fn new(key: Cow<'a, Identifier>) -> Self {
        Self { key }
    }
}
