#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_config_cookie_response(&self, packet: &SConfigCookieResponse<'_>) {
        // TODO: allow plugins to access this
        debug!(
            "Received cookie_response[config]: key: \"{}\", has_payload: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.has_payload,
            packet.payload.as_ref().map(|p| p.len()),
        );
    }
}
