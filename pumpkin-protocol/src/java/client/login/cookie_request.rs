use pumpkin_data::packet::clientbound::LOGIN_COOKIE_REQUEST;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

/// Sent by the server to retrieve a previously stored cookie from the client.
///
/// This occurs during the login phase, allowing the server to identify
/// returning players or retrieve session data stored during a previous visit.
#[derive(Serialize)]
#[java_packet(LOGIN_COOKIE_REQUEST)]
pub struct CLoginCookieRequest<'a> {
    /// The unique identifier of the cookie being requested.
    pub key: &'a ResourceLocation,
}

impl<'a> CLoginCookieRequest<'a> {
    #[must_use]
    pub const fn new(key: &'a ResourceLocation) -> Self {
        Self { key }
    }
}
