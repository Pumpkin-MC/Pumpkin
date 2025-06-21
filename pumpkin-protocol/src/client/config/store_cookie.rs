use pumpkin_data::packet::clientbound::CONFIG_STORE_COOKIE;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

#[derive(serde::Serialize)]
#[packet(CONFIG_STORE_COOKIE)]
/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian (vanilla) client only accepts cookies of up to 5 KiB in size.
pub struct CStoreCookie<'a> {
    key: &'a ResourceLocation,
    payload: &'a [u8], // 5120,
}

impl<'a> CStoreCookie<'a> {
    pub fn new(key: &'a ResourceLocation, payload: &'a [u8]) -> Self {
        Self { key, payload }
    }
}
