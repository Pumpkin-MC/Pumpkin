use pumpkin_macros::{Event, cancellable};

/// An event that occurs when the server receives a custom payload (plugin message)
/// from a client during the Play state.
///
/// Plugin messages use namespaced channels (e.g. `minecraft:brand`, `velocity:player_info`)
/// to send arbitrary data between client and server.
///
/// If cancelled, the payload is silently dropped (no further processing).
///
/// Matches Bukkit's `PluginMessageEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct CustomPayloadEvent {
    /// The channel identifier (e.g. `minecraft:brand`).
    pub channel: String,

    /// The raw payload bytes.
    pub data: Vec<u8>,
}

impl CustomPayloadEvent {
    #[must_use]
    pub const fn new(channel: String, data: Vec<u8>) -> Self {
        Self {
            channel,
            data,
            cancelled: false,
        }
    }
}
