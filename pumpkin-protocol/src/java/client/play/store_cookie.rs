use pumpkin_data::packet::clientbound::PLAY_STORE_COOKIE;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian client only accepts cookies of up to 5 kiB in size.
#[derive(Serialize)]
#[packet(PLAY_STORE_COOKIE)]
pub struct CStoreCookie<'a> {
    key: &'a ResourceLocation,
    payload: &'a [u8], // 5120,
}

impl<'a> CStoreCookie<'a> {
    pub fn new(key: &'a ResourceLocation, payload: &'a [u8]) -> Self {
        Self { key, payload }
    }
}
