//! Minecraft Bedrock Edition protocol packet definitions and codecs.
//!
//! Packets in the Bedrock protocol are framed within game packet envelopes
//! (prefixed with packet ID `0xfe`), compressed (typically using Deflate or Snappy),
//! and transmitted over `RakNet` or `NetherNet` transport layers.

pub mod client;
pub mod enum_as_str;
pub mod network_item;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod server;
pub mod status;

/// Magic packet identifier (`0xfe`) indicating a Bedrock game packet batch envelope.
pub const BEDROCK_GAME_PACKET: u8 = 0xfe;

/// Sub-client index representing local split-screen players on a client device.
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
