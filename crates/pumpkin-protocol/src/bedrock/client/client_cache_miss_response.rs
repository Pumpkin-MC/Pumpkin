// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

/// Payload data for a requested chunk blob that was missing from the client cache.
#[derive(PacketWrite, Clone, Debug)]
pub struct MissingBlobData {
    /// 64-bit hash identifier of the blob.
    pub blob_id: u64,
    /// Raw serialized byte payload of the chunk blob.
    pub blob_data: Vec<u8>,
}

/// Sent by the server in response to [`SClientCacheBlobStatus`](crate::bedrock::server::client_cache_blob_status::SClientCacheBlobStatus) with requested chunk blobs.
#[derive(PacketWrite)]
#[packet(136)]
pub struct CClientCacheMissResponse {
    /// Array of blob byte payloads corresponding to missed blob hashes.
    pub missing_blobs: Vec<MissingBlobData>,
}
