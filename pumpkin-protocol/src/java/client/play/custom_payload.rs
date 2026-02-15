use pumpkin_data::packet::clientbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::ser::network_serialize_no_prefix;

/// A packet used for custom communication between the server and client.
#[derive(Serialize)]
#[java_packet(PLAY_CUSTOM_PAYLOAD)]
pub struct CCustomPayload<'a> {
    pub channel: &'a str,
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub data: &'a [u8],
}

impl<'a> CCustomPayload<'a> {
    #[must_use]
    pub const fn new(channel: &'a str, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}
