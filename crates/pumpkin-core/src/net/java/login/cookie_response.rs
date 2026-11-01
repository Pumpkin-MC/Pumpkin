#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub fn handle_login_cookie_response(&self, packet: &SLoginCookieResponse<'_>) {
        debug!(
            "Received cookie_response[login]: key: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.payload.as_ref().map(|p| p.len())
        );
    }
}
