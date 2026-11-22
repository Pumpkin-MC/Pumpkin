//! Minecraft Bedrock Edition protocol packet definitions and codecs.
//!
//! Packets in the Bedrock protocol are framed within game packet envelopes
//! (prefixed with packet ID `0xfe`), compressed (typically using Deflate or Snappy),
//! and transmitted over the `NetherNet` transport layer.

pub mod client;
pub mod enum_as_str;
pub mod network_item;
pub mod network_stack_latency;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod server;

/// Magic packet identifier (`0xfe`) indicating a Bedrock game packet batch envelope.
pub const BEDROCK_GAME_PACKET: u8 = 0xfe;

/// Sub-client index representing local split-screen players on a client device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum SubClient {
    /// Primary player instance on the client.
    Main = 0,
    /// First secondary split-screen player.
    SubClient0 = 1,
    /// Second secondary split-screen player.
    SubClient1 = 2,
    /// Third secondary split-screen player.
    SubClient2 = 3,
}

impl SubClient {
    /// Converts a 2-bit game-packet header field (`0..=3`) into a [`SubClient`].
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Main),
            1 => Some(Self::SubClient0),
            2 => Some(Self::SubClient1),
            3 => Some(Self::SubClient2),
            _ => None,
        }
    }

    /// Returns `true` for split-screen sub-client slots (`1..=3`).
    #[must_use]
    pub const fn is_sub_client(self) -> bool {
        !matches!(self, Self::Main)
    }
}
