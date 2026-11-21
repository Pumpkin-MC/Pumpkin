// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Sent by the server in response to [`SRequestChunkRadius`](crate::bedrock::server::request_chunk_radius::SRequestChunkRadius) to confirm the approved chunk render distance.
#[derive(PacketWrite)]
#[packet(70)]
pub struct CChunkRadiusUpdated {
    /// Approved chunk radius (in chunks) that the server will send to the client.
    pub chunk_radius: VarInt,
}
