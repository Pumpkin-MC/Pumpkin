use pumpkin_data::packet::clientbound::PLAY_COOKIE_REQUEST;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CPlayCookieRequest<'a> {
    key: &'a ResourceLocation,
}

impl<'a> CPlayCookieRequest<'a> {
    pub fn new(key: &'a ResourceLocation) -> Self {
        Self { key }
    }
}
