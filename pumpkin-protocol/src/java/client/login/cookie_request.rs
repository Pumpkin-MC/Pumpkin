use pumpkin_data::packet::clientbound::LOGIN_COOKIE_REQUEST;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

#[derive(Serialize)]
#[packet(LOGIN_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CLoginCookieRequest<'a> {
    pub key: &'a ResourceLocation,
}

impl<'a> CLoginCookieRequest<'a> {
    #[must_use]
    pub const fn new(key: &'a ResourceLocation) -> Self {
        Self { key }
    }
}
