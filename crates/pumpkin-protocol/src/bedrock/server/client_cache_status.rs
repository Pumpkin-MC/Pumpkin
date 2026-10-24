// Last verified for v2169

use crate::serial::PacketRead;
use pumpkin_macros::packet;

/// Sent by the client during login to inform the server whether client-side chunk blob caching is supported.
#[derive(PacketRead)]
#[packet(129)]
pub struct SClientCacheStatus {
    /// True if the client supports caching sub-chunk blobs by hash.
    pub is_cache_supported: bool,
}
