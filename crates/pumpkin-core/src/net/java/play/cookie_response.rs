#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_cookie_response(&self, packet: &SPCookieResponse<'_>) {
        // TODO: allow plugins to access this
        debug!(
            "Received cookie_response[play]: key: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.payload.as_ref().map(|p| p.len())
        );
    }
}
