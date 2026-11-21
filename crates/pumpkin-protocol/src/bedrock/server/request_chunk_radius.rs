// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

/// Sent by the client to request a specific chunk render distance from the server.
#[derive(PacketRead, Debug)]
#[packet(69)]
pub struct SRequestChunkRadius {
    /// Requested chunk render radius in chunks.
    pub chunk_radius: VarInt,
    /// Upper limit of chunk render radius requested by the client settings.
    pub max_chunk_radius: u8,
}
