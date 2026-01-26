use pumpkin_data::packet::clientbound::LOGIN_CUSTOM_QUERY;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::{VarInt, ser::network_serialize_no_prefix};

/// Sent by the server to initiate a custom plugin messaging exchange during login.
///
/// This is used by server software (like proxies or anti-cheats) to request
/// information from a client-side mod before the player officially joins.
#[derive(Serialize)]
#[java_packet(LOGIN_CUSTOM_QUERY)]
pub struct CLoginPluginRequest<'a> {
    /// A unique ID for this request. The client must include this same ID
    /// in its response so the server can match them up.
    pub message_id: VarInt,
    /// The name of the custom channel (e.g., "velocity:main").
    pub channel: &'a str,
    /// The raw payload data. Unlike standard plugin messages, this data
    /// is often serialized without a length prefix at the end of the packet.
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub data: &'a [u8],
}

impl<'a> CLoginPluginRequest<'a> {
    pub fn new(message_id: VarInt, channel: &'a str, data: &'a [u8]) -> Self {
        Self {
            message_id,
            channel,
            data,
        }
    }
}
