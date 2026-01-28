use pumpkin_data::packet::clientbound::CONFIG_COOKIE_REQUEST;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

#[derive(serde::Serialize)]
#[java_packet(CONFIG_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    pub key: &'a ResourceLocation,
}

impl<'a> CCookieRequest<'a> {
    #[must_use]
    pub fn new(key: &'a ResourceLocation) -> Self {
        Self { key }
    }
}
